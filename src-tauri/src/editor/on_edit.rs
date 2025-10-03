use crate::{
    metamath::{
        mm_parser::html_validation,
        mmp_parser::{
            self,
            calc_indention::{calc_indention, CalcIndention},
            MmpParserStage1, MmpParserStage2, MmpParserStage2Success, MmpParserStage3,
            MmpParserStage3Success, MmpParserStage3Theorem, MmpParserStage4, MmpParserStage5,
            ProofLineStatus,
        },
    },
    model::{
        self, CommentPageData, ConstantsPageData, DatabaseElementPageData,
        FloatingHypothesisPageData, HeaderPageData, Hypothesis, MetamathData, Theorem,
        TheoremPageData, VariablesPageData,
    },
    util::{self, description_parser},
    AppState, Error,
};
use serde::{ser::SerializeStruct, Serialize};
use tauri::async_runtime::Mutex;

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

    let stage_0 = mmp_parser::new(text);

    let stage_1_success = match stage_0.next_stage()? {
        MmpParserStage1::Success(success) => success,
        MmpParserStage1::Fail(fail) => {
            return Ok(OnEditData {
                page_data: None,
                errors: vec![fail.error],
            });
        }
    };

    let stage_2_success = match stage_1_success.next_stage()? {
        MmpParserStage2::Success(success) => success,
        MmpParserStage2::Fail(fail) => {
            return Ok(OnEditData {
                page_data: None,
                errors: fail.errors,
            });
        }
    };

    let stage_3_success = match stage_2_success.next_stage(&stage_1_success, mm_data)? {
        MmpParserStage3::Success(success) => success,
        MmpParserStage3::Fail(fail) => {
            return Ok(OnEditData {
                page_data: None,
                errors: fail.errors,
            })
        }
    };

    let stage_3_theorem = match stage_3_success {
        MmpParserStage3Success::Empty => {
            return Ok(OnEditData {
                page_data: Some(DatabaseElementPageData::Empty),
                errors: Vec::new(),
            })
        }
        MmpParserStage3Success::Header(mut stage_3_header) => {
            stage_3_header
                .parent_header_path
                .path
                .push(stage_3_header.header_i);
            return Ok(OnEditData {
                page_data: Some(DatabaseElementPageData::Header(HeaderPageData {
                    header_path: stage_3_header.parent_header_path.to_string(),
                    description: stage_3_header.description,
                    title: stage_3_header.title,
                })),
                errors: Vec::new(),
            });
        }
        MmpParserStage3Success::Comment(stage_3_comment) => {
            return Ok(OnEditData {
                page_data: Some(DatabaseElementPageData::Comment(CommentPageData {
                    comment_path: format!(
                        "{}#{}",
                        stage_3_comment.parent_header_path.to_string(),
                        stage_3_comment.comment_i
                    ),
                    comment: stage_3_comment.comment,
                })),
                errors: Vec::new(),
            })
        }
        MmpParserStage3Success::Constants(constants) => {
            return Ok(OnEditData {
                page_data: Some(DatabaseElementPageData::Constants(ConstantsPageData {
                    constants,
                })),
                errors: Vec::new(),
            })
        }
        MmpParserStage3Success::Variables(variables) => {
            return Ok(OnEditData {
                page_data: Some(DatabaseElementPageData::Variables(VariablesPageData {
                    variables: variables.into_iter().map(|v| (v, String::new())).collect(),
                })),
                errors: Vec::new(),
            })
        }
        MmpParserStage3Success::FloatingHypohesis(floating_hypothesis) => {
            return Ok(OnEditData {
                page_data: Some(DatabaseElementPageData::FloatingHypothesis(
                    FloatingHypothesisPageData {
                        floating_hypothesis,
                    },
                )),
                errors: Vec::new(),
            })
        }
        MmpParserStage3Success::Theorem(stage_3_theorem) => stage_3_theorem,
    };

    let stage_4_success =
        match stage_3_theorem.next_stage(&stage_1_success, &stage_2_success, mm_data)? {
            MmpParserStage4::Success(success) => success,
            MmpParserStage4::Fail(fail) => {
                return Ok(OnEditData {
                    page_data: Some(calc_theorem_page_data(
                        mm_data,
                        &stage_2_success,
                        stage_3_theorem,
                        fail.reference_numbers,
                        fail.proof_line_statuses,
                        None,
                    )?),
                    errors: fail.errors,
                })
            }
        };

    let stage_5 = stage_4_success.next_stage(&stage_2_success, &stage_3_theorem, mm_data)?;

    Ok(OnEditData {
        page_data: Some(calc_theorem_page_data(
            mm_data,
            &stage_2_success,
            stage_3_theorem,
            stage_4_success.reference_numbers,
            stage_4_success.proof_line_statuses,
            Some(stage_5),
        )?),
        errors: Vec::new(),
    })
}

