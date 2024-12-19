use std::fmt;

#[tauri::command]
pub fn text_to_axium(text: &str) -> Result<Theorem, Error> {
    if !text.is_ascii() {
        return Err(Error::InvalidCharactersError);
    }

    let mut last_token: Option<&str> = None;

    let mut name: Option<String> = None;
    let mut description = String::from("");
    let mut disjoints: Vec<String> = Vec::new();
    let mut hypotheses: Vec<Hypothesis> = Vec::new();
    let mut assertion: Option<String> = None;

    let mut token_iter = text.split_whitespace();
    while let Some(token) = token_iter.next() {
        match token {
            "$(" => description = get_next_as_string_until(&mut token_iter, "$)"),
            "$d" => {
                let disjoint_cond = get_next_as_string_until(&mut token_iter, "$.");
                disjoints.push(disjoint_cond);
            }
            "$e" => {
                let label = last_token.ok_or(Error::InvalidFormatError)?.to_string();
                let hypothesis = get_next_as_string_until(&mut token_iter, "$.");
                hypotheses.push(Hypothesis { label, hypothesis })
            }
            "$a" => {
                name = last_token.map(|s| s.to_string());
                assertion = Some(get_next_as_string_until(&mut token_iter, "$."));
            }
            _ => {
                last_token = Some(token);
            }
        }
    }

    let name = name.ok_or(Error::InvalidFormatError)?;
    let assertion = assertion.ok_or(Error::InvalidFormatError)?;

    Ok(Theorem {
        name,
        description,
        disjoints,
        hypotheses,
        assertion,
        proof: None,
    })
}

fn get_next_as_string_until(iter: &mut std::str::SplitWhitespace, until: &str) -> String {
    let mut result = String::new();
    while let Some(token) = iter.next() {
        if token == until {
            break;
        } else {
            result.push_str(token);
            result.push(' ');
        }
    }
    result.pop();
    result
}

#[derive(Debug)]
pub struct Theorem {
    pub name: String,
    pub description: String,
    pub disjoints: Vec<String>,
    pub hypotheses: Vec<Hypothesis>,
    pub assertion: String,
    pub proof: Option<String>,
}

impl serde::Serialize for Theorem {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        use serde::ser::SerializeStruct;

        // 3 is the number of fields in the struct.
        let mut state = serializer.serialize_struct("InProgressTheorem", 6)?;
        state.serialize_field("name", &self.name)?;
        state.serialize_field("description", &self.description)?;
        state.serialize_field("disjoints", &self.disjoints)?;
        state.serialize_field("hypotheses", &self.hypotheses)?;
        state.serialize_field("assertion", &self.assertion)?;
        state.serialize_field("proof", &self.proof)?;
        state.end()
    }
}

#[derive(Debug)]
pub struct Hypothesis {
    pub label: String,
    pub hypothesis: String,
}

impl serde::Serialize for Hypothesis {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        use serde::ser::SerializeStruct;

        // 3 is the number of fields in the struct.
        let mut state = serializer.serialize_struct("InProgressTheorem", 2)?;
        state.serialize_field("label", &self.label)?;
        state.serialize_field("hypothesis", &self.hypothesis)?;
        state.end()
    }
}

#[derive(Debug)]
pub enum Error {
    InvalidCharactersError,
    InvalidFormatError,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl serde::Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}
