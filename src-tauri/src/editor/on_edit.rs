use crate::{
    model::{
        self, DatabaseElementPageData, FloatingHypothesis, FloatingHypothesisPageData, Hypothesis,
        Theorem, TheoremPageData,
    },
    util, AppState, Error,
};
use serde::{ser::SerializeStruct, Serialize};
use tauri::async_runtime::Mutex;

use super::unify::{self, MmpInfoStructuredForUnify, MmpLabel};

pub struct OnEditData {
    pub page_data: Option<DatabaseElementPageData>,
    pub errors: Vec<DetailedError>,
}

pub struct DetailedError {
    pub error_type: Error,
    pub start_line_number: u32,
    pub start_column: u32,
    pub end_line_number: u32,
    pub end_column: u32,
}

#[tauri::command]
pub async fn on_edit(
    state: tauri::State<'_, Mutex<AppState>>,
    text: &str,
) -> Result<OnEditData, Error> {
    let app_state = state.lock().await;
    let mm_data = app_state.metamath_data.as_ref().ok_or(Error::NoMmDbError)?;

    let (whitespace_before_first_statement, statement_strs) =
        match text_to_statement_strs_with_error_info(text)? {
            Ok(tuple) => tuple,
            Err(detailed_err) => {
                return Ok(OnEditData {
                    page_data: None,
                    errors: vec![detailed_err],
                })
            }
        };
    let mmp_info_structured_for_unify =
        unify::statement_strs_to_mmp_info_structured_for_unify(&statement_strs, mm_data)?;

    Ok(OnEditData {
        page_data: Some(get_database_element_page_data(
            &mmp_info_structured_for_unify,
        )?),
        errors: Vec::new(),
    })
}

// If successful, returns a tuple (a,b) where a is the whitespace before the first line and b is a vec of all the lines
pub fn text_to_statement_strs_with_error_info(
    text: &str,
) -> Result<Result<(&str, Vec<&str>), DetailedError>, Error> {
    let mut statements = Vec::new();

    let mut text_i: usize = 0;
    let text_bytes = text.as_bytes();

    let mut line_count: u32 = 1;
    let mut last_line_length: u32 = 0;

    while text_bytes
        .get(text_i)
        .is_some_and(|c| c.is_ascii_whitespace())
    {
        last_line_length += 1;

        if text_bytes.get(text_i).is_some_and(|c| *c == b'\n') {
            line_count += 1;
            last_line_length = 0;
        }

        text_i += 1;
    }

    let whitespace_before_first_statement = text.get(0..text_i).ok_or(Error::InternalLogicError)?;

    if text_i != 0 && text_bytes.get(text_i - 1).is_some_and(|c| *c != b'\n') {
        return Ok(Err(DetailedError {
            error_type: Error::WhitespaceBeforeFirstTokenError,
            start_line_number: line_count,
            start_column: 1,
            end_line_number: line_count,
            end_column: last_line_length + 1,
        }));
    }

    let mut statement_start = text_i;
    text_i += 1;

    while let Some(&char) = text_bytes.get(text_i) {
        if !char.is_ascii_whitespace() && text_bytes.get(text_i - 1).is_some_and(|c| *c == b'\n') {
            statements.push(
                text.get(statement_start..text_i)
                    .ok_or(Error::InternalLogicError)?,
            );
            statement_start = text_i;
        }

        text_i += 1;
    }

    statements.push(
        text.get(statement_start..text_i)
            .ok_or(Error::InternalLogicError)?,
    );

    Ok(Ok((whitespace_before_first_statement, statements)))
}

fn get_database_element_page_data(
    mmp_info_structured_for_unify: &MmpInfoStructuredForUnify,
) -> Result<DatabaseElementPageData, Error> {
    match mmp_info_structured_for_unify.label {
        Some(MmpLabel::Theorem(_)) => get_theorem_page_data(mmp_info_structured_for_unify),
        Some(MmpLabel::Axiom(_)) => get_theorem_page_data(mmp_info_structured_for_unify),
        Some(MmpLabel::Header {
            header_pos: _,
            title: _,
        }) => Err(Error::InternalLogicError),
        None => {
            if !mmp_info_structured_for_unify.floating_hypotheses.is_empty() {
                get_floating_hypothesis_page_data(mmp_info_structured_for_unify)
            } else {
                Err(Error::InternalLogicError)
            }
        }
    }
}

