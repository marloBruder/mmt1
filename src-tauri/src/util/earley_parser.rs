use std::{
    cmp::Ordering,
    collections::{BinaryHeap, HashSet},
    hash::Hash,
};

use crate::{
    model::{ParseTree, SymbolNumberMapping},
    Error,
};

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
    pub var_order: Vec<u32>,
}

#[derive(Debug)]
struct StateSet {
    unprocessed_states: Vec<State>,
    unprocessed_states_set: HashSet<StateRaw>,
    processed_states: Vec<State>,
    processed_states_set: HashSet<StateRaw>,
}

#[derive(Clone, Debug)]
struct State {
    pub rule_i: i32,
    pub processed_i: u32,
    pub start_i: u32,
    pub parse_trees: Vec<ParseTree>,
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
struct StateRaw {
    pub rule_i: i32,
    pub processed_i: u32,
    pub start_i: u32,
}

impl StateSet {
    pub fn new() -> StateSet {
        StateSet {
            unprocessed_states: Vec::new(),
            unprocessed_states_set: HashSet::new(),
            processed_states: Vec::new(),
            processed_states_set: HashSet::new(),
        }
    }

    pub fn insert(&mut self, state: State) {
        let state_raw = state.to_raw();
        if !self.processed_states_set.contains(&state_raw)
            && !self.unprocessed_states_set.contains(&state_raw)
        {
            self.unprocessed_states_set.insert(state_raw);
            self.unprocessed_states.push(state);
        }
    }

    pub fn get_next(&mut self) -> Option<&State> {
        let state = self.unprocessed_states.pop()?;
        let state_raw = state.to_raw();
        self.unprocessed_states_set.remove(&state_raw);

        self.processed_states_set.insert(state_raw);
        self.processed_states.push(state);
        self.processed_states.last()
    }

    pub fn get_processed(&self, i: usize) -> Option<&State> {
        self.processed_states.get(i)
    }

    pub fn take_processed(&mut self, state: &State) -> Option<State> {
        for (i, processed_state) in self.processed_states.iter().enumerate() {
            if state == processed_state {
                return Some(self.processed_states.swap_remove(i));
            }
        }
        None
    }
}

impl PartialEq for State {
    // Implement PartialEq in a way that ignores the parse_trees,
    // as they are just additional information and should not impact which states are considered equal
    fn eq(&self, other: &Self) -> bool {
        self.rule_i == other.rule_i
            && self.processed_i == other.processed_i
            && self.start_i == other.start_i
    }
}

impl Eq for State {}

impl PartialOrd for State {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for State {
    // Implement Ord in a way that ignores the parse_trees,
    // as they are just additional information and should not impact which states are considered equal
    fn cmp(&self, other: &Self) -> Ordering {
        if self.rule_i < other.rule_i {
            return Ordering::Less;
        }
        if self.rule_i > other.rule_i {
            return Ordering::Greater;
        }
        if self.processed_i < other.processed_i {
            return Ordering::Less;
        }
        if self.processed_i > other.processed_i {
            return Ordering::Greater;
        }
        if self.start_i < other.start_i {
            return Ordering::Less;
        }
        if self.start_i > other.start_i {
            return Ordering::Greater;
        }
        Ordering::Equal
    }
}

impl Hash for State {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.rule_i.hash(state);
        self.processed_i.hash(state);
        self.start_i.hash(state);
    }
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

    pub fn to_raw(&self) -> StateRaw {
        StateRaw {
            rule_i: self.rule_i,
            processed_i: self.processed_i,
            start_i: self.start_i,
        }
    }
}

pub fn earley_parse(
    grammar: &Grammar,
    expression: &Vec<u32>,
    match_against: Vec<u32>,
    symbol_number_mapping: &SymbolNumberMapping,
) -> Result<Option<Vec<ParseTree>>, Error> {
    let match_against_len = match_against.len();

    let extended_grammar = ExtendedGrammar {
        grammar,
        main_rule: GrammarRule {
            left_side: 0,
            right_side: match_against,
            label: String::new(),
            var_order: Vec::new(), // never accessed
        },
    };

    let mut state_sets: Vec<StateSet> = vec![StateSet::new()];
    state_sets.get_mut(0).unwrap().insert(State {
        rule_i: -1,
        processed_i: 0,
        start_i: 0,
        parse_trees: Vec::new(),
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

    // println!("{:?}", state_sets.get(expression.len()));

    let ret = state_sets
        .get_mut(expression.len())
        .ok_or(Error::InternalLogicError)?
        .take_processed(&State {
            rule_i: -1,
            processed_i: match_against_len as u32,
            start_i: 0,
            parse_trees: Vec::new(),
        });

    if ret.is_none() {
        for k in 0..(expression.len() + 1) {
            println!("{}:", k);
            for state in &state_sets.get(k).unwrap().processed_states {
                println!(
                    "{} ::= {:?}",
                    state.rule(&extended_grammar).left_side,
                    state.rule(&extended_grammar).right_side
                );
                print!("{:?} ", state.rule(&extended_grammar).label);
                println!("{:?}", state);
            }
        }
    }

    Ok(ret.map(|s| s.parse_trees))
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
                parse_trees: Vec::new(),
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
            parse_trees: state.parse_trees.clone(),
            // parse_trees: Vec::new(),
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
            let mut new_parse_trees = other_state.parse_trees.clone();
            if state.rule_i < 0 {
                return Err(Error::InternalLogicError);
            }
            new_parse_trees.push(ParseTree {
                nodes: state.parse_trees.clone(),
                rule: state.rule_i as u32,
            });

            let new_state = State {
                rule_i: other_state.rule_i,
                processed_i: other_state.processed_i + 1,
                start_i: other_state.start_i,
                parse_trees: new_parse_trees,
                // parse_trees: Vec::new(),
            };

            // println!("{:?}", new_state.proof);

            let current_set = state_sets
                .get_mut(k as usize)
                .ok_or(Error::InternalLogicError)?;

            current_set.insert(new_state);
        }
        i += 1;
    }

    Ok(())
}
