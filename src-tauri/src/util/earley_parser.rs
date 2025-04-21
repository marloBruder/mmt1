use crate::{model::SymbolNumberMapping, Error};

#[derive(Debug, Default)]
pub struct Grammar {
    pub rules: Vec<GrammarRule>,
}

pub struct ExtendedGrammar<'a> {
    pub grammar: &'a Grammar,
    pub main_rule: GrammarRule,
}

#[derive(Debug)]
pub struct GrammarRule {
    pub left_side: u32,
    pub right_side: Vec<u32>,
    pub label: String,
}

#[derive(Clone, PartialEq, Debug, Eq, PartialOrd, Ord)]
struct State {
    pub rule_i: i32,
    pub processed_i: u32,
    pub start_i: u32,
    pub proof: Vec<String>,
}

impl State {
    pub fn next_token(&self, extended_grammar: &ExtendedGrammar) -> Option<u32> {
        extended_grammar
            .grammar
            .rules
            .get(self.rule_i as usize)
            .unwrap_or(&extended_grammar.main_rule)
            .right_side
            .get(self.processed_i as usize)
            .map(|t| *t)
    }

    pub fn rule<'a>(&self, extended_grammar: &'a ExtendedGrammar) -> &'a GrammarRule {
        extended_grammar
            .grammar
            .rules
            .get(self.rule_i as usize)
            .unwrap_or(&extended_grammar.main_rule)
    }
}

pub fn earley_parse(
    grammar: &Grammar,
    expression: &Vec<u32>,
    match_against: Vec<u32>,
    symbol_number_mapping: &SymbolNumberMapping,
) -> Result<bool, Error> {
    let match_against_len = match_against.len();

    let extended_grammar = ExtendedGrammar {
        grammar,
        main_rule: GrammarRule {
            left_side: 0,
            right_side: match_against,
            label: String::new(),
        },
    };

    let mut state_sets: Vec<StateSet> = vec![StateSet::new()];
    state_sets.get_mut(0).unwrap().insert(State {
        rule_i: -1,
        processed_i: 0,
        start_i: 0,
        proof: Vec::new(),
    });

    for k in 0..(expression.len() as u32 + 1) {
        state_sets.push(StateSet::new());
        while let Some(state) = state_sets
            .get_mut(k as usize)
            .unwrap()
            .get_next()
            .map(|s| s.clone())
        {
            // if state is not finished
            if let Some(num) = state.next_token(&extended_grammar) {
                //if the next element of state is a nonterminal
                if symbol_number_mapping.is_typecode(num) {
                    predictor(&state, k, &extended_grammar, &mut state_sets)?;
                } else {
                    scanner(&state, k, expression, &extended_grammar, &mut state_sets)?;
                }
            } else {
                completer(&state, k, &extended_grammar, &mut state_sets)?;
            }
        }
    }

    // for k in 0..(expression.len() + 1) {
    //     println!("{}:", k);
    //     for state in state_sets.get(k).unwrap() {
    //         println!("{:?}", state)
    //     }
    // }

    // Ok(state_sets
    //     .get(expression.len())
    //     .ok_or(Error::InternalLogicError)?
    //     .processed_contains(&State {
    //         rule_i: -1,
    //         processed_i: match_against_len as u32,
    //         start_i: 0,

    //     }))
    Ok(true)
}

fn predictor(
    state: &State,
    k: u32,
    extended_grammar: &ExtendedGrammar,
    state_sets: &mut Vec<StateSet>,
) -> Result<(), Error> {
    for (rule_i, rule) in extended_grammar.grammar.rules.iter().enumerate() {
        if Some(rule.left_side) == state.next_token(extended_grammar) {
            let current_set = state_sets
                .get_mut(k as usize)
                .ok_or(Error::InternalLogicError)?;

            let new_state = State {
                rule_i: rule_i as i32,
                processed_i: 0,
                start_i: k,
                proof: Vec::new(),
            };

            current_set.insert(new_state);
        }
    }

    Ok(())
}

fn scanner(
    state: &State,
    k: u32,
    expression: &Vec<u32>,
    extended_grammar: &ExtendedGrammar,
    state_sets: &mut Vec<StateSet>,
) -> Result<(), Error> {
    if state.start_i < expression.len() as u32
        && state.next_token(extended_grammar) == expression.get(k as usize).map(|num| *num)
    {
        let next_set = state_sets
            .get_mut((k + 1) as usize)
            .ok_or(Error::InternalLogicError)?;

        let new_state = State {
            rule_i: state.rule_i,
            processed_i: state.processed_i + 1,
            start_i: state.start_i,
            proof: state.proof.clone(),
        };

        next_set.insert(new_state);
    }

    Ok(())
}

fn completer(
    state: &State,
    k: u32,
    extended_grammar: &ExtendedGrammar,
    state_sets: &mut Vec<StateSet>,
) -> Result<(), Error> {
    let mut i = 0;
    while let Some(other_state) = state_sets
        .get(state.start_i as usize)
        .ok_or(Error::InternalLogicError)?
        .get_processed(i)
    {
        if Some(state.rule(extended_grammar).left_side) == other_state.next_token(extended_grammar)
        {
            let mut new_proof = other_state.proof.clone();
            if !state.proof.is_empty() {
                new_proof.push(state.proof.join(" "));
                new_proof.last_mut().unwrap().push(' ');
                new_proof
                    .last_mut()
                    .unwrap()
                    .push_str(&state.rule(extended_grammar).label);
            } else {
                new_proof.push(state.rule(extended_grammar).label.clone());
            }

            let new_state = State {
                rule_i: other_state.rule_i as i32,
                processed_i: other_state.processed_i + 1,
                start_i: other_state.start_i,
                proof: new_proof,
            };

            println!("{:?}", new_state.proof);

            let current_set = state_sets
                .get_mut(k as usize)
                .ok_or(Error::InternalLogicError)?;

            current_set.insert(new_state);
        }
        i += 1;
    }

    Ok(())
}

struct StateSet {
    unprocessed_states: Vec<State>,
    processed_states: Vec<State>,
}

impl StateSet {
    pub fn new() -> StateSet {
        StateSet {
            unprocessed_states: Vec::new(),
            processed_states: Vec::new(),
        }
    }

    pub fn insert(&mut self, state: State) {
        if self.processed_states.binary_search(&state).is_err() {
            match self.unprocessed_states.binary_search(&state) {
                Ok(_pos) => {}
                Err(pos) => self.unprocessed_states.insert(pos, state),
            }
        }
    }

    pub fn get_next(&mut self) -> Option<&State> {
        let state = self.unprocessed_states.pop()?;
        let insert_pos;
        match self.processed_states.binary_search(&state) {
            Ok(pos) => insert_pos = pos, // Should theoretically never happen
            Err(pos) => {
                self.processed_states.insert(pos, state);
                insert_pos = pos;
            }
        }
        self.processed_states.get(insert_pos)
    }

    pub fn get_processed(&self, i: usize) -> Option<&State> {
        self.processed_states.get(i)
    }

    pub fn processed_contains(&self, state: &State) -> bool {
        self.processed_states.binary_search(state).is_ok()
    }
}
