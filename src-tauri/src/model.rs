use serde::{Deserialize, Serialize};

#[derive(Debug, Default)]
pub struct MetamathData {
    pub constants: Vec<Constant>,
    pub variables: Vec<Variable>,
    pub floating_hypotheses: Vec<FloatingHypohesis>,
    pub theorems: Vec<Theorem>,
    pub in_progress_theorems: Vec<InProgressTheorem>,
    pub theorem_list_header: Header,
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
    pub name: String,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InProgressTheorem {
    pub name: String,
    pub text: String,
}

#[derive(Debug, Default)]
pub struct Header {
    pub title: String,
    pub theorems: Vec<Theorem>,
    pub sub_headers: Vec<Header>,
}

pub struct HeaderRepresentation {
    pub title: String,
    pub theorem_names: Vec<String>,
    pub sub_header_names: Vec<String>,
}

#[derive(Deserialize)]
pub struct HeaderPath {
    pub path: Vec<usize>,
}

pub struct TheoremPath {
    pub header_path: HeaderPath,
    pub theorem_index: usize,
}

pub struct TheoremPageData {
    pub theorem: Theorem,
    pub proof_lines: Vec<ProofLine>,
}

#[derive(Serialize)]
pub struct ProofLine {
    pub hypotheses: Vec<i32>,
    pub reference: String,
    pub indention: i32,
    pub assertion: String,
}

impl Header {
    pub fn representation(&self) -> HeaderRepresentation {
        HeaderRepresentation {
            title: self.title.clone(),
            theorem_names: self.theorems.iter().map(|t| t.name.clone()).collect(),
            sub_header_names: self.sub_headers.iter().map(|sh| sh.title.clone()).collect(),
        }
    }

    pub fn find_theorem_by_name(&self, name: &str) -> Option<&Theorem> {
        for theorem in &self.theorems {
            if theorem.name == name {
                return Some(theorem);
            }
        }

        for sub_header in &self.sub_headers {
            let sub_header_res = sub_header.find_theorem_by_name(name);
            if sub_header_res.is_some() {
                return sub_header_res;
            }
        }

        None
    }

    pub fn calc_theorem_path_by_name(&self, name: &str) -> Option<TheoremPath> {
        for (index, theorem) in self.theorems.iter().enumerate() {
            if theorem.name == name {
                return Some(TheoremPath {
                    header_path: HeaderPath { path: Vec::new() },
                    theorem_index: index,
                });
            }
        }

        for (index, sub_header) in self.sub_headers.iter().enumerate() {
            let sub_header_res = sub_header.calc_theorem_path_by_name(name);
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

        for (index, sub_header) in self.sub_headers.iter().enumerate() {
            let sub_header_res = sub_header.calc_header_path_by_title(title);
            if let Some(mut header_path) = sub_header_res {
                header_path.path.insert(0, index);
                return Some(header_path);
            }
        }

        None
    }

    pub fn count_theorems_and_headers(&self) -> i32 {
        let mut sum = 1 + self.theorems.len() as i32;
        for sub_header in &self.sub_headers {
            sum += sub_header.count_theorems_and_headers();
        }
        sum
    }
}

impl HeaderPath {
    pub fn resolve<'a>(&self, top_header: &'a Header) -> Option<&'a Header> {
        let mut header = top_header;

        for &index in &self.path {
            header = header.sub_headers.get(index)?;
        }

        Some(header)
    }

    pub fn resolve_mut<'a>(&self, top_header: &'a mut Header) -> Option<&'a mut Header> {
        let mut header = top_header;

        for &index in &self.path {
            header = header.sub_headers.get_mut(index)?;
        }

        Some(header)
    }
}

impl serde::Serialize for HeaderRepresentation {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        use serde::ser::SerializeStruct;

        let mut state = serializer.serialize_struct("HeaderRepresentation", 2)?;
        state.serialize_field("title", &self.title)?;
        state.serialize_field("theoremNames", &self.theorem_names)?;
        state.serialize_field("subHeaderNames", &self.sub_header_names)?;
        state.end()
    }
}

impl serde::Serialize for TheoremPageData {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        use serde::ser::SerializeStruct;

        let mut state = serializer.serialize_struct("TheoremPageData", 2)?;
        state.serialize_field("theorem", &self.theorem)?;
        state.serialize_field("proofLines", &self.proof_lines)?;
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