pub fn calc_theorem_page_data(
    mm_data: &MetamathData,
    stage_2_success: &MmpParserStage2Success,
    stage_3_theorem: MmpParserStage3Theorem,
    reference_numbers: Vec<Option<u32>>,
    proof_line_statuses: Vec<ProofLineStatus>,
    stage_5: Option<MmpParserStage5>,
) -> Result<DatabaseElementPageData, Error> {
    let (html_allowed_tags_and_attributes, css_allowed_properties) =
        html_validation::create_rule_structs();

    let mut proof_lines: Vec<model::ProofLine> = Vec::new();
    let mut preview_errors: Vec<(bool, bool, bool, bool)> = Vec::new();
    let mut preview_deleted_markers: Vec<bool> = Vec::new();
    let mut preview_confirmations: Vec<bool> = Vec::new();
    let mut preview_confirmations_recursive: Vec<bool> = Vec::new();
    let mut preview_unify_markers: Vec<(bool, bool, bool, bool)> = Vec::new();

    if let Some(stage_5) = stage_5 {
        let indention_vec = calc_indention(&stage_5.unify_result)?;

        for ((ul, reference_number), indention) in stage_5
            .unify_result
            .into_iter()
            .zip(stage_5.unify_reference_numbers.into_iter())
            .zip(indention_vec.into_iter())
        {
            if !(ul.deleted_line && ul.new_line) {
                update_preview_markers(
                    ul.status,
                    ul.deleted_line,
                    &mut preview_errors,
                    &mut preview_deleted_markers,
                    &mut preview_confirmations,
                    &mut preview_confirmations_recursive,
                    &mut preview_unify_markers,
                );

                let assertion = ul
                    .parse_tree
                    .map(|pt| {
                        pt.to_expression(
                            &mm_data.optimized_data.symbol_number_mapping,
                            &mm_data.optimized_data.grammar,
                        )
                    })
                    .transpose()?
                    .unwrap_or(String::new());

                proof_lines.push(model::ProofLine {
                    step_name: ul.step_name,
                    hypotheses: ul.hypotheses,
                    reference: ul.step_ref,
                    indention,
                    reference_number,
                    old_assertion: if ul.old_assertion.as_ref().is_none_or(|oa| *oa == assertion) {
                        None
                    } else {
                        ul.old_assertion
                    },
                    assertion,
                });
            }
        }
    } else {
        let indention_vec = calc_indention(&stage_2_success.proof_lines)?;

        for (((pl, indention), ref_number), status) in stage_2_success
            .proof_lines
            .iter()
            .zip(indention_vec.into_iter())
            .zip(reference_numbers.into_iter())
            .zip(proof_line_statuses.into_iter())
        {
            update_preview_markers(
                status,
                false,
                &mut preview_errors,
                &mut preview_deleted_markers,
                &mut preview_confirmations,
                &mut preview_confirmations_recursive,
                &mut preview_unify_markers,
            );

            proof_lines.push(model::ProofLine {
                step_name: pl.step_name.to_string(),
                hypotheses: if pl.hypotheses.len() != 0 {
                    pl.hypotheses.split(',').map(|s| s.to_string()).collect()
                } else {
                    Vec::new()
                },
                reference: pl.step_ref.to_string(),
                reference_number: ref_number,
                indention,
                assertion: util::str_to_space_seperated_string(pl.expression),
                old_assertion: None,
            });
        }
    }

    let description = stage_2_success
        .comments
        .first()
        .map(|s| s.to_string())
        .unwrap_or(String::new());

    Ok(DatabaseElementPageData::Theorem(TheoremPageData {
        description_parsed: description_parser::parse_description(
            &description,
            &mm_data.database_header,
            &html_allowed_tags_and_attributes,
            &css_allowed_properties,
        )
        .0,
        theorem: Theorem {
            label: stage_3_theorem.label.to_string(),
            description,
            distincts: stage_2_success
                .distinct_vars
                .iter()
                .map(|s| util::str_to_space_seperated_string(s))
                .collect(),
            hypotheses: stage_2_success
                .proof_lines
                .iter()
                .filter(|pl| pl.is_hypothesis)
                .map(|pl| Hypothesis {
                    label: pl.step_ref.to_string(),
                    expression: util::str_to_space_seperated_string(pl.expression),
                })
                .collect(),
            assertion: stage_2_success
                .proof_lines
                .iter()
                .find(|pl| pl.step_name == "qed")
                .map(|pl| util::str_to_space_seperated_string(pl.expression))
                .unwrap_or(String::new()),
            proof: if stage_3_theorem.is_axiom {
                None
            } else {
                Some("Proof not yet complete".to_string())
            },
        },
        theorem_number: 0,
        proof_lines,
        preview_errors: Some(preview_errors),
        preview_deleted_markers: Some(preview_deleted_markers),
        preview_confirmations: Some(preview_confirmations),
        preview_confirmations_recursive: Some(preview_confirmations_recursive),
        preview_unify_markers: Some(preview_unify_markers),
        last_theorem_label: None,
        next_theorem_label: None,
        axiom_dependencies: stage_3_theorem.axiom_dependencies,
        definition_dependencies: stage_3_theorem.definition_dependencies,
        references: Vec::new(),
    }))
}