fn get_theorem_page_data(
    mmp_info_structured_for_unify: &MmpInfoStructuredForUnify,
) -> Result<DatabaseElementPageData, Error> {
    let axiom = matches!(
        mmp_info_structured_for_unify.label,
        Some(MmpLabel::Axiom(_))
    );

    let label = match mmp_info_structured_for_unify.label {
        Some(MmpLabel::Theorem(label)) => label,
        Some(MmpLabel::Axiom(label)) => label,
        _ => return Err(Error::InternalLogicError),
    };

    let label = label.to_string();

    let mut proof_lines: Vec<model::ProofLine> = Vec::new();

    let mut assertion = String::new();
    let mut hypotheses: Vec<Hypothesis> = Vec::new();

    for proof_line in &mmp_info_structured_for_unify.proof_lines {
        proof_lines.push(model::ProofLine {
            indention: 1,
            assertion: util::str_to_space_seperated_string(proof_line.expression),
            hypotheses: proof_line
                .hypotheses_parsed
                .iter()
                .map(|hyp| match hyp {
                    Some(num) => *num as i32 + 1,
                    None => 0,
                })
                .collect(),
            reference: proof_line.step_ref.to_string(),
        });

        if proof_line.step_name == "qed" {
            assertion = util::str_to_space_seperated_string(proof_line.expression);
        }
        if proof_line.is_hypothesis {
            hypotheses.push(Hypothesis {
                label: proof_line.step_ref.to_string(),
                expression: util::str_to_space_seperated_string(proof_line.expression),
            })
        }
    }

    let theorem = Theorem {
        label,
        description: mmp_info_structured_for_unify
            .comments
            .first()
            .unwrap_or(&"")
            .to_string(),
        assertion,
        distincts: mmp_info_structured_for_unify
            .distinct_vars
            .iter()
            .map(|d| util::str_to_space_seperated_string(d))
            .collect(),
        hypotheses,
        proof: if !axiom {
            Some("Proof not yet finished".to_string())
        } else {
            None
        },
    };

    if axiom {
        proof_lines.clear();
    }

    Ok(DatabaseElementPageData::Theorem(TheoremPageData {
        theorem,
        theorem_number: 0,
        proof_lines,
        last_theorem_label: None,
        next_theorem_label: None,
    }))
}

fn get_floating_hypothesis_page_data(
    mmp_info_structured_for_unify: &MmpInfoStructuredForUnify,
) -> Result<DatabaseElementPageData, Error> {
    if mmp_info_structured_for_unify.floating_hypotheses.len() > 1 {
        return Err(Error::StatementOutOfPlaceError);
    }

    let floating_hypothesis_str = *mmp_info_structured_for_unify
        .floating_hypotheses
        .first()
        .ok_or(Error::InternalLogicError)?;

    let mut token_iter = floating_hypothesis_str.split_ascii_whitespace();

    let label = token_iter
        .next()
        .ok_or(Error::FloatHypStatementFormatError)?
        .to_string();
    let typecode = token_iter
        .next()
        .ok_or(Error::FloatHypStatementFormatError)?
        .to_string();
    let variable = token_iter
        .next()
        .ok_or(Error::FloatHypStatementFormatError)?
        .to_string();

    if token_iter.next().is_some() {
        return Err(Error::FloatHypStatementFormatError);
    }

    Ok(DatabaseElementPageData::FloatingHypothesis(
        FloatingHypothesisPageData {
            floating_hypothesis: FloatingHypothesis {
                label,
                typecode,
                variable,
            },
        },
    ))
}

impl Serialize for OnEditData {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;

        let mut state = serializer.serialize_struct("OnEditData", 2)?;
        state.serialize_field("pageData", &self.page_data)?;
        state.serialize_field("errors", &self.errors)?;
        state.end()
    }
}

impl Serialize for DetailedError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("DetailedError", 5)?;
        state.serialize_field("errorType", &self.error_type)?;
        state.serialize_field("startLineNumber", &self.start_line_number)?;
        state.serialize_field("startColumn", &self.start_column)?;
        state.serialize_field("endLineNumber", &self.end_line_number)?;
        state.serialize_field("endColumn", &self.end_column)?;
        state.end()
    }
}
