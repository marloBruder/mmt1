use crate::{
    model::{Hypothesis, Theorem},
    AppState,
};
use std::fmt;
use tauri::async_runtime::Mutex;

use crate::database::{self};

#[tauri::command]
pub async fn text_to_axium(
    state: tauri::State<'_, Mutex<AppState>>,
    text: &str,
) -> Result<Theorem, Error> {
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

    database::theorem::add_theorem(
        &state,
        &name,
        &description,
        &disjoints,
        &hypotheses,
        &assertion,
        None,
    )
    .await
    .or(Err(Error::SqlError))?;

    database::in_progress_theorem::delete_in_progress_theorem(state, &name)
        .await
        .or(Err(Error::SqlError))?;

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
pub enum Error {
    InvalidCharactersError,
    InvalidFormatError,
    SqlError,
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
