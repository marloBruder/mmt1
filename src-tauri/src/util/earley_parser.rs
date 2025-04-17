use crate::{model::SymbolNumberMapping, Error};

pub struct Grammar {
    pub rules: Vec<GrammarRule>,
}

pub struct ExtendedGrammar<'a> {
    pub grammar: &'a Grammar,
    pub main_rule: GrammarRule,
}

pub struct GrammarRule {
    pub left_side: u32,
    pub right_side: Vec<u32>,
}

#[derive(Clone, PartialEq, Debug)]
struct State {
    pub rule_i: i32,
    pub processed_i: u32,
    pub start_i: u32,
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

    pub fn rule_left_side(&self, extended_grammar: &ExtendedGrammar) -> u32 {
        extended_grammar
            .grammar
            .rules
            .get(self.rule_i as usize)
            .unwrap_or(&extended_grammar.main_rule)
            .left_side
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
        },
    };

    let mut state_sets: Vec<Vec<State>> = vec![vec![State {
        rule_i: -1,
        processed_i: 0,
        start_i: 0,
    }]];

    for k in 0..(expression.len() as u32 + 1) {
        state_sets.push(Vec::new());
        let mut next_state_i = 0;
        while let Some(state) = state_sets
            .get(k as usize)
            .unwrap()
            .get(next_state_i)
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
            next_state_i += 1;
        }
    }

    for k in 0..(expression.len() + 1) {
        println!("{}:", k);
        for state in state_sets.get(k).unwrap() {
            println!("{:?}", state)
        }
    }

    Ok(state_sets
        .get(expression.len())
        .ok_or(Error::InternalLogicError)?
        .contains(&State {
            rule_i: -1,
            processed_i: match_against_len as u32,
            start_i: 0,
        }))
}

fn predictor(
    state: &State,
    k: u32,
    extended_grammar: &ExtendedGrammar,
    state_sets: &mut Vec<Vec<State>>,
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
            };

            if !current_set.contains(&new_state) {
                current_set.push(new_state);
            }
        }
    }

    Ok(())
}

fn scanner(
    state: &State,
    k: u32,
    expression: &Vec<u32>,
    extended_grammar: &ExtendedGrammar,
    state_sets: &mut Vec<Vec<State>>,
) -> Result<(), Error> {
    println!("Scanning state: {:?}", state);
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
        };

        if !next_set.contains(&new_state) {
            println!("Added after scanning!");
            next_set.push(new_state);
        }
    }

    Ok(())
}

fn completer(
    state: &State,
    k: u32,
    extended_grammar: &ExtendedGrammar,
    state_sets: &mut Vec<Vec<State>>,
) -> Result<(), Error> {
    let mut i = 0;
    while let Some(other_state) = state_sets
        .get(state.start_i as usize)
        .ok_or(Error::InternalLogicError)?
        .get(i)
    {
        if Some(state.rule_left_side(extended_grammar)) == other_state.next_token(extended_grammar)
        {
            let new_state = State {
                rule_i: other_state.rule_i as i32,
                processed_i: other_state.processed_i + 1,
                start_i: other_state.start_i,
            };

            let current_set = state_sets
                .get_mut(k as usize)
                .ok_or(Error::InternalLogicError)?;

            if !current_set.contains(&new_state) {
                current_set.push(new_state);
            }
        }
        i += 1;
    }

    Ok(())
}
