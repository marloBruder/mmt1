use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

use crate::{
    editor::unify::LocateAfterRef,
    metamath::export::{write_text_wrapped, write_text_wrapped_no_whitespace},
    util::{
        self,
        earley_parser_optimized::{self, EarleyOptimizedData, Grammar, GrammarRule},
        header_iterators::{
            ConstantIterator, FloatingHypothesisIterator, HeaderIterator,
            HeaderLocateAfterIterator, TheoremIterator, VariableIterator,
        },
    },
    Error,
};
use Statement::*;

#[derive(Debug, Default)]
pub struct MetamathData {
    pub database_header: Header,
    pub html_representations: Vec<HtmlRepresentation>,
    pub optimized_data: OptimizedMetamathData,
    pub database_path: String,
}

#[derive(Debug, Default)]
pub struct OptimizedMetamathData {
    pub variables: HashSet<String>,
    pub floating_hypotheses: Vec<FloatingHypothesis>,
    pub theorem_amount: u32,
    pub theorem_data: HashMap<String, OptimizedTheoremData>,
    pub symbol_number_mapping: SymbolNumberMapping,
    pub grammar: Grammar,
}

#[derive(Debug)]
pub struct OptimizedTheoremData {
    pub hypotheses_parsed: Vec<ParseTree>,
    pub assertion_parsed: ParseTree,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParseTree {
    pub nodes: Vec<ParseTree>,
    pub rule: u32,
}

#[derive(Debug, Default)]
pub struct SymbolNumberMapping {
    pub symbols: HashMap<u32, String>,
    pub numbers: HashMap<String, u32>,
    pub variable_typecodes: HashMap<u32, u32>,
    pub typecode_count: u32,
    pub variable_count: u32,
    pub constant_count: u32,
}

#[derive(Debug)]
pub enum Statement {
    CommentStatement(Comment),
    ConstantStatement(Vec<Constant>),
    VariableStatement(Vec<Variable>),
    FloatingHypohesisStatement(FloatingHypothesis),
    TheoremStatement(Theorem),
}

pub enum DatabaseElement<'a> {
    Header(&'a Header, u32),
    Statement(&'a Statement),
}

#[derive(Debug, Clone, Serialize)]
pub struct Comment {
    pub text: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct Constant {
    pub symbol: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct Variable {
    pub symbol: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct FloatingHypothesis {
    pub label: String,
    pub typecode: String,
    pub variable: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct Theorem {
    pub label: String,
    pub description: String,
    pub distincts: Vec<String>,
    pub hypotheses: Vec<Hypothesis>,
    pub assertion: String,
    pub proof: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Hypothesis {
    pub label: String,
    pub expression: String,
}

#[derive(Debug, Default)]
pub struct Header {
    pub title: String,
    pub content: Vec<Statement>,
    pub subheaders: Vec<Header>,
}

pub struct HeaderRepresentation {
    pub title: String,
    pub content_titles: Vec<HeaderContentRepresentation>,
    pub subheader_titles: Vec<String>,
}

pub struct HeaderContentRepresentation {
    //Should only ever be "CommentStatement" or "ConstantStatement" or "VariableStatement" or "FloatingHypohesisStatement" or "TheoremStatement";
    pub content_type: String,
    pub title: String,
}

#[derive(Serialize, Deserialize)]
pub struct HeaderPath {
    pub path: Vec<usize>,
}

pub struct TheoremPath {
    pub header_path: HeaderPath,
    pub theorem_index: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct HtmlRepresentation {
    pub symbol: String,
    pub html: String,
}

pub enum DatabaseElementPageData {
    Theorem(TheoremPageData),
    FloatingHypothesis(FloatingHypothesisPageData),
}

pub struct TheoremPageData {
    pub theorem: Theorem,
    pub theorem_number: u32,
    pub proof_lines: Vec<ProofLine>,
    pub last_theorem_label: Option<String>,
    pub next_theorem_label: Option<String>,
}

pub struct FloatingHypothesisPageData {
    pub floating_hypothesis: FloatingHypothesis,
}

#[derive(Serialize)]
pub struct ProofLine {
    pub hypotheses: Vec<i32>,
    pub reference: String,
    pub indention: i32,
    pub assertion: String,
}

pub struct TheoremListData {
    pub list: Vec<ListEntry>,
    pub page_amount: u32,
}

pub enum ListEntry {
    Header(HeaderListEntry),
    Comment(CommentListEntry),
    Constant(ConstantListEntry),
    Variable(VariableListEntry),
    FloatingHypohesis(FloatingHypothesisListEntry),
    Theorem(TheoremListEntry),
}

pub struct HeaderListEntry {
    pub header_path: String,
    pub title: String,
}

pub struct CommentListEntry {
    pub comment_path: String,
    pub text: String,
}

pub struct ConstantListEntry {
    pub constants: String,
}

pub struct VariableListEntry {
    pub variables: String,
}

pub struct FloatingHypothesisListEntry {
    pub label: String,
    pub typecode: String,
    pub variable: String,
}

pub struct TheoremListEntry {
    pub label: String,
    pub theorem_number: u32,
    pub hypotheses: Vec<String>,
    pub assertion: String,
    pub description: String,
}

impl MetamathData {
    pub fn valid_new_symbols(&self, symbols: &Vec<&str>) -> bool {
        self.database_header
            .iter()
            .find(|c| match c {
                DatabaseElement::Statement(s) => match s {
                    Statement::CommentStatement(_) => false,
                    Statement::ConstantStatement(consts) => {
                        for c in consts {
                            for symbol in symbols {
                                if &c.symbol == symbol {
                                    return true;
                                }
                            }
                        }
                        false
                    }
                    Statement::VariableStatement(vars) => {
                        for v in vars {
                            for symbol in symbols {
                                if &v.symbol == symbol {
                                    return true;
                                }
                            }
                        }
                        false
                    }
                    Statement::FloatingHypohesisStatement(fh) => {
                        for symbol in symbols {
                            if &fh.label == symbol {
                                return true;
                            }
                        }
                        false
                    }
                    Statement::TheoremStatement(t) => {
                        for symbol in symbols {
                            if &t.label == symbol {
                                return true;
                            }
                        }
                        false
                    }
                },
                DatabaseElement::Header(_, _) => false,
            })
            .is_none()
    }

    pub fn calc_optimized_theorem_data(&mut self) -> Result<(), Error> {
        let mut i = 0;
        for theorem in self.database_header.theorem_iter() {
            // if i % 100 == 0 {
            println!("{}", i);
            // }
            i += 1;
            if theorem.assertion.starts_with("|- ") {
                self.optimized_data.theorem_data.insert(
                    theorem.label.to_string(),
                    theorem.calc_optimized_data(self)?,
                );
            }
        }

        Ok(())
    }

    pub fn recalc_optimized_floating_hypotheses_after_one_new(&mut self) -> Result<(), Error> {
        let mut i: usize = 0;
        for floating_hypothesis in self.database_header.floating_hypohesis_iter() {
            let optimized_floating_hypothesis_option =
                self.optimized_data.floating_hypotheses.get(i);

            match optimized_floating_hypothesis_option {
                Some(optimized_floating_hypothesis) => {
                    if floating_hypothesis.label != optimized_floating_hypothesis.label {
                        self.optimized_data
                            .floating_hypotheses
                            .insert(i, floating_hypothesis.clone());
                        return Ok(());
                    }
                }
                None => {
                    // Happens when the new floating hypothesis was inserted at the end
                    self.optimized_data
                        .floating_hypotheses
                        .push(floating_hypothesis.clone());
                    return Ok(());
                }
            }

            i += 1;
        }

        Ok(())
    }

    pub fn recalc_symbol_number_mapping_and_grammar(&mut self) -> Result<(), Error> {
        self.optimized_data.symbol_number_mapping =
            SymbolNumberMapping::calc_mapping(&self.database_header);

        Grammar::calc_grammar(self)?;
        // let mut i: u32 = 1;
        // while let Some(symbol) = self.optimized_data.symbol_number_mapping.symbols.get(&i) {
        //     println!("{}: {}", i, symbol);
        //     if i == self.optimized_data.symbol_number_mapping.typecode_count
        //         || i == self.optimized_data.symbol_number_mapping.typecode_count
        //             + self.optimized_data.symbol_number_mapping.variable_count
        //     {
        //         println!("");
        //     }
        //     i += 1;
        // }
        // for grammar_rule in &self.optimized_data.grammar.rules {
        //     println!("{:?}", grammar_rule);
        // }
        Ok(())
    }
}

impl Statement {
    pub fn write_mm_string(&self, target: &mut String) {
        match self {
            Self::CommentStatement(comment) => {
                target.push_str("$(");
                write_text_wrapped(target, &comment.text, "   ");
                write_text_wrapped(target, "$)", "   ");
            }
            Self::ConstantStatement(constants) => {
                target.push_str("  $c");
                for constant in constants {
                    write_text_wrapped(target, &constant.symbol, "   ");
                }
                write_text_wrapped(target, "$.", "   ");
            }
            Self::VariableStatement(variables) => {
                target.push_str("  $v");
                for variable in variables {
                    write_text_wrapped(target, &variable.symbol, "   ");
                }
                write_text_wrapped(target, "$.", "   ");
            }
            Self::FloatingHypohesisStatement(floating_hypothesis) => {
                target.push_str("  ");
                target.push_str(&floating_hypothesis.label);
                write_text_wrapped(target, "$f", "   ");
                write_text_wrapped(target, &floating_hypothesis.typecode, "   ");
                write_text_wrapped(target, &floating_hypothesis.variable, "   ");
                write_text_wrapped(target, "$.", "   ");
            }
            Self::TheoremStatement(theorem) => {
                let scoped = !(theorem.distincts.is_empty() && theorem.hypotheses.is_empty());
                let scoped_offset = if scoped { 2 } else { 0 };

                if scoped {
                    target.push_str("  ${\n");
                }

                for dist_vars in &theorem.distincts {
                    target.push_str("    $d");
                    write_text_wrapped(target, dist_vars, "       ");
                    write_text_wrapped(target, "$.", "       ");
                    target.push('\n');
                }

                for hyp in &theorem.hypotheses {
                    target.push_str("    ");
                    target.push_str(&hyp.label);
                    write_text_wrapped(target, "$e", "       ");
                    write_text_wrapped(target, &hyp.expression, "       ");
                    write_text_wrapped(target, "$.", "       ");
                    target.push('\n');
                }

                if !theorem.description.is_empty() {
                    target.push_str(util::spaces(scoped_offset + 2));
                    target.push_str("$(");
                    write_text_wrapped(
                        target,
                        &theorem.description,
                        util::spaces(scoped_offset + 5),
                    );
                    write_text_wrapped(target, "$)", util::spaces(scoped_offset + 5));
                    target.push('\n');
                }

                target.push_str(util::spaces(scoped_offset + 2));
                target.push_str(&theorem.label);
                match &theorem.proof {
                    None => {
                        write_text_wrapped(target, "$a", util::spaces(scoped_offset + 4));
                        write_text_wrapped(
                            target,
                            &theorem.assertion,
                            util::spaces(scoped_offset + 4),
                        );
                        write_text_wrapped(target, "$.", util::spaces(scoped_offset + 4));
                    }
                    Some(proof) => {
                        write_text_wrapped(target, "$p", util::spaces(scoped_offset + 4));
                        write_text_wrapped(
                            target,
                            &theorem.assertion,
                            util::spaces(scoped_offset + 4),
                        );
                        write_text_wrapped(target, "$=", util::spaces(scoped_offset + 4));
                        target.push('\n');
                        target.push_str(util::spaces(scoped_offset + 3));
                        if proof.starts_with('(') {
                            // should always be the case
                            if let Some((labels, steps)) = proof.split_once(')') {
                                write_text_wrapped(target, labels, util::spaces(scoped_offset + 4));
                                write_text_wrapped(target, ")", util::spaces(scoped_offset + 4));
                                target.push(' ');
                                write_text_wrapped_no_whitespace(
                                    target,
                                    steps,
                                    util::spaces(scoped_offset + 4),
                                );
                            }
                        } else {
                            write_text_wrapped(target, proof, util::spaces(scoped_offset + 4));
                        }
                        write_text_wrapped(target, "$.", util::spaces(scoped_offset + 4));
                    }
                }

                if scoped {
                    target.push_str("\n  $}");
                }
            }
        }
    }

    pub fn insert_mm_string(&self, target: &mut String, insert_pos: usize) {
        let mut mm_string = String::new();

        self.write_mm_string(&mut mm_string);

        target.insert_str(insert_pos, &mm_string);
    }

    //     pub fn is_variable(&self) -> bool {
    //         match self {
    //             VariableStatement(_) => true,
    //             _ => false,
    //         }
    //     }

    //     pub fn is_costant(&self) -> bool {
    //         match self {
    //             ConstantStatement(_) => true,
    //             _ => false,
    //         }
    //     }

    //     pub fn is_floating_hypothesis(&self) -> bool {
    //         match self {
    //             FloatingHypohesisStatement(_) => true,
    //             _ => false,
    //         }
    //     }

    //     pub fn is_theorem(&self) -> bool {
    //         match self {
    //             TheoremStatement(_) => true,
    //             _ => false,
    //         }
    //     }
}

impl ParseTree {
    pub fn calc_proof(&self, grammar: &Grammar) -> Result<String, Error> {
        let mut proof = String::new();

        let mut trees = vec![(self, 0)];

        while let Some((tree, next_node_i)) = trees.last_mut() {
            if let Some(&node_i) = grammar
                .rules
                .get(tree.rule as usize)
                .ok_or(Error::InternalLogicError)?
                .var_order
                .get(*next_node_i as usize)
            {
                let node = tree
                    .nodes
                    .get(node_i as usize)
                    .ok_or(Error::InternalLogicError)?;

                *next_node_i += 1;
                trees.push((node, 0));
            } else {
                proof.push_str(
                    &grammar
                        .rules
                        .get(tree.rule as usize)
                        .ok_or(Error::InternalLogicError)?
                        .label,
                );
                proof.push(' ');
                trees.pop();
            }
        }

        proof.pop();

        Ok(proof)
    }

    pub fn are_substitutions(
        trees: &Vec<&ParseTree>,
        others: &Vec<&ParseTree>,
        grammar: &Grammar,
    ) -> Result<bool, Error> {
        if trees.len() != others.len() {
            return Ok(false);
        }

        let mut substitutions: HashMap<u32, &ParseTree> = HashMap::new();

        let mut check: Vec<(&ParseTree, &ParseTree)> = trees
            .iter()
            .zip(others.iter())
            .map(|(t, o)| (*t, *o))
            .collect();

        while let Some((subtree, other_subtree)) = check.pop() {
            let subtree_rule = grammar
                .rules
                .get(subtree.rule as usize)
                .ok_or(Error::InternalLogicError)?;
            let other_subtree_rule = grammar
                .rules
                .get(other_subtree.rule as usize)
                .ok_or(Error::InternalLogicError)?;

            if subtree_rule.is_floating_hypothesis {
                match substitutions.get(&subtree.rule) {
                    Some(&sub) => {
                        if sub != other_subtree {
                            return Ok(false);
                        }
                    }
                    None => {
                        if subtree_rule.left_side == other_subtree_rule.left_side {
                            substitutions.insert(subtree.rule, other_subtree);
                        } else {
                            return Ok(false);
                        }
                    }
                }
            } else {
                if subtree.rule != other_subtree.rule
                    || subtree.nodes.len() != other_subtree.nodes.len()
                {
                    return Ok(false);
                }
                for (node, other_node) in subtree.nodes.iter().zip(other_subtree.nodes.iter()) {
                    check.push((node, other_node));
                }
            }
        }

        Ok(true)
    }
}

impl SymbolNumberMapping {
    pub fn calc_mapping(header: &Header) -> SymbolNumberMapping {
        let mut symbols: HashMap<u32, String> = HashMap::new();
        let mut numbers: HashMap<String, u32> = HashMap::new();
        let mut variable_typecodes: HashMap<u32, u32> = HashMap::new();
        let mut next_i: u32 = 1;
        let mut typecodes: Vec<&str> = Vec::new();

        for fh in header.floating_hypohesis_iter() {
            if !typecodes.contains(&&*fh.typecode) {
                typecodes.push(&fh.typecode);
                let mut typecode_string = "$".to_string();
                typecode_string.push_str(&fh.typecode);
                symbols.insert(next_i, typecode_string.clone());
                numbers.insert(typecode_string, next_i);
                next_i += 1;
            }
        }

        let typecode_count = next_i - 1;

        for var in header.variable_iter() {
            symbols.insert(next_i, var.symbol.to_string());
            numbers.insert(var.symbol.to_string(), next_i);
            next_i += 1;
        }

        let variable_count = next_i - typecode_count - 1;

        for constant in header.constant_iter() {
            symbols.insert(next_i, constant.symbol.to_string());
            numbers.insert(constant.symbol.to_string(), next_i);
            next_i += 1;
        }

        let constant_count = next_i - typecode_count - variable_count - 1;

        for fh in header.floating_hypohesis_iter() {
            if let Some(num) = numbers.get(&fh.variable) {
                let mut typecode_string = "$".to_string();
                typecode_string.push_str(&fh.typecode);
                variable_typecodes.insert(*num, *numbers.get(&typecode_string).unwrap());
            }
        }

        SymbolNumberMapping {
            symbols,
            numbers,
            variable_typecodes,
            typecode_count,
            variable_count,
            constant_count,
        }
    }

    pub fn expression_to_number_vec(&self, expression: &str) -> Result<Vec<u32>, ()> {
        let mut expression_vec: Vec<u32> = Vec::new();

        for token in expression.split_ascii_whitespace() {
            expression_vec.push(*self.numbers.get(token).ok_or(())?);
        }

        Ok(expression_vec)
    }

    pub fn expression_to_number_vec_replace_variables_with_typecodes(
        &self,
        expression: &str,
    ) -> Result<(Vec<u32>, Vec<u32>), Error> {
        let mut variables: Vec<u32> = Vec::new();
        Ok((
            expression
                .split_ascii_whitespace()
                .map(|t| {
                    let mut num = *self.numbers.get(t).ok_or(Error::InactiveMathSymbolError)?;
                    if self.is_variable(num) {
                        variables.push(num);
                        num = *self
                            .variable_typecodes
                            .get(&num)
                            .ok_or(Error::VariableWithoutTypecode)?;
                    }
                    Ok(num)
                })
                .collect::<Result<Vec<u32>, Error>>()?,
            variables,
        ))
    }

    pub fn expression_to_number_vec_skip_first(&self, expression: &str) -> Result<Vec<u32>, Error> {
        if expression.split_ascii_whitespace().next().is_none() {
            return Err(Error::MissingExpressionError);
        }

        expression
            .split_ascii_whitespace()
            .skip(1)
            .map(|t| {
                Ok(*self
                    .numbers
                    .get(t)
                    .ok_or(Error::NonSymbolInExpressionError)?)
            })
            .collect::<Result<Vec<u32>, Error>>()
    }

    pub fn expression_to_parse_tree(
        &self,
        expression: &str,
        grammar: &Grammar,
    ) -> Result<ParseTree, Error> {
        let expression = self.expression_to_number_vec_skip_first(expression)?;

        let expression_parse_tree =
            earley_parser_optimized::earley_parse(grammar, &expression, vec![1], self)?
                .ok_or(Error::ExpressionParseError)?
                .into_iter()
                .next()
                .ok_or(Error::InternalLogicError)?;

        Ok(expression_parse_tree)
    }

    pub fn is_typecode(&self, number: u32) -> bool {
        return number <= self.typecode_count;
    }

    pub fn is_variable(&self, number: u32) -> bool {
        return self.typecode_count < number && number <= self.typecode_count + self.variable_count;
    }

    pub fn is_constant(&self, number: u32) -> bool {
        return self.typecode_count + self.variable_count < number;
    }
}

impl Grammar {
    pub fn calc_grammar(metamath_data: &mut MetamathData) -> Result<(), Error> {
        metamath_data.optimized_data.grammar = Grammar {
            rules: Vec::new(),
            earley_optimized_data: EarleyOptimizedData::default(),
        };

        let mut i = 0;

        let symbol_number_mapping = &metamath_data.optimized_data.symbol_number_mapping;

        for floating_hypothesis in &metamath_data.optimized_data.floating_hypotheses {
            metamath_data
                .optimized_data
                .grammar
                .rules
                .push(GrammarRule {
                    left_side: *symbol_number_mapping
                        .numbers
                        .get(&format!("${}", floating_hypothesis.typecode))
                        .ok_or(Error::InternalLogicError)?,
                    right_side: vec![*symbol_number_mapping
                        .numbers
                        .get(&floating_hypothesis.variable)
                        .ok_or(Error::InternalLogicError)?],
                    label: floating_hypothesis.label.clone(),
                    var_order: Vec::new(),
                    is_floating_hypothesis: true,
                });
        }

        metamath_data
            .optimized_data
            .grammar
            .recalc_earley_optimized_data(symbol_number_mapping)?;

        for theorem in metamath_data.database_header.theorem_iter() {
            if theorem.proof == None
                && theorem
                    .assertion
                    .split_ascii_whitespace()
                    .next()
                    .ok_or(Error::InternalLogicError)?
                    != "|-"
                && theorem.hypotheses.len() == 0
            {
                let mut assertion_token_iter = theorem.assertion.split_ascii_whitespace();
                let left_side = *symbol_number_mapping
                    .numbers
                    .get(&format!("${}", assertion_token_iter.next().unwrap()))
                    .ok_or(Error::InternalLogicError)?;

                let mut vars: Vec<u32> = Vec::new();

                let right_side = assertion_token_iter
                    .map(|t| {
                        let mut num = *symbol_number_mapping
                            .numbers
                            .get(t)
                            .ok_or(Error::InternalLogicError)?;
                        if symbol_number_mapping.is_variable(num) {
                            vars.push(num);
                            num = *symbol_number_mapping
                                .variable_typecodes
                                .get(&num)
                                .ok_or(Error::InternalLogicError)?;
                        }
                        Ok(num)
                    })
                    .collect::<Result<Vec<u32>, Error>>()?;

                let mut var_order: Vec<u32> = Vec::new();

                for floating_hypothesis in &metamath_data.optimized_data.floating_hypotheses {
                    for (i, &var) in vars.iter().enumerate() {
                        if *symbol_number_mapping
                            .numbers
                            .get(&floating_hypothesis.variable)
                            .ok_or(Error::InternalLogicError)?
                            == var
                        {
                            var_order.push(i as u32);
                            break;
                        }
                    }
                }

                metamath_data
                    .optimized_data
                    .grammar
                    .rules
                    .push(GrammarRule {
                        left_side,
                        right_side,
                        label: theorem.label.clone(),
                        var_order,
                        is_floating_hypothesis: false,
                    });

                metamath_data
                    .optimized_data
                    .grammar
                    .recalc_earley_optimized_data(symbol_number_mapping)?;
            } else if theorem
                .assertion
                .split_ascii_whitespace()
                .next()
                .ok_or(Error::InternalLogicError)?
                == "|-"
            {
                i += 1;
                if i % 100 == 0 {
                    println!("{}:", i);
                }

                metamath_data.optimized_data.theorem_data.insert(
                    theorem.label.to_string(),
                    theorem.calc_optimized_data(metamath_data)?,
                );
            }
        }

        Ok(())
    }

    fn recalc_earley_optimized_data(
        &mut self,
        symbol_number_mapping: &SymbolNumberMapping,
    ) -> Result<(), Error> {
        let mut completer_rules: Vec<Vec<Vec<usize>>> =
            vec![
                vec![Vec::new(); symbol_number_mapping.typecode_count as usize];
                symbol_number_mapping.typecode_count as usize
            ];

        let mut combined_states_to_add: Vec<Vec<u32>> =
            vec![Vec::new(); symbol_number_mapping.typecode_count as usize];

        let mut single_states_to_add: Vec<Vec<Vec<usize>>> = vec![
            vec![
                Vec::new();
                (symbol_number_mapping.variable_count + symbol_number_mapping.constant_count)
                    as usize
            ];
            symbol_number_mapping.typecode_count
                as usize
        ];

        for (rule_i, rule) in self.rules.iter().enumerate() {
            let right_side_first = *rule.right_side.first().ok_or(Error::InternalLogicError)?;
            if symbol_number_mapping.is_typecode(right_side_first) {
                completer_rules
                    .get_mut(rule.left_side as usize - 1)
                    .ok_or(Error::InternalLogicError)?
                    .get_mut(right_side_first as usize - 1)
                    .ok_or(Error::InternalLogicError)?
                    .push(rule_i);

                if !combined_states_to_add
                    .get(rule.left_side as usize - 1)
                    .ok_or(Error::InternalLogicError)?
                    .contains(&right_side_first)
                {
                    combined_states_to_add
                        .get_mut(rule.left_side as usize - 1)
                        .ok_or(Error::InternalLogicError)?
                        .push(right_side_first);
                }
            } else {
                single_states_to_add
                    .get_mut(rule.left_side as usize - 1)
                    .ok_or(Error::InternalLogicError)?
                    .get_mut((right_side_first - symbol_number_mapping.typecode_count - 1) as usize)
                    .ok_or(Error::InternalLogicError)?
                    .push(rule_i);
            }
        }

        self.earley_optimized_data = EarleyOptimizedData {
            completer_rules,
            combined_states_to_add,
            single_states_to_add,
        };

        Ok(())
    }
}

impl FloatingHypothesis {
    pub fn to_assertions_string(&self) -> String {
        format!("{} {}", self.typecode, self.variable)
    }
}

impl Theorem {
    pub fn to_theorem_list_entry(&self, theorem_number: u32) -> TheoremListEntry {
        TheoremListEntry {
            label: self.label.clone(),
            theorem_number,
            hypotheses: self
                .hypotheses
                .iter()
                .map(|hypothesis| hypothesis.expression.clone())
                .collect(),
            assertion: self.assertion.clone(),
            description: self.description.clone(),
        }
    }

    pub fn calc_optimized_data(
        &self,
        metamath_data: &MetamathData,
    ) -> Result<OptimizedTheoremData, Error> {
        let hypotheses_parsed = self
            .hypotheses
            .iter()
            .map(|h| {
                earley_parser_optimized::earley_parse(
                    &metamath_data.optimized_data.grammar,
                    &metamath_data
                        .optimized_data
                        .symbol_number_mapping
                        .expression_to_number_vec_skip_first(&h.expression)
                        .or(Err(Error::InternalLogicError))?,
                    vec![1],
                    &metamath_data.optimized_data.symbol_number_mapping,
                )?
                .ok_or(Error::ExpressionParseError)?
                .into_iter()
                .next()
                .ok_or(Error::InternalLogicError)
            })
            .collect::<Result<Vec<ParseTree>, Error>>()?;

        let assertion_parsed = earley_parser_optimized::earley_parse(
            &metamath_data.optimized_data.grammar,
            &metamath_data
                .optimized_data
                .symbol_number_mapping
                .expression_to_number_vec_skip_first(&self.assertion)
                .or(Err(Error::InternalLogicError))?,
            vec![1],
            &metamath_data.optimized_data.symbol_number_mapping,
        )?
        .ok_or(Error::ExpressionParseError)?
        .into_iter()
        .next()
        .ok_or(Error::InternalLogicError)?;

        // for hyp in &hypotheses_parsed {
        //     println!("{:?}", hyp.calc_proof(&grammar));
        // }
        // println!("{:?}", assertion_parsed.calc_proof(&grammar));

        Ok(OptimizedTheoremData {
            hypotheses_parsed,
            assertion_parsed,
        })
    }
}

impl Header {
    pub fn to_representation(&self) -> HeaderRepresentation {
        HeaderRepresentation {
            title: self.title.clone(),
            content_titles: self
                .content
                .iter()
                .map(|t| match t {
                    CommentStatement(_) => HeaderContentRepresentation {
                        content_type: "CommentStatement".to_string(),
                        title: "Comment".to_string(),
                    },
                    ConstantStatement(constants) => HeaderContentRepresentation {
                        content_type: "ConstantStatement".to_string(),
                        title: constants
                            .iter()
                            .fold((true, String::new()), |(first, mut s), c| {
                                if !first {
                                    s.push(' ');
                                }
                                s.push_str(&c.symbol);
                                (false, s)
                            })
                            .1,
                    },
                    VariableStatement(variables) => HeaderContentRepresentation {
                        content_type: "VariableStatement".to_string(),
                        title: variables
                            .iter()
                            .fold((true, String::new()), |(first, mut s), v| {
                                if !first {
                                    s.push(' ');
                                }
                                s.push_str(&v.symbol);
                                (false, s)
                            })
                            .1,
                    },
                    FloatingHypohesisStatement(floating_hypohesis) => HeaderContentRepresentation {
                        content_type: "FloatingHypothesisStatement".to_string(),
                        title: floating_hypohesis.label.clone(),
                    },
                    TheoremStatement(theorem) => HeaderContentRepresentation {
                        content_type: "TheoremStatement".to_string(),
                        title: theorem.label.clone(),
                    },
                })
                .collect(),
            subheader_titles: self.subheaders.iter().map(|sh| sh.title.clone()).collect(),
        }
    }

    pub fn find_theorem_by_label(&self, label: &str) -> Option<&Theorem> {
        self.theorem_iter().find(|t| t.label == label)

        // for theorem in &self.theorems {
        //     if theorem.name == name {
        //         return Some(theorem);
        //     }
        // }

        // for sub_header in &self.sub_headers {
        //     let sub_header_res = sub_header.find_theorem_by_name(name);
        //     if sub_header_res.is_some() {
        //         return sub_header_res;
        //     }
        // }

        // None
    }

    pub fn calc_theorem_path_by_label(&self, label: &str) -> Option<TheoremPath> {
        for (index, statement) in self.content.iter().enumerate() {
            if let TheoremStatement(theorem) = statement {
                if theorem.label == label {
                    return Some(TheoremPath {
                        header_path: HeaderPath { path: Vec::new() },
                        theorem_index: index,
                    });
                }
            }
        }

        for (index, sub_header) in self.subheaders.iter().enumerate() {
            let sub_header_res = sub_header.calc_theorem_path_by_label(label);
            if let Some(mut theorem_path) = sub_header_res {
                theorem_path.header_path.path.insert(0, index);
                return Some(theorem_path);
            }
        }

        None
    }

    pub fn calc_header_path_by_title(&self, title: &str) -> Option<HeaderPath> {
        if self.title == title {
            return Some(HeaderPath { path: Vec::new() });
        }

        for (index, sub_header) in self.subheaders.iter().enumerate() {
            let sub_header_res = sub_header.calc_header_path_by_title(title);
            if let Some(mut header_path) = sub_header_res {
                header_path.path.insert(0, index);
                return Some(header_path);
            }
        }

        None
    }

    // pub fn count_theorems_and_headers(&self) -> i32 {
    //     let mut sum = 1 + self.theorems.len() as i32;
    //     for sub_header in &self.sub_headers {
    //         sum += sub_header.count_theorems_and_headers();
    //     }
    //     sum
    // }

    pub fn iter(&self) -> HeaderIterator {
        HeaderIterator::new(self)
    }

    pub fn constant_iter(&self) -> ConstantIterator {
        ConstantIterator::new(self)
    }

    pub fn variable_iter(&self) -> VariableIterator {
        VariableIterator::new(self)
    }

    pub fn floating_hypohesis_iter(&self) -> FloatingHypothesisIterator {
        FloatingHypothesisIterator::new(self)
    }

    pub fn theorem_iter(&self) -> TheoremIterator {
        TheoremIterator::new(self)
    }

    pub fn locate_after_iter<'a, 'b>(
        &'a self,
        locate_after: LocateAfterRef<'b>,
    ) -> HeaderLocateAfterIterator<'a, 'b> {
        HeaderLocateAfterIterator::new(self, locate_after)
    }
}

impl HeaderPath {
    pub fn from_str(str: &str) -> Option<HeaderPath> {
        if str.contains('+') {
            return None;
        }

        Some(HeaderPath {
            path: str
                .split('.')
                .map(|s| {
                    let i = s.parse::<usize>().ok()?;
                    if i == 0 {
                        return None;
                    }
                    Some(i - 1)
                })
                .collect::<Option<Vec<usize>>>()?,
        })
    }

    pub fn to_string(&self) -> String {
        self.path
            .iter()
            .fold((true, String::new()), |(first, mut s), t| {
                if !first {
                    s.push('.');
                }
                s.push_str(&(*t + 1).to_string());
                (false, s)
            })
            .1
    }

    pub fn resolve<'a>(&self, top_header: &'a Header) -> Option<&'a Header> {
        let mut header = top_header;

        for &index in &self.path {
            header = header.subheaders.get(index)?;
        }

        Some(header)
    }

    pub fn resolve_mut<'a>(&self, top_header: &'a mut Header) -> Option<&'a mut Header> {
        let mut header = top_header;

        for &index in &self.path {
            header = header.subheaders.get_mut(index)?;
        }

        Some(header)
    }
}

impl Default for HeaderPath {
    fn default() -> Self {
        HeaderPath { path: Vec::new() }
    }
}

impl Default for TheoremPath {
    fn default() -> Self {
        TheoremPath {
            theorem_index: 0,
            header_path: HeaderPath::default(),
        }
    }
}

impl serde::Serialize for HeaderRepresentation {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        use serde::ser::SerializeStruct;

        let mut state = serializer.serialize_struct("HeaderRepresentation", 3)?;
        state.serialize_field("title", &self.title)?;
        state.serialize_field("contentTitles", &self.content_titles)?;
        state.serialize_field("subheaderTitles", &self.subheader_titles)?;
        state.end()
    }
}

impl serde::Serialize for HeaderContentRepresentation {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        use serde::ser::SerializeStruct;

        let mut state = serializer.serialize_struct("HeaderContentRepresentation", 2)?;
        state.serialize_field("contentType", &self.content_type)?;
        state.serialize_field("title", &self.title)?;
        state.end()
    }
}

impl serde::Serialize for TheoremPath {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        use serde::ser::SerializeStruct;

        let mut state = serializer.serialize_struct("TheoremPath", 2)?;
        state.serialize_field("headerPath", &self.header_path)?;
        state.serialize_field("theoremIndex", &self.theorem_index)?;
        state.end()
    }
}

impl serde::Serialize for DatabaseElementPageData {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        match self {
            Self::Theorem(theorem_page_data) => theorem_page_data.serialize(serializer),
            Self::FloatingHypothesis(floating_hypothesis_page_data) => {
                floating_hypothesis_page_data.serialize(serializer)
            }
        }
    }
}

