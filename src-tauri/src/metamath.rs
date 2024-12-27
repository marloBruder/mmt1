use crate::{
    model::{Constant, FloatingHypohesis, Hypothesis, Theorem, Variable},
    AppState,
};
use std::fmt;
use tauri::{async_runtime::Mutex, State};

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

#[tauri::command]
pub async fn text_to_constants(
    state: tauri::State<'_, Mutex<AppState>>,
    text: &str,
) -> Result<Vec<Constant>, Error> {
    if !text.is_ascii() {
        return Err(Error::InvalidCharactersError);
    }

    let symbols = text_to_constant_or_variable_symbols(text, true)?;

    database::constant::set_constants(&state, &symbols)
        .await
        .or(Err(Error::SqlError))?;

    let mut constants = Vec::new();

    for symbol in symbols {
        constants.push(Constant {
            symbol: symbol.to_string(),
        })
    }

    Ok(constants)
}

#[tauri::command]
pub async fn text_to_variables(
    state: tauri::State<'_, Mutex<AppState>>,
    text: &str,
) -> Result<Vec<Variable>, Error> {
    if !text.is_ascii() {
        return Err(Error::InvalidCharactersError);
    }

    let symbols = text_to_constant_or_variable_symbols(text, false)?;

    database::variable::set_variables(&state, &symbols)
        .await
        .or(Err(Error::SqlError))?;

    let mut variables = Vec::new();

    for symbol in symbols {
        variables.push(Variable {
            symbol: symbol.to_string(),
        })
    }

    Ok(variables)
}

// Takes a text and returns references to the symbols between
// "$c" and "$.", if constant is true,
// "$v" and "$.", if constant is false
// If there is a string not between these, the function returns an Error
fn text_to_constant_or_variable_symbols(text: &str, constant: bool) -> Result<Vec<&str>, Error> {
    let mut symbols = Vec::new();

    // True if token is after "$c", but before the next "$."
    let mut within_statement = false;

    for token in text.split_whitespace() {
        if !within_statement {
            match token {
                "$c" if constant => within_statement = true,
                "$v" if !constant => within_statement = true,
                _ => return Err(Error::InvalidFormatError),
            }
        } else {
            match token {
                "$." => within_statement = false,
                s => symbols.push(s),
            }
        }
    }

    Ok(symbols)
}

#[tauri::command]
pub async fn text_to_floating_hypotheses(
    state: State<'_, Mutex<AppState>>,
    text: &str,
) -> Result<Vec<FloatingHypohesis>, Error> {
    if !text.is_ascii() {
        return Err(Error::InvalidCharactersError);
    }

    let mut floating_hypotheses = Vec::new();

    let mut token_iter = text.split_whitespace();

    let mut next_label: Option<&str> = None;

    while let Some(token) = token_iter.next() {
        match (token, next_label) {
            ("$f", Some(label)) => {
                let typecode = token_iter.next();
                let variable = token_iter.next();

                if token_iter.next() == Some("$.") {
                    floating_hypotheses.push(FloatingHypohesis {
                        label: label.to_string(),
                        // Safe unwraps, because the if branch requires a later call of next to have returned Some
                        typecode: typecode.unwrap().to_string(),
                        variable: variable.unwrap().to_string(),
                    })
                } else {
                    return Err(Error::InvalidFormatError);
                }
            }
            (label, None) => next_label = Some(label),
            (_, _) => return Err(Error::InvalidFormatError),
        }
    }

    database::floating_hypothesis::set_floating_hypotheses(&state, &floating_hypotheses)
        .await
        .or(Err(Error::SqlError))?;

    Ok(floating_hypotheses)
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
