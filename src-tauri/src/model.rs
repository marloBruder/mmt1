use std::{
    collections::{HashMap, HashSet},
    fs::File,
    path::PathBuf,
    sync::Arc,
};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};

use crate::{
    metamath::{
        export::{write_text_wrapped, write_text_wrapped_no_whitespace},
        mmp_parser::LocateAfterRef,
    },
    util::{
        self, description_parser,
        earley_parser_optimized::{
            self, EarleyOptimizedData, Grammar, GrammarRule, InputSymbol, Symbol, WorkVariable,
        },
        header_iterators::{
            ConstantIterator, ConstantLocateAfterIterator, FloatingHypothesisIterator,
            HeaderIterator, HeaderLocateAfterIterator, TheoremIterator, TheoremLocateAfterIterator,
            VariableIterator, VariableLocateAfterIterator,
        },
    },
    Error, Settings,
};
use Statement::*;

#[derive(Debug, Default)]
pub struct MetamathData {
    pub database_id: u32,
    pub database_header: Header,
    pub html_representations: Vec<HtmlRepresentation>,
    pub optimized_data: OptimizedMetamathData,
    pub grammar_calculations_done: bool,
    pub database_path: String,
    pub syntax_typecodes: Vec<SyntaxTypecode>,
    pub logical_typecodes: Vec<LogicalTypecode>,
    pub variable_colors: Vec<VariableColor>,
    pub alt_variable_colors: Vec<VariableColor>,
}

pub struct IdManager {
    next_id: u32,
}

#[derive(Debug, Default)]
pub struct OptimizedMetamathData {
    pub floating_hypotheses: Vec<FloatingHypothesis>,
    pub theorem_amount: u32,
    pub theorem_data: HashMap<String, OptimizedTheoremData>,
    pub symbol_number_mapping: SymbolNumberMapping,
    pub grammar: Grammar,
}

#[derive(Debug)]
pub struct OptimizedTheoremData {
    pub theorem_type: TheoremType,
    pub parse_trees: Option<TheoremParseTrees>,
    pub distinct_variable_pairs: HashSet<(String, String)>,
    pub axiom_dependencies: Vec<usize>,
    pub definition_dependencies: Vec<usize>,
    pub references: Vec<usize>,
    pub description_parsed: Vec<ParsedDescriptionSegment>,
}

#[derive(Debug)]
pub enum TheoremType {
    Theorem,
    Axiom,
    Definition,
    SyntaxAxiom,
}