impl serde::Serialize for TheoremPageData {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        use serde::ser::SerializeStruct;

        let mut state = serializer.serialize_struct("TheoremPageData", 5)?;
        state.serialize_field("theorem", &self.theorem)?;
        state.serialize_field("theoremNumber", &self.theorem_number)?;
        state.serialize_field("proofLines", &self.proof_lines)?;
        state.serialize_field("lastTheoremLabel", &self.last_theorem_label)?;
        state.serialize_field("nextTheoremLabel", &self.next_theorem_label)?;
        state.serialize_field("discriminator", "TheoremPageData")?;
        state.end()
    }
}

impl serde::Serialize for FloatingHypothesisPageData {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        use serde::ser::SerializeStruct;

        let mut state = serializer.serialize_struct("FloatingHypothesisPageData", 1)?;
        state.serialize_field("floatingHypothesis", &self.floating_hypothesis)?;
        state.serialize_field("discriminator", "FloatingHypothesisPageData")?;
        state.end()
    }
}

impl serde::Serialize for TheoremListData {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        use serde::ser::SerializeStruct;

        let mut state = serializer.serialize_struct("TheoremListData", 2)?;
        state.serialize_field("list", &self.list)?;
        state.serialize_field("pageAmount", &self.page_amount)?;
        state.end()
    }
}