fn update_preview_markers(
    status: ProofLineStatus,
    deleted_line: bool,
    preview_errors: &mut Vec<(bool, bool, bool, bool)>,
    preview_deleted_markers: &mut Vec<bool>,
    preview_confirmations: &mut Vec<bool>,
    preview_confirmations_recursive: &mut Vec<bool>,
    preview_unify_markers: &mut Vec<(bool, bool, bool, bool)>,
) {
    let mut preview_errors_marker = (false, false, false, false);
    let mut preview_deleted_marker = false;
    let mut preview_confirmations_marker = false;
    let mut preview_confirmations_recursive_marker = false;
    let mut preview_unify_marker = (false, false, false, false);

    if deleted_line {
        preview_deleted_marker = true;
    } else {
        match status {
            ProofLineStatus::None => {}
            ProofLineStatus::Err(err) => {
                preview_errors_marker = err;
            }
            ProofLineStatus::Correct => {
                preview_confirmations_marker = true;
            }
            ProofLineStatus::CorrectRecursively => {
                preview_confirmations_recursive_marker = true;
            }
            ProofLineStatus::Unified(unified, _) => {
                preview_unify_marker = unified;
            }
        }
    }

    preview_errors.push(preview_errors_marker);
    preview_deleted_markers.push(preview_deleted_marker);
    preview_confirmations.push(preview_confirmations_marker);
    preview_confirmations_recursive.push(preview_confirmations_recursive_marker);
    preview_unify_markers.push(preview_unify_marker);
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
