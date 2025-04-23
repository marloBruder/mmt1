use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

use crate::{
    util::{
        earley_parser::{Grammar, GrammarRule},
        header_iterators::{
            ConstantIterator, FloatingHypothesisIterator, HeaderIterator, TheoremIterator,
            VariableIterator,
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
    pub floating_hypotheses: Vec<FloatingHypohesis>,
    pub symbol_number_mapping: SymbolNumberMapping,
    pub grammar: Grammar,
}

#[derive(Debug, Default)]
pub struct SymbolNumberMapping {
    pub symbols: HashMap<u32, String>,
    pub numbers: HashMap<String, u32>,
    pub variable_typecodes: HashMap<u32, u32>,
    pub typecode_count: u32,
    pub variable_count: u32,
}

#[derive(Debug)]
pub enum Statement {
    CommentStatement(Comment),
    ConstantStatement(Vec<Constant>),
    VariableStatement(Vec<Variable>),
    FloatingHypohesisStatement(FloatingHypohesis),
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
pub struct FloatingHypohesis {
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

pub struct TheoremPageData {
    pub theorem: Theorem,
    pub theorem_number: u32,
    pub proof_lines: Vec<ProofLine>,
    pub last_theorem_label: Option<String>,
    pub next_theorem_label: Option<String>,
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
    Theorem(TheoremListEntry),
    Header(HeaderListEntry),
}

pub struct TheoremListEntry {
    pub label: String,
    pub theorem_number: u32,
    pub hypotheses: Vec<String>,
    pub assertion: String,
    pub description: String,
}

pub struct HeaderListEntry {
    pub header_path: String,
    pub title: String,
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

        self.optimized_data.grammar = Grammar::calc_grammar(
            &self.database_header,
            &self.optimized_data.symbol_number_mapping,
        )?;
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

// impl Statement {
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
// }

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
    pub fn calc_grammar(
        header: &Header,
        symbol_number_mapping: &SymbolNumberMapping,
    ) -> Result<Grammar, Error> {
        let mut rules = Vec::new();

        for floating_hypothesis in header.floating_hypohesis_iter() {
            rules.push(GrammarRule {
                left_side: *symbol_number_mapping
                    .numbers
                    .get(&format!("${}", floating_hypothesis.typecode))
                    .ok_or(Error::InternalLogicError)?,
                right_side: vec![*symbol_number_mapping
                    .numbers
                    .get(&floating_hypothesis.variable)
                    .ok_or(Error::InternalLogicError)?],
                label: floating_hypothesis.label.clone(),
            });
        }

        for theorem in header.theorem_iter() {
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
                let right_side = assertion_token_iter
                    .map(|t| {
                        let mut num = *symbol_number_mapping
                            .numbers
                            .get(t)
                            .ok_or(Error::InternalLogicError)?;
                        if symbol_number_mapping.is_variable(num) {
                            num = *symbol_number_mapping
                                .variable_typecodes
                                .get(&num)
                                .ok_or(Error::InternalLogicError)?;
                        }
                        Ok(num)
                    })
                    .collect::<Result<Vec<u32>, Error>>()?;
                rules.push(GrammarRule {
                    left_side,
                    right_side,
                    label: theorem.label.clone(),
                });
            }
        }

        Ok(Grammar { rules })
    }
}

impl FloatingHypohesis {
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
}

impl HeaderPath {
    pub fn from_str(str: &str) -> Result<HeaderPath, ()> {
        if str.contains('+') {
            return Err(());
        }

        Ok(HeaderPath {
            path: str
                .split('.')
                .map(|s| {
                    let i = s.parse::<usize>().or(Err(()))?;
                    if i == 0 {
                        return Err(());
                    }
                    Ok(i - 1)
                })
                .collect::<Result<Vec<usize>, ()>>()?,
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
            Self::Header(ref header_list_entry) => {
                let mut state = serializer.serialize_struct("HeaderListEntry", 2)?;
                state.serialize_field("headerPath", &header_list_entry.header_path)?;
                state.serialize_field("title", &header_list_entry.title)?;
                state.serialize_field("discriminator", "HeaderListEntry")?;
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