impl serde::Serialize for ListEntry {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        use serde::ser::SerializeStruct;

        match *self {
            Self::Header(ref header_list_entry) => {
                let mut state = serializer.serialize_struct("HeaderListEntry", 2)?;
                state.serialize_field("headerPath", &header_list_entry.header_path)?;
                state.serialize_field("title", &header_list_entry.title)?;
                state.serialize_field("discriminator", "HeaderListEntry")?;
                state.end()
            }
            Self::Comment(ref comment_list_entry) => {
                let mut state = serializer.serialize_struct("CommentListEntry", 2)?;
                state.serialize_field("commentPath", &comment_list_entry.comment_path)?;
                state.serialize_field("text", &comment_list_entry.text)?;
                state.serialize_field("discriminator", "CommentListEntry")?;
                state.end()
            }
            Self::Constant(ref constant_list_entry) => {
                let mut state = serializer.serialize_struct("ConstantListEntry", 1)?;
                state.serialize_field("constants", &constant_list_entry.constants)?;
                state.serialize_field("discriminator", "ConstantListEntry")?;
                state.end()
            }
            Self::Variable(ref variable_list_entry) => {
                let mut state = serializer.serialize_struct("VariableListEntry", 1)?;
                state.serialize_field("variables", &variable_list_entry.variables)?;
                state.serialize_field("discriminator", "VariableListEntry")?;
                state.end()
            }
            Self::FloatingHypohesis(ref floating_hypothesis_list_entry) => {
                let mut state = serializer.serialize_struct("FloatingHypothesisListEntry", 3)?;
                state.serialize_field("label", &floating_hypothesis_list_entry.label)?;
                state.serialize_field("typecode", &floating_hypothesis_list_entry.typecode)?;
                state.serialize_field("variable", &floating_hypothesis_list_entry.variable)?;
                state.serialize_field("discriminator", "FloatingHypothesisListEntry")?;
                state.end()
            }
            Self::Theorem(ref theorem_list_entry) => {
                let mut state = serializer.serialize_struct("TheoremListEntry", 5)?;
                state.serialize_field("label", &theorem_list_entry.label)?;
                state.serialize_field("theoremNumber", &theorem_list_entry.theorem_number)?;
                state.serialize_field("hypotheses", &theorem_list_entry.hypotheses)?;
                state.serialize_field("assertion", &theorem_list_entry.assertion)?;
                state.serialize_field("description", &theorem_list_entry.description)?;
                state.serialize_field("discriminator", "TheoremListEntry")?;
                state.end()
            }
        }
    }
}

