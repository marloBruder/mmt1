use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use crate::util::header_iterators::{
    ConstantIterator, HeaderIterator, TheoremIterator, VariableIterator,
};
use Statement::*;

#[derive(Debug, Default)]
pub struct MetamathData {
    // pub constants: Vec<Constant>,
    // pub variables: Vec<Variable>,
    // pub floating_hypotheses: Vec<FloatingHypohesis>,
    pub database_header: Header,
    pub html_representations: Vec<HtmlRepresentation>,
    pub optimized_data: OptimizedMetamathData,
    pub database_path: String,
}

#[derive(Debug, Default)]
pub struct OptimizedMetamathData {
    pub variables: HashSet<String>,
    pub floating_hypotheses: Vec<FloatingHypohesis>,
}

#[derive(Debug)]
pub enum Statement {
    CommentStatement(Comment),
    ConstantStatement(Constant),
    VariableStatement(Variable),
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
    pub disjoints: Vec<String>,
    pub hypotheses: Vec<Hypothesis>,
    pub assertion: String,
    pub proof: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Hypothesis {
    pub label: String,
    pub hypothesis: String,
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
}

#[derive(Serialize)]
pub struct ProofLine {
    pub hypotheses: Vec<i32>,
    pub reference: String,
    pub indention: i32,
    pub assertion: String,
}

pub struct TheoremListEntry {
    pub label: String,
    pub theorem_number: u32,
    pub hypotheses: Vec<String>,
    pub assertion: String,
    pub description: String,
}

impl MetamathData {
    pub fn label_exists(&self, label: &str) -> bool {
        if self.database_header.find_theorem_by_label(label).is_some() {
            return true;
        }

        false
    }

    pub fn valid_label(label: &str) -> bool {
        if label == "" {
            return false;
        }

        for ch in label.chars() {
            match ch {
                'a'..='z' | 'A'..='Z' | '0'..='9' | '_' | '.' | '-' => {}
                _ => {
                    return false;
                }
            }
        }
        true
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
                .map(|hypothesis| hypothesis.hypothesis.clone())
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
                    ConstantStatement(constant) => HeaderContentRepresentation {
                        content_type: "ConstantStatement".to_string(),
                        title: constant.symbol.clone(),
                    },
                    VariableStatement(variable) => HeaderContentRepresentation {
                        content_type: "VariableStatement".to_string(),
                        title: variable.symbol.clone(),
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

    pub fn find_theorem_by_label_calc_number(&self, label: &str) -> Option<(u32, &Theorem)> {
        self.theorem_iter()
            .enumerate()
            .find(|(_, t)| t.label == label)
            .map(|(i, t)| (i as u32, t))
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

    pub fn theorem_iter(&self) -> TheoremIterator {
        TheoremIterator::new(self)
    }
}

impl HeaderPath {
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

        let mut state = serializer.serialize_struct("TheoremPageData", 3)?;
        state.serialize_field("theorem", &self.theorem)?;
        state.serialize_field("theoremNumber", &self.theorem_number)?;
        state.serialize_field("proofLines", &self.proof_lines)?;
        state.end()
    }
}

impl serde::Serialize for TheoremListEntry {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        use serde::ser::SerializeStruct;

        let mut state = serializer.serialize_struct("TheoremListEntry", 5)?;
        state.serialize_field("label", &self.label)?;
        state.serialize_field("theoremNumber", &self.theorem_number)?;
        state.serialize_field("hypotheses", &self.hypotheses)?;
        state.serialize_field("assertion", &self.assertion)?;
        state.serialize_field("description", &self.description)?;
        state.end()
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