#[derive(Debug)]
pub struct TheoremParseTrees {
    pub hypotheses_parsed: Vec<ParseTree>,
    pub assertion_parsed: ParseTree,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParseTree {
    pub typecode: u32,
    pub top_node: ParseTreeNode,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ParseTreeNode {
    Node {
        rule_i: u32,
        sub_nodes: Vec<ParseTreeNode>,
    },
    WorkVariable(WorkVariable),
}

#[derive(Debug, Clone)]
pub enum ParsedDescriptionSegment {
    Text(String),
    MathMode(String),
    Label(String, Option<u32>),
    Link(String),
    Italic(String),
    Subscript(String),
    Html(String),
    HtmlCharacterRef(String),
}

#[derive(Debug, Default)]
pub struct SymbolNumberMapping {
    pub symbols: HashMap<u32, String>,
    pub numbers: HashMap<String, u32>,
    pub variable_typecodes: HashMap<u32, u32>,
    pub typecode_default_vars: Vec<(u32, u32)>,
    pub typecode_count: u32,
    pub variable_count: u32,
    pub constant_count: u32,
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
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

#[derive(Debug, Default, Clone)]
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

#[derive(Debug, Clone)]
pub struct SyntaxTypecode {
    pub typecode: String,
}

#[derive(Debug, Clone)]
pub struct LogicalTypecode {
    pub typecode: String,
    pub syntax_typecode: String,
}

#[derive(Debug)]
pub struct VariableColor {
    pub typecode: String,
    pub color: String,
}

#[derive(Serialize)]
pub struct ColorInformation {
    pub typecode: String,
    pub variables: Vec<String>,
    pub color: String,
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
    Empty,
    Header(HeaderPageData),
    Comment(CommentPageData),
    Constants(ConstantsPageData),
    Variables(VariablesPageData),
    FloatingHypothesis(FloatingHypothesisPageData),
    Theorem(TheoremPageData),
}

pub struct HeaderPageData {
    pub header_path: String,
    pub title: String,
    pub description: String,
}

pub struct CommentPageData {
    pub comment_path: String,
    pub comment: Comment,
}

pub struct ConstantsPageData {
    pub constants: Vec<Constant>,
}

pub struct VariablesPageData {
    pub variables: Vec<(Variable, String)>,
}

pub struct FloatingHypothesisPageData {
    pub floating_hypothesis: FloatingHypothesis,
}

pub struct TheoremPageData {
    pub theorem: Theorem,
    pub theorem_number: u32,
    pub proof_lines: Vec<ProofLine>,
    pub preview_errors: Option<Vec<(bool, bool, bool, bool)>>,
    pub preview_confirmations: Option<Vec<bool>>,
    pub preview_confirmations_recursive: Option<Vec<bool>>,
    pub preview_unify_markers: Option<Vec<(bool, bool, bool, bool)>>,
    pub last_theorem_label: Option<String>,
    pub next_theorem_label: Option<String>,
    pub axiom_dependencies: Vec<(String, u32)>,
    pub definition_dependencies: Vec<(String, u32)>,
    pub references: Vec<(String, u32)>,
    pub description_parsed: Vec<ParsedDescriptionSegment>,
}

pub struct ProofLine {
    pub step_name: String,
    pub hypotheses: Vec<String>,
    pub reference: String,
    pub reference_number: Option<u32>,
    pub indention: u32,
    pub assertion: String,
}

pub struct TheoremListData {
    pub list: Vec<ListEntry>,
    pub page_amount: u32,
    pub page_limits: Option<Vec<(u32, u32)>>,
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
    pub description_parsed: Vec<ParsedDescriptionSegment>,
}

pub struct FolderData {
    pub path: PathBuf,
    pub file_handles: HashMap<String, File>,
}

impl MetamathData {
    pub fn symbols_not_already_taken(&self, symbols: &Vec<&str>) -> bool {
        self.database_header.iter().all(|c| match c {
            DatabaseElement::Statement(s) => match s {
                Statement::CommentStatement(_) => true,
                Statement::ConstantStatement(consts) => {
                    for c in consts {
                        for symbol in symbols {
                            if &c.symbol == symbol {
                                return false;
                            }
                        }
                    }
                    true
                }
                Statement::VariableStatement(vars) => {
                    for v in vars {
                        for symbol in symbols {
                            if &v.symbol == symbol {
                                return false;
                            }
                        }
                    }
                    true
                }
                Statement::FloatingHypohesisStatement(fh) => {
                    for symbol in symbols {
                        if &fh.label == symbol {
                            return false;
                        }
                    }
                    true
                }
                Statement::TheoremStatement(t) => {
                    for symbol in symbols {
                        if &t.label == symbol {
                            return false;
                        }
                    }
                    true
                }
            },
            DatabaseElement::Header(_, _) => true,
        })
    }

    pub fn is_variable(&self, str: &str) -> bool {
        if self.grammar_calculations_done {
            self.optimized_data
                .symbol_number_mapping
                .numbers
                .get(str)
                .is_some_and(|&n| self.optimized_data.symbol_number_mapping.is_variable(n))
        } else {
            self.optimized_data
                .floating_hypotheses
                .iter()
                .any(|fh| fh.variable == str)
        }
    }

    pub fn expression_to_parse_tree(&self, expression: &str) -> Result<ParseTree, Error> {
        self.optimized_data
            .symbol_number_mapping
            .expression_to_parse_tree(
                expression,
                &self.optimized_data.grammar,
                &self.optimized_data.floating_hypotheses,
                &self.syntax_typecodes,
                &self.logical_typecodes,
            )
    }

    pub fn calc_optimized_theorem_data(
        &mut self,
        app: Option<&AppHandle>,
        allowed_tags_and_attributes: &HashMap<String, HashSet<String>>,
        allowed_css_properties: &HashSet<String>,
        stop: Option<Arc<std::sync::Mutex<bool>>>,
        settings: &Settings,
    ) -> Result<Vec<(String, String)>, Error> {
        let mut last_reported_progress = 0;

        let mut invalid_description_html = Vec::new();

        for (i, theorem) in self.database_header.theorem_iter().enumerate() {
            let theorem_type = theorem.calc_theorem_type(settings);

            let (axiom_dependencies, definition_dependencies) = theorem
                .calc_dependencies_and_add_references(&mut self.optimized_data, i, &theorem_type);

            let (description_parsed, invalid_html) = description_parser::parse_description(
                &theorem.description,
                &self.database_header,
                allowed_tags_and_attributes,
                allowed_css_properties,
            );

            invalid_description_html.extend(
                invalid_html
                    .into_iter()
                    .map(|html| (theorem.label.clone(), html)),
            );

            let optimized_theorem_data = OptimizedTheoremData {
                theorem_type,
                distinct_variable_pairs: util::calc_distinct_variable_pairs(&theorem.distincts),
                parse_trees: None,
                axiom_dependencies,
                definition_dependencies,
                references: Vec::new(),
                description_parsed,
            };

            self.optimized_data
                .theorem_data
                .insert(theorem.label.to_string(), optimized_theorem_data);

            if let Some(app_handle) = app {
                let progress = (i as u32 * 100) / self.optimized_data.theorem_amount;

                if progress > last_reported_progress {
                    app_handle
                        .emit("calc-optimized-theorem-data-progress", progress)
                        .ok();
                    last_reported_progress = progress;
                }
            }

            if let Some(ref stop_arc) = stop {
                let stop_bool = stop_arc.lock().or(Err(Error::InternalLogicError))?;
                if *stop_bool {
                    return Err(Error::OpenDatabaseStoppedEarlyError);
                }
            }

            if (i + 1) % 1000 == 0 {
                println!("Theorem data: {}", (i + 1));
            }
        }

        Ok(invalid_description_html)
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

    // pub fn recalc_symbol_number_mapping_and_grammar(&mut self) -> Result<(), Error> {
    //     self.optimized_data.symbol_number_mapping =
    //         SymbolNumberMapping::calc_mapping(&self.database_header);

    //     Grammar::calc_grammar(self)?;
    //     // let mut i: u32 = 1;
    //     // while let Some(symbol) = self.optimized_data.symbol_number_mapping.symbols.get(&i) {
    //     //     println!("{}: {}", i, symbol);
    //     //     if i == self.optimized_data.symbol_number_mapping.typecode_count
    //     //         || i == self.optimized_data.symbol_number_mapping.typecode_count
    //     //             + self.optimized_data.symbol_number_mapping.variable_count
    //     //     {
    //     //         println!("");
    //     //     }
    //     //     i += 1;
    //     // }
    //     // for grammar_rule in &self.optimized_data.grammar.rules {
    //     //     println!("{:?}", grammar_rule);
    //     // }
    //     Ok(())
    // }

    pub fn calc_color_information(&self, alt: bool) -> Vec<ColorInformation> {
        let variable_colors = if alt {
            &self.alt_variable_colors
        } else {
            &self.variable_colors
        };

        let mut color_information: Vec<ColorInformation> = variable_colors
            .iter()
            .map(|vc| ColorInformation {
                typecode: vc.typecode.clone(),
                variables: Vec::new(),
                color: vc.color.clone(),
            })
            .collect();

        self.database_header
            .floating_hypohesis_iter()
            .for_each(|fh| {
                color_information
                    .iter_mut()
                    .find(|ci| ci.typecode == fh.typecode)
                    .map(|ci| ci.variables.push(fh.variable.clone()));
            });

        color_information
    }

    pub fn syntax_typecode_of_logical_typecode(&self, logical_typecode_i: u32) -> Option<u32> {
        let logical_typecode = self
            .optimized_data
            .symbol_number_mapping
            .symbols
            .get(&logical_typecode_i)?;

        let syntax_typecode = &self
            .logical_typecodes
            .iter()
            .find(|lt| lt.typecode == *logical_typecode)?
            .syntax_typecode;

        let syntax_typecode_i = *self
            .optimized_data
            .symbol_number_mapping
            .numbers
            .get(syntax_typecode)?;

        Some(syntax_typecode_i)
    }
}

impl IdManager {
    pub fn new() -> IdManager {
        IdManager { next_id: 0 }
    }

    pub fn get_next_id(&mut self) -> u32 {
        self.next_id += 1;
        self.next_id - 1
    }
}

impl TheoremType {
    pub fn is_theorem(&self) -> bool {
        matches!(self, TheoremType::Theorem)
    }

    pub fn is_axiom(&self) -> bool {
        matches!(self, TheoremType::Axiom)
    }

    pub fn is_definition(&self) -> bool {
        matches!(self, TheoremType::Definition)
    }

    pub fn is_syntax_axiom(&self) -> bool {
        matches!(self, TheoremType::SyntaxAxiom)
    }
}

impl TheoremParseTrees {
    pub fn to_cloned_parse_tree_vec(&self) -> Vec<ParseTree> {
        let mut parse_trees_vec = self.hypotheses_parsed.clone();
        parse_trees_vec.push(self.assertion_parsed.clone());
        parse_trees_vec
    }

    pub fn to_ref_parse_tree_vec(&self) -> Vec<&ParseTree> {
        let mut parse_trees_vec: Vec<&ParseTree> = self.hypotheses_parsed.iter().collect();
        parse_trees_vec.push(&self.assertion_parsed);
        parse_trees_vec
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
    pub fn calc_proof(&self, grammar: &Grammar) -> Result<Option<String>, Error> {
        let mut proof = String::new();

        let mut trees = vec![(&self.top_node, 0)];

        while let Some((tree, next_node_i)) = trees.last_mut() {
            match tree {
                ParseTreeNode::Node { rule_i, sub_nodes } => {
                    if let Some(&node_i) = grammar
                        .rules
                        .get(*rule_i as usize)
                        .ok_or(Error::InternalLogicError)?
                        .var_order
                        .get(*next_node_i as usize)
                    {
                        let node = sub_nodes
                            .get(node_i as usize)
                            .ok_or(Error::InternalLogicError)?;

                        *next_node_i += 1;
                        trees.push((node, 0));
                    } else {
                        proof.push_str(
                            &grammar
                                .rules
                                .get(*rule_i as usize)
                                .ok_or(Error::InternalLogicError)?
                                .label,
                        );
                        proof.push(' ');
                        trees.pop();
                    }
                }
                ParseTreeNode::WorkVariable(_) => return Ok(None),
            }
        }

        proof.pop();

        Ok(Some(proof))
    }

    pub fn are_substitutions(
        trees: &Vec<&ParseTree>,
        other_trees: &Vec<&ParseTree>,
        distinct_vars: &HashSet<(String, String)>,
        other_distinct_vars: &HashSet<(String, String)>,
        grammar: &Grammar,
        symbol_number_mapping: &SymbolNumberMapping,
    ) -> Result<bool, Error> {
        if trees.len() != other_trees.len()
            || trees.iter().any(|t| t.has_work_variables())
            || other_trees.iter().any(|t| t.has_work_variables())
        {
            return Ok(false);
        }

        let mut substitutions: HashMap<u32, &ParseTreeNode> = HashMap::new();

        let mut check: Vec<(&ParseTreeNode, &ParseTreeNode)> = trees
            .iter()
            .zip(other_trees.iter())
            .map(|(t, o)| (&t.top_node, &o.top_node))
            .collect();

        while let Some((subtree, other_subtree)) = check.pop() {
            let ParseTreeNode::Node { rule_i, sub_nodes } = subtree else {
                return Err(Error::InternalLogicError);
            };
            let ParseTreeNode::Node {
                rule_i: other_rule_i,
                sub_nodes: other_sub_nodes,
            } = other_subtree
            else {
                return Err(Error::InternalLogicError);
            };

            let subtree_rule = grammar
                .rules
                .get(*rule_i as usize)
                .ok_or(Error::InternalLogicError)?;
            let other_subtree_rule = grammar
                .rules
                .get(*other_rule_i as usize)
                .ok_or(Error::InternalLogicError)?;

            if subtree_rule.is_floating_hypothesis {
                match substitutions.get(rule_i) {
                    Some(&sub) => {
                        if sub != other_subtree {
                            return Ok(false);
                        }
                    }
                    None => {
                        if subtree_rule.left_side == other_subtree_rule.left_side {
                            substitutions.insert(*rule_i, other_subtree);
                        } else {
                            return Ok(false);
                        }
                    }
                }
            } else {
                if *rule_i != *other_rule_i || sub_nodes.len() != other_sub_nodes.len() {
                    return Ok(false);
                }
                for (node, other_node) in sub_nodes.iter().zip(other_sub_nodes.iter()) {
                    check.push((node, other_node));
                }
            }
        }

        if !distinct_vars.is_empty() {
            let substitutions_variables: HashMap<&str, HashSet<u32>> = substitutions
                .iter()
                .filter_map(|(rule_i, &parse_tree)| {
                    if let Some(rule) = grammar.rules.get(*rule_i as usize) {
                        if let Some(right_side_first) = rule.right_side.first() {
                            if let Some(symbol) = symbol_number_mapping
                                .symbols
                                .get(&right_side_first.symbol_i)
                            {
                                if let Ok(vars_in_parse_tree) =
                                    parse_tree.get_floating_hypotheses_rules(grammar)
                                {
                                    return Some((&**symbol, vars_in_parse_tree));
                                }
                            }
                        }
                    }
                    // Should never be the case
                    None
                })
                .collect();

            for (var_1, var_2) in distinct_vars {
                if let Some(var_1_vars_in_parse_tree) = substitutions_variables.get(&**var_1) {
                    if let Some(var_2_vars_in_parse_tree) = substitutions_variables.get(&**var_2) {
                        for &var_1_var in var_1_vars_in_parse_tree.iter() {
                            for &var_2_var in var_2_vars_in_parse_tree.iter() {
                                if var_1_var == var_2_var
                                    || !other_distinct_vars.contains(&(
                                        ParseTree::get_floating_hypothesis_rule_variable_symbol(
                                            var_1_var,
                                            grammar,
                                            symbol_number_mapping,
                                        )?
                                        .clone(),
                                        ParseTree::get_floating_hypothesis_rule_variable_symbol(
                                            var_2_var,
                                            grammar,
                                            symbol_number_mapping,
                                        )?
                                        .clone(),
                                    ))
                                {
                                    return Ok(false);
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(true)
    }

    fn get_floating_hypothesis_rule_variable_symbol<'a>(
        rule_i: u32,
        grammar: &Grammar,
        symbol_number_mapping: &'a SymbolNumberMapping,
    ) -> Result<&'a String, Error> {
        symbol_number_mapping
            .symbols
            .get(
                &grammar
                    .rules
                    .get(rule_i as usize)
                    .ok_or(Error::InternalLogicError)?
                    .right_side
                    .first()
                    .ok_or(Error::InternalLogicError)?
                    .symbol_i,
            )
            .ok_or(Error::InternalLogicError)
    }

    fn has_work_variables(&self) -> bool {
        let mut check: Vec<&ParseTreeNode> = vec![&self.top_node];

        while let Some(node) = check.pop() {
            match node {
                ParseTreeNode::Node { sub_nodes, .. } => {
                    check.extend(sub_nodes.iter());
                }
                ParseTreeNode::WorkVariable(_) => {
                    return true;
                }
            }
        }

        false
    }
}

impl ParseTreeNode {
    pub fn get_floating_hypotheses_rules(&self, grammar: &Grammar) -> Result<HashSet<u32>, Error> {
        let mut rules: HashSet<u32> = HashSet::new();

        let mut check: Vec<&ParseTreeNode> = vec![self];

        while let Some(node) = check.pop() {
            if let ParseTreeNode::Node { rule_i, sub_nodes } = node {
                if grammar
                    .rules
                    .get(*rule_i as usize)
                    .ok_or(Error::InternalLogicError)?
                    .is_floating_hypothesis
                {
                    rules.insert(*rule_i);
                } else {
                    check.extend(sub_nodes.iter());
                }
            }
        }

        Ok(rules)
    }
}

impl SymbolNumberMapping {
    pub fn calc_mapping(header: &Header) -> SymbolNumberMapping {
        let mut symbols: HashMap<u32, String> = HashMap::new();
        let mut numbers: HashMap<String, u32> = HashMap::new();
        let mut variable_typecodes: HashMap<u32, u32> = HashMap::new();
        let mut typecode_default_vars: Vec<(u32, u32)> = Vec::new();
        let mut next_i: u32 = 1;
        let mut typecodes: Vec<&str> = Vec::new();

        for fh in header.floating_hypohesis_iter() {
            if !typecodes.contains(&&*fh.typecode) {
                typecodes.push(&fh.typecode);
                let typecode_string = format!("${}", fh.typecode);
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
                let typecode_string = format!("${}", fh.typecode);
                let variable_typecode_i = *numbers.get(&typecode_string).unwrap();
                variable_typecodes.insert(*num, variable_typecode_i);

                if typecode_default_vars
                    .iter()
                    .find(|(typecode_i, _)| *typecode_i == variable_typecode_i)
                    .is_none()
                {
                    typecode_default_vars.push((variable_typecode_i, *num));
                }
            }
        }

        SymbolNumberMapping {
            symbols,
            numbers,
            variable_typecodes,
            typecode_default_vars,
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

    pub fn expression_to_input_vec_skip_first(
        &self,
        expression: &str,
        floating_hypotheses: &Vec<FloatingHypothesis>,
    ) -> Result<Vec<InputSymbol>, Error> {
        if expression.split_ascii_whitespace().next().is_none() {
            return Err(Error::MissingExpressionError);
        }

        expression
            .split_ascii_whitespace()
            .skip(1)
            .map(|t| {
                if let Some((before, after)) = t.split_once('$') {
                    for floating_hypothesis in floating_hypotheses {
                        if before == floating_hypothesis.variable {
                            if after.starts_with('+') {
                                return Err(Error::InvalidWorkVariableError);
                            }

                            return Ok(InputSymbol::WorkVariable(WorkVariable {
                                typecode_i: *self
                                    .numbers
                                    .get(&format!("${}", floating_hypothesis.typecode))
                                    .ok_or(Error::InvalidWorkVariableError)?,
                                variable_i: *self
                                    .numbers
                                    .get(before)
                                    .ok_or(Error::InternalLogicError)?,
                                number: after.parse().or(Err(Error::InvalidWorkVariableError))?,
                            }));
                        }
                    }

                    Err(Error::InvalidWorkVariableError)
                } else {
                    Ok(InputSymbol::Symbol(Symbol {
                        symbol_i: *self
                            .numbers
                            .get(t)
                            .ok_or(Error::NonSymbolInExpressionError)?,
                    }))
                }
            })
            .collect::<Result<Vec<InputSymbol>, Error>>()
    }

    pub fn expression_to_parse_tree(
        &self,
        expression: &str,
        grammar: &Grammar,
        floating_hypotheses: &Vec<FloatingHypothesis>,
        syntax_typecodes: &Vec<SyntaxTypecode>,
        logical_typecodes: &Vec<LogicalTypecode>,
    ) -> Result<ParseTree, Error> {
        let expression_input_vec =
            self.expression_to_input_vec_skip_first(expression, floating_hypotheses)?;

        let typecode_str = expression
            .split_ascii_whitespace()
            .next()
            .ok_or(Error::InternalLogicError)?;

        let typecode = *self
            .numbers
            .get(typecode_str)
            .ok_or(Error::NonSymbolInExpressionError)?;

        let is_syntax_typecode = syntax_typecodes
            .iter()
            .any(|st| st.typecode == typecode_str);

        let logical_typecode_syntax_typecode_option = logical_typecodes.iter().find_map(|lt| {
            if lt.typecode == typecode_str {
                Some(&*lt.syntax_typecode)
            } else {
                None
            }
        });

        let syntax_typecode = if is_syntax_typecode {
            typecode_str
        } else if let Some(logical_typecode_syntax_typecode) =
            logical_typecode_syntax_typecode_option
        {
            logical_typecode_syntax_typecode
        } else {
            return Err(Error::InvalidTypecodeError);
        };

        let syntax_typecode_number = *self
            .numbers
            .get(&format!("${}", syntax_typecode))
            .ok_or(Error::SyntaxTypecodeWithoutFloatHypsError)?;

        let top_node = earley_parser_optimized::earley_parse(
            grammar,
            &expression_input_vec,
            vec![Symbol {
                symbol_i: syntax_typecode_number,
            }],
            self,
        )?
        .ok_or(Error::ExpressionParseError)?
        .into_iter()
        .next()
        .ok_or(Error::InternalLogicError)?;

        Ok(ParseTree { typecode, top_node })
    }

    pub fn is_typecode(&self, number: u32) -> bool {
        return 0 < number && number <= self.typecode_count;
    }

    pub fn is_variable(&self, number: u32) -> bool {
        return self.typecode_count < number && number <= self.typecode_count + self.variable_count;
    }

    // pub fn is_constant(&self, number: u32) -> bool {
    //     return self.typecode_count + self.variable_count < number;
    // }

    pub fn get_typecode_default_variable_i(&self, typecode_i: u32) -> Option<u32> {
        self.typecode_default_vars
            .iter()
            .find_map(|(typecode, default_var)| {
                if *typecode == typecode_i {
                    Some(*default_var)
                } else {
                    None
                }
            })
    }
}

impl Grammar {
    pub fn calc_grammar_and_parse_trees<'a>(
        database_header: &'a Header,
        symbol_number_mapping: &SymbolNumberMapping,
        floating_hypotheses: &Vec<FloatingHypothesis>,
        syntax_typecodes: &Vec<SyntaxTypecode>,
        logical_typecodes: &Vec<LogicalTypecode>,
        theorem_amount: u32,
        database_id: u32,
        app: Option<AppHandle>,
        stop: Option<Arc<std::sync::Mutex<bool>>>,
    ) -> Result<Option<(Grammar, Vec<(&'a str, ParseTree, Vec<ParseTree>)>)>, Error> {
        let mut grammar = Grammar {
            rules: Vec::new(),
            earley_optimized_data: EarleyOptimizedData::default(),
        };

        let mut parse_trees = Vec::new();

        let mut theorems_parsed = 0;
        let mut last_progress_reported = 0;

        for typecode in 0..symbol_number_mapping.typecode_count {
            grammar.rules.push(GrammarRule {
                left_side: Symbol {
                    symbol_i: typecode + 1,
                },
                right_side: vec![Symbol { symbol_i: 0 }],
                label: "WorkVariable".to_string(),
                var_order: Vec::new(),
                is_floating_hypothesis: false,
            });
        }

        for floating_hypothesis in floating_hypotheses {
            grammar.rules.push(GrammarRule {
                left_side: Symbol {
                    symbol_i: *symbol_number_mapping
                        .numbers
                        .get(&format!("${}", floating_hypothesis.typecode))
                        .ok_or(Error::InternalLogicError)?,
                },
                right_side: vec![Symbol {
                    symbol_i: *symbol_number_mapping
                        .numbers
                        .get(&floating_hypothesis.variable)
                        .ok_or(Error::InternalLogicError)?,
                }],
                label: floating_hypothesis.label.clone(),
                var_order: Vec::new(),
                is_floating_hypothesis: true,
            });
        }

        grammar.recalc_earley_optimized_data(symbol_number_mapping)?;

        for theorem in database_header.theorem_iter() {
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
                let left_side = Symbol {
                    symbol_i: *symbol_number_mapping
                        .numbers
                        .get(&format!("${}", assertion_token_iter.next().unwrap()))
                        .ok_or(Error::InternalLogicError)?,
                };

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
                        Ok(Symbol { symbol_i: num })
                    })
                    .collect::<Result<Vec<Symbol>, Error>>()?;

                let mut var_order: Vec<u32> = Vec::new();

                for floating_hypothesis in database_header.floating_hypohesis_iter() {
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

                grammar.rules.push(GrammarRule {
                    left_side,
                    right_side,
                    label: theorem.label.clone(),
                    var_order,
                    is_floating_hypothesis: false,
                });

                grammar.recalc_earley_optimized_data(symbol_number_mapping)?;
            } else if theorem
                .assertion
                .split_ascii_whitespace()
                .next()
                .ok_or(Error::InternalLogicError)?
                == "|-"
            {
                let (assertion_parsed, hypotheses_parsed) = theorem.calc_parse_trees(
                    &grammar,
                    symbol_number_mapping,
                    floating_hypotheses,
                    syntax_typecodes,
                    logical_typecodes,
                )?;

                parse_trees.push((theorem.label.as_str(), assertion_parsed, hypotheses_parsed));
            }

            if let Some(ref app_handle) = app {
                let progress = (theorems_parsed * 100) / theorem_amount;
                if progress > last_progress_reported {
                    last_progress_reported = progress;
                    app_handle
                        .emit("grammar-calculations-progress", (progress, database_id))
                        .ok();
                }
            }

            if let Some(ref stop_arc) = stop {
                let stop_bool = stop_arc.lock().or(Err(Error::InternalLogicError))?;
                if *stop_bool {
                    return Ok(None);
                }
            }

            theorems_parsed += 1;

            if theorems_parsed % 1000 == 0 {
                println!("Parse trees: {}", theorems_parsed);
            }
        }

        if let Some(ref app_handle) = app {
            app_handle
                .emit("grammar-calculations-progress", (100, database_id))
                .ok();
        }

        Ok(Some((grammar, parse_trees)))
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
            let right_side_first = rule.right_side.first().ok_or(Error::InternalLogicError)?;
            if right_side_first.symbol_i == 0 {
                continue;
            }

            if symbol_number_mapping.is_typecode(right_side_first.symbol_i) {
                completer_rules
                    .get_mut(rule.left_side.symbol_i as usize - 1)
                    .ok_or(Error::InternalLogicError)?
                    .get_mut(right_side_first.symbol_i as usize - 1)
                    .ok_or(Error::InternalLogicError)?
                    .push(rule_i);

                if !combined_states_to_add
                    .get(rule.left_side.symbol_i as usize - 1)
                    .ok_or(Error::InternalLogicError)?
                    .contains(&right_side_first.symbol_i)
                {
                    combined_states_to_add
                        .get_mut(rule.left_side.symbol_i as usize - 1)
                        .ok_or(Error::InternalLogicError)?
                        .push(right_side_first.symbol_i);
                }
            } else {
                single_states_to_add
                    .get_mut(rule.left_side.symbol_i as usize - 1)
                    .ok_or(Error::InternalLogicError)?
                    .get_mut(
                        (right_side_first.symbol_i - symbol_number_mapping.typecode_count - 1)
                            as usize,
                    )
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

impl ParsedDescriptionSegment {
    pub fn push(&mut self, char: char) {
        match self {
            Self::Text(string) => string.push(char),
            Self::MathMode(string) => string.push(char),
            Self::Label(string, _) => string.push(char),
            Self::Link(string) => string.push(char),
            Self::Italic(string) => string.push(char),
            Self::Subscript(string) => string.push(char),
            Self::Html(string) => string.push(char),
            Self::HtmlCharacterRef(string) => string.push(char),
        }
    }
}

impl FloatingHypothesis {
    pub fn to_assertions_string(&self) -> String {
        format!("{} {}", self.typecode, self.variable)
    }
}

impl Theorem {
    pub fn to_theorem_list_entry(
        &self,
        theorem_number: u32,
        optimized_data: &OptimizedMetamathData,
    ) -> TheoremListEntry {
        TheoremListEntry {
            label: self.label.clone(),
            theorem_number,
            hypotheses: self
                .hypotheses
                .iter()
                .map(|hypothesis| hypothesis.expression.clone())
                .collect(),
            assertion: self.assertion.clone(),
            description_parsed: optimized_data
                .theorem_data
                .get(&self.label)
                .map(|t_d| t_d.description_parsed.clone())
                .unwrap_or(Vec::new()),
        }
    }

    pub fn calc_parse_trees(
        &self,
        grammar: &Grammar,
        symbol_number_mapping: &SymbolNumberMapping,
        floating_hypotheses: &Vec<FloatingHypothesis>,
        syntax_typecodes: &Vec<SyntaxTypecode>,
        logical_typecodes: &Vec<LogicalTypecode>,
    ) -> Result<(ParseTree, Vec<ParseTree>), Error> {
        let hypotheses_parsed = self
            .hypotheses
            .iter()
            .map(|h| {
                symbol_number_mapping.expression_to_parse_tree(
                    &h.expression,
                    grammar,
                    floating_hypotheses,
                    syntax_typecodes,
                    logical_typecodes,
                )
            })
            .collect::<Result<Vec<ParseTree>, Error>>()?;

        let assertion_parsed = symbol_number_mapping.expression_to_parse_tree(
            &self.assertion,
            grammar,
            floating_hypotheses,
            syntax_typecodes,
            logical_typecodes,
        )?;

        // for hyp in &hypotheses_parsed {
        //     println!("{:?}", hyp.calc_proof(&grammar));
        // }
        // println!("{:?}", assertion_parsed.calc_proof(&grammar));

        Ok((assertion_parsed, hypotheses_parsed))
    }

    pub fn calc_dependencies_and_add_references(
        &self,
        optimized_data: &mut OptimizedMetamathData,
        i: usize,
        theorem_type: &TheoremType,
    ) -> (Vec<usize>, Vec<usize>) {
        let Some(proof) = self.proof.as_ref() else {
            return match theorem_type {
                TheoremType::Axiom => (vec![i], Vec::new()),
                TheoremType::Definition => (Vec::new(), vec![i]),
                _ => (Vec::new(), Vec::new()),
            };
        };

        let labels: Vec<&str> = if proof.starts_with("(") {
            proof
                .split_ascii_whitespace()
                .skip(1)
                .take_while(|token| *token != ")")
                .collect()
        } else {
            let mut already_seen: HashSet<&str> = HashSet::new();

            proof
                .split_ascii_whitespace()
                .filter(|label| {
                    if already_seen.contains(label) {
                        false
                    } else {
                        already_seen.insert(*label);
                        true
                    }
                })
                .collect()
        };

        // Add references to prior theorems
        for label in &labels {
            if let Some(theorem_data) = optimized_data.theorem_data.get_mut(*label) {
                theorem_data.references.push(i);
            }
        }

        Theorem::calc_dependencies_from_labels(&labels, optimized_data)
    }

    pub fn calc_dependencies_from_labels(
        labels: &Vec<&str>,
        optimized_data: &OptimizedMetamathData,
    ) -> (Vec<usize>, Vec<usize>) {
        let dependencies: Vec<(&Vec<usize>, &Vec<usize>)> = labels
            .iter()
            .filter_map(|label| {
                optimized_data.theorem_data.get(*label).map(|theorem_data| {
                    (
                        &theorem_data.axiom_dependencies,
                        &theorem_data.definition_dependencies,
                    )
                })
            })
            .collect();

        let (mut ax_result, mut df_result) = dependencies
            .first()
            .map(|(ax_dep_vec, df_dep_vec)| ((*ax_dep_vec).clone(), (*df_dep_vec).clone()))
            .unwrap_or((Vec::new(), Vec::new()));

        dependencies
            .iter()
            .skip(1)
            .for_each(|(ax_dep_vec, df_dep_vec)| {
                ax_dep_vec
                    .iter()
                    .for_each(|dep| match ax_result.binary_search(dep) {
                        Ok(_) => {}
                        Err(index) => {
                            ax_result.insert(index, *dep);
                        }
                    });
                df_dep_vec
                    .iter()
                    .for_each(|dep| match df_result.binary_search(dep) {
                        Ok(_) => {}
                        Err(index) => {
                            df_result.insert(index, *dep);
                        }
                    });
            });

        (ax_result, df_result)
    }

    pub fn calc_theorem_type(&self, settings: &Settings) -> TheoremType {
        if self.proof.is_some() {
            TheoremType::Theorem
        } else if !self.assertion.starts_with("|- ") {
            TheoremType::SyntaxAxiom
        } else if self.label.starts_with(&settings.definitons_start_with) {
            TheoremType::Definition
        } else {
            TheoremType::Axiom
        }
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
    }

    pub fn find_theorem_and_index_by_label(&self, label: &str) -> Option<(usize, &Theorem)> {
        self.theorem_iter()
            .enumerate()
            .find(|(_, t)| t.label == label)
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

    pub fn theorem_i_vec_to_theorem_label_vec(
        &self,
        theorem_i_vec: &Vec<usize>,
    ) -> Result<Vec<(String, u32)>, ()> {
        let mut theorem_iter = self.theorem_iter().enumerate();

        theorem_i_vec
            .iter()
            .map(|&i| {
                theorem_iter
                    .find(|(theorem_i, _)| *theorem_i == i)
                    .map(|(_, theorem)| (theorem.label.clone(), (i + 1) as u32))
            })
            .collect::<Option<Vec<(String, u32)>>>()
            .ok_or(())
    }

    pub fn theorem_label_vec_to_ordered_theorem_i_vec(
        &self,
        theorem_label_vec: &Vec<String>,
    ) -> Vec<usize> {
        self.theorem_iter()
            .enumerate()
            .filter_map(|(i, t)| {
                if theorem_label_vec.contains(&t.label) {
                    Some(i)
                } else {
                    None
                }
            })
            .collect()
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
        locate_after: Option<LocateAfterRef<'b>>,
    ) -> HeaderLocateAfterIterator<'a, 'b> {
        HeaderLocateAfterIterator::new(self, locate_after)
    }

    pub fn constant_locate_after_iter<'a, 'b>(
        &'a self,
        locate_after: Option<LocateAfterRef<'b>>,
    ) -> ConstantLocateAfterIterator<'a, 'b> {
        ConstantLocateAfterIterator::new(self, locate_after)
    }

    pub fn variable_locate_after_iter<'a, 'b>(
        &'a self,
        locate_after: Option<LocateAfterRef<'b>>,
    ) -> VariableLocateAfterIterator<'a, 'b> {
        VariableLocateAfterIterator::new(self, locate_after)
    }

    pub fn theorem_locate_after_iter<'a, 'b>(
        &'a self,
        locate_after: Option<LocateAfterRef<'b>>,
    ) -> TheoremLocateAfterIterator<'a, 'b> {
        TheoremLocateAfterIterator::new(self, locate_after)
    }

    // pub fn math_symbol_locate_after_iter<'a, 'b>(
    //     &'a self,
    //     locate_after: Option<LocateAfterRef<'b>>,
    // ) -> MathSymbolLocateAfterIterator<'a, 'b> {
    //     MathSymbolLocateAfterIterator::new(self, locate_after)
    // }
}

impl HeaderPath {
    pub fn from_str(str: &str) -> Option<HeaderPath> {
        if str.contains('+') {
            return None;
        }

        if str == "" {
            return Some(HeaderPath { path: Vec::new() });
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

impl serde::Serialize for Theorem {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        use serde::ser::SerializeStruct;

        let mut state = serializer.serialize_struct("Theorem", 6)?;
        state.serialize_field("label", &self.label)?;
        state.serialize_field("description", &self.description)?;
        state.serialize_field("distincts", &self.distincts)?;
        state.serialize_field("hypotheses", &self.hypotheses)?;
        state.serialize_field("assertion", &self.assertion)?;
        state.serialize_field("proof", &self.proof)?;
        state.end()
    }
}

impl serde::Serialize for ParsedDescriptionSegment {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        use serde::ser::SerializeStruct;

        match self {
            Self::Text(text) => {
                let mut state = serializer.serialize_struct("Text", 1)?;
                state.serialize_field("text", text)?;
                state.serialize_field("discriminator", "DescriptionText")?;
                state.end()
            }
            Self::MathMode(expression) => {
                let mut state = serializer.serialize_struct("MathMode", 1)?;
                state.serialize_field("expression", expression)?;
                state.serialize_field("discriminator", "DescriptionMathMode")?;
                state.end()
            }
            Self::Label(label, theorem_number) => {
                let mut state = serializer.serialize_struct("Label", 1)?;
                state.serialize_field("label", label)?;
                state.serialize_field("theoremNumber", theorem_number)?;
                state.serialize_field("discriminator", "DescriptionLabel")?;
                state.end()
            }
            Self::Link(url) => {
                let mut state = serializer.serialize_struct("Link", 1)?;
                state.serialize_field("url", url)?;
                state.serialize_field("discriminator", "DescriptionLink")?;
                state.end()
            }
            Self::Italic(italic) => {
                let mut state = serializer.serialize_struct("Italic", 1)?;
                state.serialize_field("italic", italic)?;
                state.serialize_field("discriminator", "DescriptionItalic")?;
                state.end()
            }
            Self::Subscript(subscript) => {
                let mut state = serializer.serialize_struct("Subscript", 1)?;
                state.serialize_field("subscript", subscript)?;
                state.serialize_field("discriminator", "DescriptionSubscript")?;
                state.end()
            }
            Self::Html(html) => {
                let mut state = serializer.serialize_struct("Html", 1)?;
                state.serialize_field("html", html)?;
                state.serialize_field("discriminator", "DescriptionHtml")?;
                state.end()
            }
            Self::HtmlCharacterRef(char_ref) => {
                let mut state = serializer.serialize_struct("HtmlCharacterRef", 1)?;
                state.serialize_field("charRef", char_ref)?;
                state.serialize_field("discriminator", "DescriptionHtmlCharacterRef")?;
                state.end()
            }
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
        use serde::ser::SerializeStruct;

        match self {
            Self::Empty => {
                let mut state = serializer.serialize_struct("EmptyPageData", 1)?;
                state.serialize_field("discriminator", "EmptyPageData")?;
                state.end()
            }
            Self::Header(header_page_data) => header_page_data.serialize(serializer),
            Self::Comment(comments_page_data) => comments_page_data.serialize(serializer),
            Self::Constants(constants_page_data) => constants_page_data.serialize(serializer),
            Self::Variables(variables_page_data) => variables_page_data.serialize(serializer),
            Self::FloatingHypothesis(floating_hypothesis_page_data) => {
                floating_hypothesis_page_data.serialize(serializer)
            }
            Self::Theorem(theorem_page_data) => theorem_page_data.serialize(serializer),
        }
    }
}

impl serde::Serialize for HeaderPageData {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        use serde::ser::SerializeStruct;

        let mut state = serializer.serialize_struct("HeaderPageData", 3)?;
        state.serialize_field("headerPath", &self.header_path)?;
        state.serialize_field("title", &self.title)?;
        state.serialize_field("description", &self.description)?;
        state.serialize_field("discriminator", "HeaderPageData")?;
        state.end()
    }
}

impl serde::Serialize for CommentPageData {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        use serde::ser::SerializeStruct;

        let mut state = serializer.serialize_struct("CommentPageData", 2)?;
        state.serialize_field("commentPath", &self.comment_path)?;
        state.serialize_field("comment", &self.comment)?;
        state.serialize_field("discriminator", "CommentPageData")?;
        state.end()
    }
}

impl serde::Serialize for ConstantsPageData {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        use serde::ser::SerializeStruct;

        let mut state = serializer.serialize_struct("ConstantsPageData", 1)?;
        state.serialize_field("constants", &self.constants)?;
        state.serialize_field("discriminator", "ConstantsPageData")?;
        state.end()
    }
}

impl serde::Serialize for VariablesPageData {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        use serde::ser::SerializeStruct;

        let mut state = serializer.serialize_struct("VariablesPageData", 1)?;
        state.serialize_field("variables", &self.variables)?;
        state.serialize_field("discriminator", "VariablesPageData")?;
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

impl serde::Serialize for TheoremPageData {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        use serde::ser::SerializeStruct;

        let mut state = serializer.serialize_struct("TheoremPageData", 11)?;
        state.serialize_field("theorem", &self.theorem)?;
        state.serialize_field("theoremNumber", &self.theorem_number)?;
        state.serialize_field("proofLines", &self.proof_lines)?;
        state.serialize_field("previewErrors", &self.preview_errors)?;
        state.serialize_field("previewConfirmations", &self.preview_confirmations)?;
        state.serialize_field(
            "previewConfirmationsRecursive",
            &self.preview_confirmations_recursive,
        )?;
        state.serialize_field("previewUnifyMarkers", &self.preview_unify_markers)?;
        state.serialize_field("lastTheoremLabel", &self.last_theorem_label)?;
        state.serialize_field("nextTheoremLabel", &self.next_theorem_label)?;
        state.serialize_field("axiomDependencies", &self.axiom_dependencies)?;
        state.serialize_field("definitionDependencies", &self.definition_dependencies)?;
        state.serialize_field("references", &self.references)?;
        state.serialize_field("descriptionParsed", &self.description_parsed)?;
        state.serialize_field("discriminator", "TheoremPageData")?;
        state.end()
    }
}

impl serde::Serialize for ProofLine {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        use serde::ser::SerializeStruct;

        let mut state = serializer.serialize_struct("ProofLine", 6)?;
        state.serialize_field("stepName", &self.step_name)?;
        state.serialize_field("hypotheses", &self.hypotheses)?;
        state.serialize_field("reference", &self.reference)?;
        state.serialize_field("referenceNumber", &self.reference_number)?;
        state.serialize_field("assertion", &self.assertion)?;
        state.serialize_field("indention", &self.indention)?;
        state.end()
    }
}

impl serde::Serialize for TheoremListData {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        use serde::ser::SerializeStruct;

        let mut state = serializer.serialize_struct("TheoremListData", 3)?;
        state.serialize_field("list", &self.list)?;
        state.serialize_field("pageAmount", &self.page_amount)?;
        state.serialize_field("pageLimits", &self.page_limits)?;
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
                state
                    .serialize_field("descriptionParsed", &theorem_list_entry.description_parsed)?;
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