// impl serde::Serialize for MetamathData {
//     fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
//     where
//         S: serde::ser::Serializer,
//     {
//         use serde::ser::SerializeStruct;

//         let mut state = serializer.serialize_struct("MetamathData", 4)?;
//         state.serialize_field("constants", &self.constants)?;
//         state.serialize_field("variables", &self.variables)?;
//         state.serialize_field("floating_hypotheses", &self.floating_hypotheses)?;
//         state.serialize_field("theorems", &self.theorems)?;
//         state.serialize_field("in_progress_theorems", &self.in_progress_theorems)?;
//         state.end()
//     }
// }

// impl serde::Serialize for Constant {
//     fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
//     where
//         S: serde::ser::Serializer,
//     {
//         use serde::ser::SerializeStruct;

//         // 3 is the number of fields in the struct.
//         let mut state = serializer.serialize_struct("Constant", 1)?;
//         state.serialize_field("symbol", &self.symbol)?;
//         state.end()
//     }
// }

// impl serde::Serialize for Variable {
//     fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
//     where
//         S: serde::ser::Serializer,
//     {
//         use serde::ser::SerializeStruct;

//         // 3 is the number of fields in the struct.
//         let mut state = serializer.serialize_struct("Variable", 1)?;
//         state.serialize_field("symbol", &self.symbol)?;
//         state.end()
//     }
// }

