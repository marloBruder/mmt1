use crate::{
    model::{
        Constant, FloatingHypohesis, Hypothesis, InProgressTheorem, MetamathData, Theorem,
        TheoremPageData, Variable,
    },
    AppState,
};
use std::fmt;
use tauri::{async_runtime::Mutex, State};

use crate::database::{self};

#[tauri::command]
pub async fn turn_into_theorem(
    state: tauri::State<'_, Mutex<AppState>>,
    in_progress_theorem: InProgressTheorem,
) -> Result<(), Error> {
    if !in_progress_theorem.text.is_ascii() {
        return Err(Error::InvalidCharactersError);
    }

    let mut last_token: Option<&str> = None;

    let mut name: Option<String> = None;
    let mut description = String::from("");
    let mut disjoints: Vec<String> = Vec::new();
    let mut hypotheses: Vec<Hypothesis> = Vec::new();
    let mut assertion: Option<String> = None;
    let mut proof: Option<String> = None;

    let mut token_iter = in_progress_theorem.text.split_whitespace();
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
            "$p" => {
                name = last_token.map(|s| s.to_string());
                assertion = Some(get_next_as_string_until(&mut token_iter, "$="));
                proof = Some(get_next_as_string_until(&mut token_iter, "$."));
            }
            _ => {
                last_token = Some(token);
            }
        }
    }

    let name = name.ok_or(Error::InvalidFormatError)?;
    let assertion = assertion.ok_or(Error::InvalidFormatError)?;

    if name != in_progress_theorem.name {
        return Err(Error::InvalidFormatError);
    }

    database::theorem::add_theorem(
        &state,
        &name,
        &description,
        &disjoints,
        &hypotheses,
        &assertion,
        proof.as_deref(),
    )
    .await
    .or(Err(Error::SqlError))?;

    database::in_progress_theorem::delete_in_progress_theorem(state, &name)
        .await
        .or(Err(Error::SqlError))?;

    Ok(())
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
                    });
                    next_label = None;
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

pub fn calc_theorem_page_data(
    theorem: &Theorem,
    metamath_data: &MetamathData,
) -> Result<TheoremPageData, Error> {
    let _proof_steps = calc_proof_steps(theorem, metamath_data);
    Ok(TheoremPageData {
        theorem: theorem.clone(),
        proof_lines: Vec::new(),
    })
}

#[derive(Debug)]
struct ProofStep {
    pub hypotheses: Vec<String>,
    pub statement: String,
}

fn calc_proof_steps(
    theorem: &Theorem,
    metamath_data: &MetamathData,
) -> Result<Vec<ProofStep>, Error> {
    if theorem.proof == None {
        return Ok(Vec::new());
    }

    let mut steps = Vec::new();

    let hypotheses = calc_all_hypotheses_of_theorem(theorem, metamath_data);

    for hypothesis in hypotheses {
        steps.push(ProofStep {
            hypotheses: Vec::new(),
            statement: hypothesis,
        })
    }

    // Safe unwrap due to if at start of function
    for token in theorem.proof.as_ref().unwrap().split_whitespace() {
        match token {
            "(" => {}
            ")" => break,
            label => {
                let label_theorem = metamath_data.get_theorem_by_name(label)?;
                let label_theorem_hypotheses =
                    calc_all_hypotheses_of_theorem(label_theorem, metamath_data);
                steps.push(ProofStep {
                    hypotheses: label_theorem_hypotheses,
                    statement: label_theorem.assertion.clone(),
                })
            }
        };
    }

    Ok(steps)
}

fn calc_all_hypotheses_of_theorem(theorem: &Theorem, metamath_data: &MetamathData) -> Vec<String> {
    let mut hypotheses = Vec::new();

    // Calculate variables occuring in assertion and hypotheses
    let variables = calc_variables_of_theorem(theorem, metamath_data);

    // Calculate proof steps of floating hypotheses
    for floating_hypothesis in &metamath_data.floating_hypotheses {
        for &variable in &variables {
            if floating_hypothesis.variable == variable {
                let mut statement = floating_hypothesis.typecode.clone();
                statement.push(' ');
                statement.push_str(&floating_hypothesis.variable);
                hypotheses.push(statement)
            }
        }
    }

    // Calculate proof steps of essential hypotheses
    for hypothesis in &theorem.hypotheses {
        hypotheses.push(hypothesis.hypothesis.clone());
    }

    hypotheses
}

fn calc_variables_of_theorem<'a>(
    theorem: &'a Theorem,
    metamath_data: &MetamathData,
) -> Vec<&'a str> {
    let mut variables = get_variables_from_statement(&theorem.assertion, metamath_data);

    for hypothesis in &theorem.hypotheses {
        let hypothesis_vars = get_variables_from_statement(&hypothesis.hypothesis, metamath_data);
        for var in hypothesis_vars {
            if !variables.contains(&var) {
                variables.push(var);
            }
        }
    }
    variables
}

fn get_variables_from_statement<'a>(
    statement: &'a str,
    metamath_data: &MetamathData,
) -> Vec<&'a str> {
    let mut vars = Vec::new();
    for token in statement.split_whitespace() {
        for variable in &metamath_data.variables {
            if variable.symbol == token {
                vars.push(token);
            }
        }
    }
    vars
}

#[derive(Debug)]
pub enum Error {
    InvalidCharactersError,
    InvalidFormatError,
    SqlError,
    NotFoundError,
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
