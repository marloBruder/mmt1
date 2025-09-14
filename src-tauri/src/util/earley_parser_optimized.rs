use std::{cmp::Ordering, hash::Hash};

use crate::{
    model::{ParseTreeNode, SymbolNumberMapping},
    Error,
};

#[derive(Debug, Default)]
pub struct Grammar {
    pub rules: Vec<GrammarRule>,
    pub earley_optimized_data: EarleyOptimizedData,
}

pub struct ExtendedGrammar<'a> {
    pub grammar: &'a Grammar,
    pub main_rule: GrammarRule,
}

#[derive(Debug)]
pub struct GrammarRule {
    pub left_side: Symbol,
    pub right_side: Vec<Symbol>,
    pub label: String,
    pub var_order: Vec<u32>,
    pub is_floating_hypothesis: bool,
}

#[derive(Debug)]
pub enum InputSymbol {
    Symbol(Symbol),
    WorkVariable(WorkVariable),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Symbol {
    pub symbol_i: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WorkVariable {
    pub typecode_i: u32,
    pub variable_i: u32,
    pub number: u32,
}

#[derive(Debug, Default)]
pub struct EarleyOptimizedData {
    pub completer_rules: Vec<Vec<Vec<usize>>>,
    pub combined_states_to_add: Vec<Vec<u32>>,
    pub single_states_to_add: Vec<Vec<Vec<usize>>>,
}

#[derive(Debug)]
struct StateSet {
    unprocessed_states: Vec<State>,
    processed_states: Vec<State>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum State {
    Single(SingleState),
    Combined(CombinedState),
}

#[derive(Clone, Debug)]
struct SingleState {
    pub rule_i: i32,
    pub processed_i: u32,
    pub start_i: u32,
    pub parse_trees: Vec<ParseTreeNode>,
}

#[derive(Debug, Clone, Copy)]
struct CombinedState {
    pub typecode: u32,
    pub start_i: u32,
}

// #[derive(Debug)]
// pub enum StateRaw {
//     Single(SingleStateRaw),
//     Combined(CombinedState),
// }

// #[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
// struct SingleStateRaw {
//     pub rule_i: i32,
//     pub processed_i: u32,
//     pub start_i: u32,
// }

impl StateSet {
    pub fn new() -> StateSet {
        StateSet {
            unprocessed_states: Vec::new(),
            // unprocessed_states_set: HashSet::new(),
            processed_states: Vec::new(),
            // processed_states_set: HashSet::new(),
        }
    }

    pub fn insert(&mut self, state: State) {
        match &state {
            State::Single(_single_state) => {
                self.unprocessed_states.push(state);
            }
            State::Combined(combined_state) => {
                for existing_state in self
                    .unprocessed_states
                    .iter()
                    .chain(self.processed_states.iter())
                {
                    if let State::Combined(existing_combined_state) = existing_state {
                        if existing_combined_state == combined_state {
                            return;
                        }
                    }
                }
                self.unprocessed_states.push(state);
            }
        }
    }

    pub fn get_next(&mut self) -> Option<State> {
        let state = self.unprocessed_states.pop()?;
        self.processed_states.push(state.clone());
        Some(state)
    }

    pub fn get_processed(&self, i: usize) -> Option<&State> {
        self.processed_states.get(i)
    }

    pub fn take_processed(self, state: &SingleState) -> Option<SingleState> {
        for processed_state in self.processed_states {
            if let State::Single(processed_single_state) = processed_state {
                if processed_single_state == *state {
                    return Some(processed_single_state);
                }
            }
        }
        None
    }
}

impl PartialEq for SingleState {
    // Implement PartialEq in a way that ignores the parse_trees,
    // as they are just additional information and should not impact which states are considered equal
    fn eq(&self, other: &Self) -> bool {
        self.rule_i == other.rule_i
            && self.processed_i == other.processed_i
            && self.start_i == other.start_i
    }
}

impl Eq for SingleState {}

impl PartialEq for CombinedState {
    // Implement PartialEq in a way that ignores next_grammare_rule_i,
    // as it is just additional information and should not impact which states are considered equal
    fn eq(&self, other: &Self) -> bool {
        self.typecode == other.typecode && self.start_i == other.start_i
    }
}

impl Eq for CombinedState {}

impl PartialOrd for SingleState {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SingleState {
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

impl Hash for SingleState {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.rule_i.hash(state);
        self.processed_i.hash(state);
        self.start_i.hash(state);
    }
}

impl SingleState {
    pub fn next_token(&self, extended_grammar: &ExtendedGrammar) -> Option<Symbol> {
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

    // pub fn to_raw(&self) -> SingleStateRaw {
    //     SingleStateRaw {
    //         rule_i: self.rule_i,
    //         processed_i: self.processed_i,
    //         start_i: self.start_i,
    //     }
    // }
}

pub fn earley_parse(
    grammar: &Grammar,
    expression: &Vec<InputSymbol>,
    match_against: Vec<Symbol>,
    symbol_number_mapping: &SymbolNumberMapping,
) -> Result<Option<Vec<ParseTreeNode>>, Error> {
    if expression.is_empty() {
        return Ok(None);
    }

    let match_against_len = match_against.len();

    let extended_grammar = ExtendedGrammar {
        grammar,
        main_rule: GrammarRule {
            left_side: Symbol { symbol_i: 0 },
            right_side: match_against,
            label: String::new(),
            var_order: Vec::new(), // never accessed
            is_floating_hypothesis: false,
        },
    };

    let mut state_sets: Vec<StateSet> = vec![StateSet::new()];
    state_sets
        .get_mut(0)
        .ok_or(Error::InternalLogicError)?
        .insert(State::Single(SingleState {
            rule_i: -1,
            processed_i: 0,
            start_i: 0,
            parse_trees: Vec::new(),
        }));

    for k in 0..(expression.len() as u32 + 1) {
        state_sets.push(StateSet::new());
        while let Some(state) = state_sets.get_mut(k as usize).unwrap().get_next() {
            match state {
                State::Single(state) => {
                    // if state is not finished
                    if let Some(num) = state.next_token(&extended_grammar) {
                        // if the next element of state is a nonterminal
                        if symbol_number_mapping.is_typecode(num.symbol_i) {
                            predictor(&state, k, &extended_grammar, &mut state_sets)?;
                        } else {
                            scanner(&state, k, expression, &extended_grammar, &mut state_sets)?;
                        }
                    } else {
                        completer(&state, k, &extended_grammar, &mut state_sets)?;
                    }
                }
                State::Combined(combined_state) => {
                    // simulate predictor
                    let current_set = state_sets
                        .get_mut(k as usize)
                        .ok_or(Error::InternalLogicError)?;

                    for &typecode in grammar
                        .earley_optimized_data
                        .combined_states_to_add
                        .get(combined_state.typecode as usize - 1)
                        .ok_or(Error::InternalLogicError)?
                    {
                        let new_state = State::Combined(CombinedState {
                            typecode,
                            start_i: k,
                        });

                        current_set.insert(new_state);
                    }

                    //simulate scanner
                    if k < expression.len() as u32 {
                        let next_set = state_sets
                            .get_mut(k as usize + 1)
                            .ok_or(Error::InternalLogicError)?;

                        match expression
                            .get(k as usize)
                            .ok_or(Error::InternalLogicError)?
                        {
                            InputSymbol::Symbol(symbol) => {
                                for &rule in grammar
                                    .earley_optimized_data
                                    .single_states_to_add
                                    .get(combined_state.typecode as usize - 1)
                                    .ok_or(Error::InternalLogicError)?
                                    .get(
                                        (symbol.symbol_i - symbol_number_mapping.typecode_count - 1)
                                            as usize,
                                    )
                                    .ok_or(Error::InternalLogicError)?
                                {
                                    let new_state = State::Single(SingleState {
                                        rule_i: rule as i32,
                                        processed_i: 1,
                                        start_i: combined_state.start_i,
                                        parse_trees: Vec::new(),
                                    });

                                    next_set.insert(new_state);
                                }
                            }
                            InputSymbol::WorkVariable(work_variable) => {
                                let new_state = State::Single(SingleState {
                                    rule_i: work_variable.typecode_i as i32 - 1,
                                    processed_i: 1,
                                    start_i: combined_state.start_i,
                                    parse_trees: vec![ParseTreeNode::WorkVariable(*work_variable)],
                                });

                                next_set.insert(new_state);
                            }
                        }
                    }
                }
            }
        }
    }

    // println!("{:?}", state_sets.get(expression.len()));
    state_sets.pop();

    let ret = state_sets
        .pop()
        .ok_or(Error::InternalLogicError)?
        .take_processed(&SingleState {
            rule_i: -1,
            processed_i: match_against_len as u32,
            start_i: 0,
            parse_trees: Vec::new(),
        });

    // if ret.is_none() {
    // if expression
    //     .iter()
    //     .any(|symbol| matches!(symbol, InputSymbol::WorkVariable(_)))
    // {
    //     println!("\n\nEarley parser for: {:?}", expression);
    //     for k in 0..(expression.len()) {
    //         println!("{}:", k);
    //         for state in &state_sets.get(k).unwrap().processed_states {
    //             if let State::Single(single_state) = state {
    //                 println!(
    //                     "{} ::= {:?}",
    //                     single_state.rule(&extended_grammar).left_side.symbol_i,
    //                     single_state
    //                         .rule(&extended_grammar)
    //                         .right_side
    //                         .iter()
    //                         .map(|symbol| symbol.symbol_i)
    //                         .collect::<Vec<u32>>()
    //                 );
    //                 print!("{:?} ", single_state.rule(&extended_grammar).label);
    //             }
    //             println!("{:?}", state);
    //         }
    //     }
    // }

    Ok(ret.map(|s| s.parse_trees))
    //.map(|s| s.parse_trees))
}

fn predictor(
    state: &SingleState,
    k: u32,
    extended_grammar: &ExtendedGrammar,
    state_sets: &mut Vec<StateSet>,
) -> Result<(), Error> {
    // println!("predict!");
    let new_state = State::Combined(CombinedState {
        typecode: state
            .next_token(extended_grammar)
            .ok_or(Error::InternalLogicError)?
            .symbol_i,
        start_i: k,
    });

    let current_set = state_sets
        .get_mut(k as usize)
        .ok_or(Error::InternalLogicError)?;

    current_set.insert(new_state);

    Ok(())
}

fn scanner(
    state: &SingleState,
    k: u32,
    expression: &Vec<InputSymbol>,
    extended_grammar: &ExtendedGrammar,
    state_sets: &mut Vec<StateSet>,
) -> Result<(), Error> {
    // println!("scan!");
    if state.start_i < expression.len() as u32 {
        if let Some(InputSymbol::Symbol(symbol)) = expression.get(k as usize) {
            if symbol.symbol_i
                == state
                    .next_token(extended_grammar)
                    .ok_or(Error::InternalLogicError)?
                    .symbol_i
            {
                let next_set = state_sets
                    .get_mut((k + 1) as usize)
                    .ok_or(Error::InternalLogicError)?;

                let new_state = State::Single(SingleState {
                    rule_i: state.rule_i,
                    processed_i: state.processed_i + 1,
                    start_i: state.start_i,
                    parse_trees: state.parse_trees.clone(),
                    // parse_trees: Vec::new(),
                });

                next_set.insert(new_state);
            }
        }
    }

    Ok(())
}

fn completer(
    state: &SingleState,
    k: u32,
    extended_grammar: &ExtendedGrammar,
    state_sets: &mut Vec<StateSet>,
) -> Result<(), Error> {
    // println!("complete!");
    let mut i = 0;
    while let Some(other_state) = state_sets
        .get(state.start_i as usize)
        .ok_or(Error::InternalLogicError)?
        .get_processed(i)
    {
        match other_state {
            State::Single(other_single_state) => {
                if let Some(other_state_symbol) = other_single_state.next_token(extended_grammar) {
                    if state.rule(extended_grammar).left_side.symbol_i
                        == other_state_symbol.symbol_i
                    {
                        let mut new_parse_trees = other_single_state.parse_trees.clone();
                        if state.rule_i < 0 {
                            return Err(Error::InternalLogicError);
                        }
                        if state
                            .rule(extended_grammar)
                            .right_side
                            .first()
                            .ok_or(Error::InternalLogicError)?
                            .symbol_i
                            != 0
                        {
                            new_parse_trees.push(ParseTreeNode::Node {
                                rule_i: state.rule_i as u32,
                                sub_nodes: state.parse_trees.clone(),
                            });
                        } else {
                            // Special case where the state has a work variable rule:
                            // Instead of creating a new Node with that rule, clone the work variable being caried by the state
                            new_parse_trees.push(
                                state
                                    .parse_trees
                                    .first()
                                    .ok_or(Error::InternalLogicError)?
                                    .clone(),
                            );
                        }

                        let new_state = State::Single(SingleState {
                            rule_i: other_single_state.rule_i,
                            processed_i: other_single_state.processed_i + 1,
                            start_i: other_single_state.start_i,
                            parse_trees: new_parse_trees,
                            // parse_trees: Vec::new(),
                        });

                        // println!("{:?}", new_state.proof);

                        let current_set = state_sets
                            .get_mut(k as usize)
                            .ok_or(Error::InternalLogicError)?;

                        current_set.insert(new_state);
                    }
                }
            }
            State::Combined(other_combined_state) => {
                let mut new_states = Vec::new();

                if state.rule_i != -1 {
                    for &rule_i in extended_grammar
                        .grammar
                        .earley_optimized_data
                        .completer_rules
                        .get(other_combined_state.typecode as usize - 1)
                        .ok_or(Error::InternalLogicError)?
                        .get(state.rule(extended_grammar).left_side.symbol_i as usize - 1)
                        .ok_or(Error::InternalLogicError)?
                    {
                        let new_parse_trees = if state
                            .rule(extended_grammar)
                            .right_side
                            .first()
                            .ok_or(Error::InternalLogicError)?
                            .symbol_i
                            != 0
                        {
                            vec![ParseTreeNode::Node {
                                rule_i: state.rule_i as u32,
                                sub_nodes: state.parse_trees.clone(),
                            }]
                        } else {
                            // Special case where the state has a work variable rule:
                            // Instead of creating a new Node with that rule, clone the work variable being caried by the state
                            vec![state
                                .parse_trees
                                .first()
                                .ok_or(Error::InternalLogicError)?
                                .clone()]
                        };

                        new_states.push(State::Single(SingleState {
                            rule_i: rule_i as i32,
                            processed_i: 1,
                            start_i: other_combined_state.start_i,
                            parse_trees: new_parse_trees,
                        }));
                    }
                }

                let current_set = state_sets
                    .get_mut(k as usize)
                    .ok_or(Error::InternalLogicError)?;

                for new_state in new_states {
                    current_set.insert(new_state);
                }
            }
        }
        i += 1;
    }

    Ok(())
}