// impl serde::Serialize for FloatingHypohesis {
//     fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
//     where
//         S: serde::ser::Serializer,
//     {
//         use serde::ser::SerializeStruct;

//         // 3 is the number of fields in the struct.
//         let mut state = serializer.serialize_struct("FloatingHypohesis", 3)?;
//         state.serialize_field("label", &self.label)?;
//         state.serialize_field("typecode", &self.typecode)?;
//         state.serialize_field("variable", &self.variable)?;
//         state.end()
//     }
// }

// impl serde::Serialize for Theorem {
//     fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
//     where
//         S: serde::ser::Serializer,
//     {
//         use serde::ser::SerializeStruct;

//         // 3 is the number of fields in the struct.
//         let mut state = serializer.serialize_struct("InProgressTheorem", 6)?;
//         state.serialize_field("name", &self.name)?;
//         state.serialize_field("description", &self.description)?;
//         state.serialize_field("disjoints", &self.disjoints)?;
//         state.serialize_field("hypotheses", &self.hypotheses)?;
//         state.serialize_field("assertion", &self.assertion)?;
//         state.serialize_field("proof", &self.proof)?;
//         state.end()
//     }
// }

// impl serde::Serialize for Hypothesis {
//     fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
//     where
//         S: serde::ser::Serializer,
//     {
//         use serde::ser::SerializeStruct;

//         // 3 is the number of fields in the struct.
//         let mut state = serializer.serialize_struct("InProgressTheorem", 2)?;
//         state.serialize_field("label", &self.label)?;
//         state.serialize_field("hypothesis", &self.hypothesis)?;
//         state.end()
//     }
// }

// impl serde::Serialize for InProgressTheorem {
//     fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
//     where
//         S: serde::ser::Serializer,
//     {
//         use serde::ser::SerializeStruct;

//         // 3 is the number of fields in the struct.
//         let mut state = serializer.serialize_struct("InProgressTheorem", 2)?;
//         state.serialize_field("name", &self.name)?;
//         state.serialize_field("text", &self.text)?;
//         state.end()
//     }
// }
