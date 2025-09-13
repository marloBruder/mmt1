use crate::{
    metamath::{
        mm_parser::html_validation,
        mmp_parser::{
            self, MmpParserStage1, MmpParserStage2, MmpParserStage2Success, MmpParserStage3,
            MmpParserStage3Success, MmpParserStage3Theorem, MmpParserStage4, MmpParserStage5,
        },
    },
    model::{
        self, CommentPageData, ConstantsPageData, DatabaseElementPageData, FloatingHypothesis,
        FloatingHypothesisPageData, HeaderPageData, Hypothesis, MetamathData, ParseTree,
        ParseTreeNode, SymbolNumberMapping, Theorem, TheoremPageData, VariablesPageData,
    },
    util::{self, description_parser},
    AppState, Error,
};
use serde::{ser::SerializeStruct, Serialize};
use tauri::async_runtime::Mutex;

use super::unify::{LocateAfterRef, MmpInfoStructuredForUnify, MmpLabel, MmpStatement, ProofLine};

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
                    page_data: calc_theorem_page_data(
                        mm_data,
                        &stage_2_success,
                        stage_3_theorem,
                        fail.reference_numbers,
                        fail.preview_errors,
                        fail.preview_confirmations,
                        fail.preview_confirmations_recursive,
                        None,
                    ),
                    errors: fail.errors,
                })
            }
        };

    let stage_5 = stage_4_success.next_stage(&stage_2_success, mm_data)?;

    Ok(OnEditData {
        page_data: calc_theorem_page_data(
            mm_data,
            &stage_2_success,
            stage_3_theorem,
            stage_4_success.reference_numbers,
            stage_4_success.preview_errors,
            stage_4_success.preview_confirmations,
            stage_4_success.preview_confirmations_recursive,
            Some(&stage_5),
        ),
        errors: Vec::new(),
    })

    // let (line_number_before_first_statement, statement_strs) =
    //     match text_to_statement_strs_with_error_info(text)? {
    //         Ok(tuple) => tuple,
    //         Err(detailed_err) => {
    //             return Ok(OnEditData {
    //                 page_data: None,
    //                 errors: vec![detailed_err],
    //             })
    //         }
    //     };

    // let (mmp_info_structured_for_unify_option, errors) =
    //     statement_strs_to_mmp_info_structured_for_unify_with_error_info(
    //         &statement_strs,
    //         mm_data,
    //         line_number_before_first_statement,
    //     )?;

    // let page_data = match mmp_info_structured_for_unify_option {
    //     Some(info) => Some(get_database_element_page_data(&info)?),
    //     None => None,
    // };

    // Ok(OnEditData { page_data, errors })
}

pub fn calc_theorem_page_data(
    mm_data: &MetamathData,
    stage_2_success: &MmpParserStage2Success,
    stage_3_theorem: MmpParserStage3Theorem,
    reference_numbers: Vec<Option<u32>>,
    preview_errors: Vec<(bool, bool, bool, bool)>,
    preview_confirmations: Vec<bool>,
    preview_confirmations_recursive: Vec<bool>,
    stage_5: Option<&MmpParserStage5>,
) -> Option<DatabaseElementPageData> {
    let (html_allowed_tags_and_attributes, css_allowed_properties) =
        html_validation::create_rule_structs();

    let mut proof_lines: Vec<model::ProofLine> = Vec::new();
    let mut preview_unify_markers: Vec<(bool, bool, bool, bool)> = Vec::new();

    for (i, ((pl, &indention), ref_number)) in stage_2_success
        .proof_lines
        .iter()
        .zip(stage_3_theorem.indention.iter())
        .zip(reference_numbers.into_iter())
        .enumerate()
    {
        let mut unify_markers = (false, false, false, false);

        let unify_result = stage_5.and_then(|s5| s5.unify_result.get(i));
        let unify_step_ref = unify_result.and_then(|ur| ur.step_ref.as_ref());

        proof_lines.push(model::ProofLine {
            step_name: pl.step_name.to_string(),
            hypotheses: if pl.hypotheses.len() != 0 {
                pl.hypotheses.split(',').map(|s| s.to_string()).collect()
            } else {
                Vec::new()
            },
            reference: if let Some(step_ref) = unify_step_ref {
                unify_markers.2 = true;
                step_ref.clone()
            } else {
                pl.step_ref.to_string()
            },
            reference_number: ref_number,
            assertion: util::str_to_space_seperated_string(pl.expression),
            indention,
        });
        preview_unify_markers.push(unify_markers);
    }

    let description = stage_2_success
        .comments
        .first()
        .map(|s| s.to_string())
        .unwrap_or(String::new());

    Some(DatabaseElementPageData::Theorem(TheoremPageData {
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

pub fn statement_strs_to_mmp_info_structured_for_unify_with_error_info<'a>(
    statement_strs: &Vec<&'a str>,
    mm_data: &MetamathData,
    line_offset: u32,
) -> Result<(Option<MmpInfoStructuredForUnify<'a>>, Vec<DetailedError>), Error> {
    let mut label: Option<MmpLabel<'a>> = None;
    let mut allow_discouraged: bool = false;
    let mut locate_after: Option<LocateAfterRef<'a>> = None;
    let mut distinct_vars: Vec<&'a str> = Vec::new();
    let mut constants: Option<&'a str> = None;
    let mut variables: Vec<&'a str> = Vec::new();
    let mut floating_hypotheses: Vec<&'a str> = Vec::new();
    let mut proof_lines: Vec<ProofLine<'a>> = Vec::new();
    let mut comments: Vec<&'a str> = Vec::new();
    let mut statements: Vec<MmpStatement> = Vec::with_capacity(statement_strs.len());

    let mut errors: Vec<DetailedError> = Vec::new();
    let mut return_info: bool = true;
    let mut current_line: u32 = line_offset;

    for &statement_str in statement_strs {
        let mut token_iter = statement_str.split_ascii_whitespace();

        let last_non_whitespace_pos = last_non_whitespace_pos(statement_str);

        match token_iter.next().ok_or(Error::InternalLogicError)? {
            "$c" => {
                if constants.is_some() {
                    return_info = false;

                    errors.push(DetailedError {
                        error_type: Error::TooManyConstStatementsError,
                        start_line_number: current_line,
                        start_column: 1,
                        end_line_number: current_line + last_non_whitespace_pos.0 - 1,
                        end_column: last_non_whitespace_pos.1 + 1,
                    })
                }

                constants = Some(
                    statement_str
                        .get(3..statement_str.len())
                        .ok_or(Error::InternalLogicError)?,
                );

                if token_iter.next().is_none() {
                    return_info = false;

                    errors.push(DetailedError {
                        error_type: Error::EmptyConstStatementError,
                        start_line_number: current_line,
                        start_column: 1,
                        end_line_number: current_line + last_non_whitespace_pos.0 - 1,
                        end_column: last_non_whitespace_pos.1 + 1,
                    })
                }

                statements.push(MmpStatement::Constant);
            }
            "$v" => {
                variables.push(
                    statement_str
                        .get(3..statement_str.len())
                        .ok_or(Error::InternalLogicError)?,
                );

                if token_iter.next().is_none() {
                    return_info = false;

                    errors.push(DetailedError {
                        error_type: Error::EmptyVarStatementError,
                        start_line_number: current_line,
                        start_column: 1,
                        end_line_number: current_line + last_non_whitespace_pos.0 - 1,
                        end_column: last_non_whitespace_pos.1 + 1,
                    })
                }

                statements.push(MmpStatement::Variable);
            }
            "$f" => {
                floating_hypotheses.push(
                    statement_str
                        .get(3..statement_str.len())
                        .ok_or(Error::InternalLogicError)?,
                );

                // token_iter should only have exactly three more token
                let first_token = token_iter.next();
                let second_token = token_iter.next();
                let third_token = token_iter.next();
                let fourth_token = token_iter.next();

                if first_token.is_none() {
                    return_info = false;

                    errors.push(DetailedError {
                        error_type: Error::FloatHypStatementFormatError,
                        start_line_number: current_line,
                        start_column: 1,
                        end_line_number: current_line,
                        end_column: 3, // Length of "$f" + 1
                    })
                } else if second_token.is_none() || third_token.is_none() {
                    return_info = false;
                    let second_token_start_pos = nth_token_start_pos(statement_str, 1);

                    errors.push(DetailedError {
                        error_type: Error::FloatHypStatementFormatError,
                        start_line_number: current_line + second_token_start_pos.0 - 1,
                        start_column: second_token_start_pos.1,
                        end_line_number: current_line + last_non_whitespace_pos.0 - 1,
                        end_column: last_non_whitespace_pos.1 + 1,
                    })
                } else if fourth_token.is_some() {
                    return_info = false;
                    let fifth_token_start_pos = nth_token_start_pos(statement_str, 4);

                    errors.push(DetailedError {
                        error_type: Error::FloatHypStatementFormatError,
                        start_line_number: current_line + fifth_token_start_pos.0 - 1,
                        start_column: fifth_token_start_pos.1,
                        end_line_number: current_line + last_non_whitespace_pos.0 - 1,
                        end_column: last_non_whitespace_pos.1 + 1,
                    })
                }

                statements.push(MmpStatement::FloatingHypohesis);
            }
            "$theorem" => {
                if label.is_some() {
                    return_info = false;

                    errors.push(DetailedError {
                        error_type: Error::MultipleMmpLabelsError,
                        start_line_number: current_line,
                        start_column: 1,
                        end_line_number: current_line + last_non_whitespace_pos.0 - 1,
                        end_column: last_non_whitespace_pos.1 + 1,
                    });
                }

                if let Some(theorem_label) = token_iter.next() {
                    label = Some(MmpLabel::Theorem(theorem_label));
                } else {
                    return_info = false;

                    errors.push(DetailedError {
                        error_type: Error::MissingTheoremLabelError,
                        start_line_number: current_line,
                        start_column: 1,
                        end_line_number: current_line,
                        end_column: 9, // Length of "$theorem" + 1
                    })
                }

                if token_iter.next().is_some() {
                    return_info = false;
                    let third_token_start_pos = nth_token_start_pos(statement_str, 2);

                    errors.push(DetailedError {
                        error_type: Error::TooManyTheoremLabelTokensError,
                        start_line_number: current_line + third_token_start_pos.0 - 1,
                        start_column: third_token_start_pos.1,
                        end_line_number: current_line + last_non_whitespace_pos.0 - 1,
                        end_column: last_non_whitespace_pos.1 + 1,
                    })
                }

                statements.push(MmpStatement::MmpLabel);
            }
            "$axiom" => {
                if label.is_some() {
                    return_info = false;

                    errors.push(DetailedError {
                        error_type: Error::MultipleMmpLabelsError,
                        start_line_number: current_line,
                        start_column: 1,
                        end_line_number: current_line + last_non_whitespace_pos.0 - 1,
                        end_column: last_non_whitespace_pos.1 + 1,
                    });
                }

                if let Some(axiom_label) = token_iter.next() {
                    label = Some(MmpLabel::Axiom(axiom_label));
                } else {
                    return_info = false;

                    errors.push(DetailedError {
                        error_type: Error::MissingAxiomLabelError,
                        start_line_number: current_line,
                        start_column: 1,
                        end_line_number: current_line,
                        end_column: 9, // Length of "$theorem" + 1
                    })
                }

                if token_iter.next().is_some() {
                    return_info = false;
                    let third_token_start_pos = nth_token_start_pos(statement_str, 2);

                    errors.push(DetailedError {
                        error_type: Error::TooManyAxiomLabelTokensError,
                        start_line_number: current_line + third_token_start_pos.0 - 1,
                        start_column: third_token_start_pos.1,
                        end_line_number: current_line + last_non_whitespace_pos.0 - 1,
                        end_column: last_non_whitespace_pos.1 + 1,
                    })
                }

                statements.push(MmpStatement::MmpLabel);
            }
            "$header" => {
                if label.is_some() {
                    return_info = false;

                    errors.push(DetailedError {
                        error_type: Error::MultipleMmpLabelsError,
                        start_line_number: current_line,
                        start_column: 1,
                        end_line_number: current_line + last_non_whitespace_pos.0 - 1,
                        end_column: last_non_whitespace_pos.1 + 1,
                    });
                }

                if let Some(header_pos) = token_iter.next() {
                    // make sure there follows at least one token
                    if token_iter.next().is_some() {
                        let statement_bytes = statement_str.as_bytes();
                        let mut statement_i: usize = 0;
                        while statement_bytes
                            .get(statement_i)
                            .is_some_and(|c| !c.is_ascii_whitespace())
                        {
                            statement_i += 1;
                        }
                        while statement_bytes
                            .get(statement_i)
                            .is_some_and(|c| c.is_ascii_whitespace())
                        {
                            statement_i += 1;
                        }
                        while statement_bytes
                            .get(statement_i)
                            .is_some_and(|c| !c.is_ascii_whitespace())
                        {
                            statement_i += 1;
                        }

                        let title = statement_str
                            .get((statement_i + 1)..statement_str.len())
                            .ok_or(Error::InternalLogicError)?;

                        label = Some(MmpLabel::Header { header_pos, title });

                        statements.push(MmpStatement::MmpLabel);
                    } else {
                        return_info = false;

                        errors.push(DetailedError {
                            error_type: Error::TooFewHeaderTokensError,
                            start_line_number: current_line,
                            start_column: 1,
                            end_line_number: current_line + last_non_whitespace_pos.0 - 1,
                            end_column: last_non_whitespace_pos.1 + 1,
                        });
                        // Make sure label is set to Some(_) so that future label statements will be flagged as errors
                        // Since return_info is false, the content within Some(_) does not matter
                        label = Some(MmpLabel::Theorem(""));
                    }
                } else {
                    return_info = false;

                    errors.push(DetailedError {
                        error_type: Error::TooFewHeaderTokensError,
                        start_line_number: current_line,
                        start_column: 1,
                        end_line_number: current_line + last_non_whitespace_pos.0 - 1,
                        end_column: last_non_whitespace_pos.1 + 1,
                    });
                    // Make sure label is set to Some(_) so that future label statements will be flagged as errors
                    // Since return_info is false, the content within Some(_) does not matter
                    label = Some(MmpLabel::Theorem(""));
                }
            }
            "$d" => {
                distinct_vars.push(
                    statement_str
                        .get(3..statement_str.len())
                        .ok_or(Error::InternalLogicError)?,
                );

                // make sure there are at least two more tokens
                if token_iter.next().is_none() || token_iter.next().is_none() {
                    return_info = false;

                    errors.push(DetailedError {
                        error_type: Error::ZeroOrOneSymbolDisjError,
                        start_line_number: current_line,
                        start_column: 1,
                        end_line_number: current_line + last_non_whitespace_pos.0 - 1,
                        end_column: last_non_whitespace_pos.1 + 1,
                    });
                }

                statements.push(MmpStatement::DistinctVar);
            }
            "$allowdiscouraged" => {
                if allow_discouraged {
                    return_info = false;

                    errors.push(DetailedError {
                        error_type: Error::MultipleAllowDiscouragedError,
                        start_line_number: current_line,
                        start_column: 1,
                        end_line_number: current_line + last_non_whitespace_pos.0 - 1,
                        end_column: last_non_whitespace_pos.1 + 1,
                    });
                }

                allow_discouraged = true;

                if token_iter.next().is_some() {
                    return_info = false;
                    let second_token_start_pos = nth_token_start_pos(statement_str, 1);

                    errors.push(DetailedError {
                        error_type: Error::TokensAfterAllowDiscouragedError,
                        start_line_number: current_line + second_token_start_pos.0 - 1,
                        start_column: second_token_start_pos.1,
                        end_line_number: current_line + last_non_whitespace_pos.0 - 1,
                        end_column: last_non_whitespace_pos.1 + 1,
                    });
                }

                statements.push(MmpStatement::AllowDiscouraged);
            }
            "$locateafter" => {
                if locate_after.is_some() {
                    return_info = false;

                    errors.push(DetailedError {
                        error_type: Error::MultipleLocateAfterError,
                        start_line_number: current_line,
                        start_column: 1,
                        end_line_number: current_line + last_non_whitespace_pos.0 - 1,
                        end_column: last_non_whitespace_pos.1 + 1,
                    });
                }

                if let Some(locate_after_label) = token_iter.next() {
                    locate_after = Some(LocateAfterRef::LocateAfter(locate_after_label));
                } else {
                    return_info = false;

                    errors.push(DetailedError {
                        error_type: Error::TooFewLocateAfterTokensError,
                        start_line_number: current_line,
                        start_column: 1,
                        end_line_number: current_line + last_non_whitespace_pos.0 - 1,
                        end_column: last_non_whitespace_pos.1 + 1,
                    });

                    // Make sure locate_after is set to Some(_) so that future locate-after statements will be flagged as errors
                    // Since return_info is false, the content within Some(_) does not matter
                    locate_after = Some(LocateAfterRef::LocateAfter(""));
                }

                if token_iter.next().is_some() {
                    return_info = false;
                    let third_token_start_pos = nth_token_start_pos(statement_str, 2);

                    errors.push(DetailedError {
                        error_type: Error::TooManyLocateAfterTokensError,
                        start_line_number: current_line + third_token_start_pos.0 - 1,
                        start_column: third_token_start_pos.1,
                        end_line_number: current_line + last_non_whitespace_pos.0 - 1,
                        end_column: last_non_whitespace_pos.1 + 1,
                    });

                    // Make sure locate_after is set to Some(_) so that future locate-after statements will be flagged as errors
                    // Since return_info is false, the content within Some(_) does not matter
                    locate_after = Some(LocateAfterRef::LocateAfter(""));
                }

                statements.push(MmpStatement::LocateAfter);
            }
            "$locateafterconst" => {
                if locate_after.is_some() {
                    return_info = false;

                    errors.push(DetailedError {
                        error_type: Error::MultipleLocateAfterError,
                        start_line_number: current_line,
                        start_column: 1,
                        end_line_number: current_line + last_non_whitespace_pos.0 - 1,
                        end_column: last_non_whitespace_pos.1 + 1,
                    });
                }

                if let Some(locate_after_constant) = token_iter.next() {
                    locate_after = Some(LocateAfterRef::LocateAfterConst(locate_after_constant));
                } else {
                    return_info = false;

                    errors.push(DetailedError {
                        error_type: Error::TooFewLocateAfterConstTokensError,
                        start_line_number: current_line,
                        start_column: 1,
                        end_line_number: current_line + last_non_whitespace_pos.0 - 1,
                        end_column: last_non_whitespace_pos.1 + 1,
                    });

                    // Make sure locate_after is set to Some(_) so that future locate-after statements will be flagged as errors
                    // Since return_info is false, the content within Some(_) does not matter
                    locate_after = Some(LocateAfterRef::LocateAfter(""));
                }

                if token_iter.next().is_some() {
                    return_info = false;
                    let third_token_start_pos = nth_token_start_pos(statement_str, 2);

                    errors.push(DetailedError {
                        error_type: Error::TooManyLocateAfterConstTokensError,
                        start_line_number: current_line + third_token_start_pos.0 - 1,
                        start_column: third_token_start_pos.1,
                        end_line_number: current_line + last_non_whitespace_pos.0 - 1,
                        end_column: last_non_whitespace_pos.1 + 1,
                    });

                    // Make sure locate_after is set to Some(_) so that future locate-after statements will be flagged as errors
                    // Since return_info is false, the content within Some(_) does not matter
                    locate_after = Some(LocateAfterRef::LocateAfter(""));
                }

                statements.push(MmpStatement::LocateAfter);
            }
            "$locateaftervar" => {
                if locate_after.is_some() {
                    return_info = false;

                    errors.push(DetailedError {
                        error_type: Error::MultipleLocateAfterError,
                        start_line_number: current_line,
                        start_column: 1,
                        end_line_number: current_line + last_non_whitespace_pos.0 - 1,
                        end_column: last_non_whitespace_pos.1 + 1,
                    });
                }

                if let Some(locate_after_variable) = token_iter.next() {
                    locate_after = Some(LocateAfterRef::LocateAfterConst(locate_after_variable));
                } else {
                    return_info = false;

                    errors.push(DetailedError {
                        error_type: Error::TooFewLocateAfterVarTokensError,
                        start_line_number: current_line,
                        start_column: 1,
                        end_line_number: current_line + last_non_whitespace_pos.0 - 1,
                        end_column: last_non_whitespace_pos.1 + 1,
                    });

                    // Make sure locate_after is set to Some(_) so that future locate-after statements will be flagged as errors
                    // Since return_info is false, the content within Some(_) does not matter
                    locate_after = Some(LocateAfterRef::LocateAfter(""));
                }

                if token_iter.next().is_some() {
                    return_info = false;
                    let third_token_start_pos = nth_token_start_pos(statement_str, 2);

                    errors.push(DetailedError {
                        error_type: Error::TooManyLocateAfterVarTokensError,
                        start_line_number: current_line + third_token_start_pos.0 - 1,
                        start_column: third_token_start_pos.1,
                        end_line_number: current_line + last_non_whitespace_pos.0 - 1,
                        end_column: last_non_whitespace_pos.1 + 1,
                    });

                    // Make sure locate_after is set to Some(_) so that future locate-after statements will be flagged as errors
                    // Since return_info is false, the content within Some(_) does not matter
                    locate_after = Some(LocateAfterRef::LocateAfter(""));
                }

                statements.push(MmpStatement::LocateAfter);
            }
            t if t.starts_with('*') => {
                statements.push(MmpStatement::Comment);
                comments.push(
                    statement_str
                        .get(1..statement_str.len())
                        .ok_or(Error::InternalLogicError)?,
                );
            }
            t if t.starts_with('$') => {
                let first_token_end_pos = nth_token_end_pos(statement_str, 0);

                errors.push(DetailedError {
                    error_type: Error::InvalidDollarTokenError,
                    start_line_number: current_line,
                    start_column: 1,
                    end_line_number: current_line,
                    end_column: first_token_end_pos.1 + 1,
                });
            }
            step_prefix => {
                let prefix_parts: Vec<&str> = step_prefix.split(':').collect();
                if prefix_parts.len() != 3 {
                    return_info = false;
                    let first_token_end_pos = nth_token_end_pos(statement_str, 0);

                    errors.push(DetailedError {
                        error_type: Error::InvalidMmpStepPrefixFormatError,
                        start_line_number: current_line,
                        start_column: 1,
                        end_line_number: current_line + first_token_end_pos.0 - 1,
                        end_column: first_token_end_pos.1 + 1,
                    });
                } else {
                    let prefix_step_name = prefix_parts.get(0).unwrap();

                    let mut is_hypothesis = false;
                    let step_name: &str;

                    if prefix_step_name.starts_with('h') {
                        is_hypothesis = true;
                        step_name = prefix_step_name.split_at(1).1;
                    } else {
                        step_name = prefix_step_name;
                    }

                    if step_name.contains(',') || step_name == "" {
                        return_info = false;

                        errors.push(DetailedError {
                            error_type: Error::InvalidMmpStepNameError,
                            start_line_number: current_line,
                            start_column: 1 + is_hypothesis as u32,
                            end_line_number: current_line,
                            end_column: is_hypothesis as u32 + step_name.len() as u32 + 1,
                        });
                    }

                    let hypotheses = *prefix_parts.get(1).unwrap();

                    let mut hypotheses_parsed: Vec<Option<usize>> = Vec::new();

                    if !hypotheses.is_empty() {
                        let mut start_column =
                            1 + is_hypothesis as u32 + step_name.len() as u32 + 1;
                        for hyp in hypotheses.split(',') {
                            if hyp == "?" {
                                hypotheses_parsed.push(None);
                            } else {
                                match proof_lines
                                    .iter()
                                    .enumerate()
                                    .find(|(_, pl)| pl.step_name == hyp)
                                {
                                    Some((i, _)) => hypotheses_parsed.push(Some(i)),
                                    None => {
                                        return_info = false;

                                        errors.push(DetailedError {
                                            error_type: Error::HypNameDoesntExistError,
                                            start_line_number: current_line,
                                            start_column: start_column,
                                            end_line_number: current_line,
                                            end_column: start_column + hyp.len() as u32,
                                        });
                                    }
                                }
                            }
                            start_column += hyp.len() as u32 + 1;
                        }
                    }

                    let step_ref = *prefix_parts.get(2).unwrap();

                    let expression = statement_str
                        .get(step_prefix.len()..statement_str.len())
                        .ok_or(Error::InternalLogicError)?;

                    // if token_iter.next().is_none() {
                    //     errors.push(DetailedError {
                    //         error_type: Error::MissingMmpStepExpressionError,
                    //         start_line_number: current_line,
                    //         start_column: last_non_whitespace_pos.1,
                    //         end_line_number: current_line,
                    //         end_column: last_non_whitespace_pos.1 + 1,
                    //     });
                    // }

                    // This will be overritten if expression is convertable to a parse tree
                    let mut parse_tree = ParseTree {
                        typecode: 0,
                        top_node: ParseTreeNode::Node {
                            rule_i: 0,
                            sub_nodes: Vec::new(),
                        },
                    };

                    match mm_data.expression_to_parse_tree(expression) {
                        Ok(pt) => parse_tree = pt,
                        Err(Error::MissingExpressionError) => {
                            errors.push(DetailedError {
                                error_type: Error::MissingMmpStepExpressionError,
                                start_line_number: current_line,
                                start_column: last_non_whitespace_pos.1 + 1,
                                end_line_number: current_line,
                                end_column: last_non_whitespace_pos.1 + 2,
                            });
                        }
                        Err(Error::NonSymbolInExpressionError) => {
                            errors.append(&mut calc_non_symbol_in_expression_errors(
                                expression,
                                &mm_data.optimized_data.symbol_number_mapping,
                                current_line,
                                step_prefix.len() as u32,
                            ));
                        }
                        Err(Error::ExpressionParseError) => {
                            errors.push(DetailedError {
                                error_type: Error::ExpressionParseError,
                                start_line_number: current_line,
                                start_column: last_non_whitespace_pos.1 + 1,
                                end_line_number: current_line,
                                end_column: last_non_whitespace_pos.1 + 2,
                            });
                        }
                        Err(_) => {
                            return Err(Error::InternalLogicError);
                        }
                    };

                    statements.push(MmpStatement::ProofLine);
                    proof_lines.push(ProofLine {
                        is_hypothesis,
                        step_name,
                        hypotheses,
                        hypotheses_parsed,
                        step_ref,
                        expression,
                        parse_tree,
                    });
                }
            }
        }

        current_line += new_lines_in_str(statement_str);
    }

    Ok((
        if return_info {
            Some(MmpInfoStructuredForUnify {
                label,
                allow_discouraged,
                locate_after,
                distinct_vars,
                constants,
                variables,
                floating_hypotheses,
                proof_lines,
                comments,
                statements,
            })
        } else {
            None
        },
        errors,
    ))
}

// Returns (a, b), where a is the line number and b is the column number of the last non-whitespace character
fn last_non_whitespace_pos(str: &str) -> (u32, u32) {
    let mut last_non_whitespace_line_number = 1;
    let mut last_non_whitespace_column_number = 1;

    let mut line_number = 1;
    let mut column_number = 0;

    for char in str.chars() {
        column_number += 1;

        if char == '\n' {
            line_number += 1;
            column_number = 0;
        }

        if !char.is_whitespace() {
            last_non_whitespace_line_number = line_number;
            last_non_whitespace_column_number = column_number;
        }
    }

    (
        last_non_whitespace_line_number,
        last_non_whitespace_column_number,
    )
}

fn nth_token_start_pos(str: &str, n: u32) -> (u32, u32) {
    let mut tokens_seen = 0;
    let mut seeing_token = false;

    let mut line_number = 1;
    let mut column_number = 0;

    for char in str.chars() {
        column_number += 1;

        if char == '\n' {
            line_number += 1;
            column_number = 0;
        }

        if char.is_whitespace() {
            if seeing_token {
                tokens_seen += 1;
            }

            seeing_token = false;
        } else {
            if tokens_seen == n {
                break;
            }

            seeing_token = true;
        }
    }

    (line_number, column_number)
}

fn nth_token_end_pos(str: &str, n: u32) -> (u32, u32) {
    let mut tokens_seen = 0;
    let mut seeing_token = false;

    let mut line_number = 1;
    let mut column_number = 0;

    for char in str.chars() {
        column_number += 1;

        if char.is_whitespace() {
            if tokens_seen == n {
                column_number -= 1;
                break;
            }

            if seeing_token {
                tokens_seen += 1;
            }

            seeing_token = false;
        } else {
            seeing_token = true;
        }

        if char == '\n' {
            line_number += 1;
            column_number = 0;
        }
    }

    (line_number, column_number)
}

fn new_lines_in_str(str: &str) -> u32 {
    str.chars().filter(|c| *c == '\n').count() as u32
}

fn calc_non_symbol_in_expression_errors(
    expression: &str,
    symbol_number_mapping: &SymbolNumberMapping,
    first_line: u32,
    first_line_offset: u32,
) -> Vec<DetailedError> {
    let mut errors = Vec::new();

    let mut line = first_line;
    let mut column = first_line_offset;

    let mut current_token_start_column = column;

    let mut current_token = String::new();

    let mut seeing_token = false;

    for char in expression.chars() {
        column += 1;

        if char == '\n' {
            line += 1;
            column = 0;
        }

        if char.is_ascii_whitespace() {
            if seeing_token {
                if current_token.starts_with('$')
                    || symbol_number_mapping.numbers.get(&current_token).is_none()
                {
                    errors.push(DetailedError {
                        error_type: Error::NonSymbolInExpressionError,
                        start_line_number: line,
                        start_column: current_token_start_column,
                        end_line_number: line,
                        end_column: column,
                    });
                }

                current_token = String::new();
            }
            seeing_token = false;
        } else {
            if !seeing_token {
                current_token_start_column = column;
            }
            seeing_token = true;
            current_token.push(char)
        }
    }

    errors
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
        // proof_lines.push(model::ProofLine {
        //     indention: 1,
        //     assertion: util::str_to_space_seperated_string(proof_line.expression),
        //     hypotheses: proof_line
        //         .hypotheses_parsed
        //         .iter()
        //         .map(|hyp| match hyp {
        //             Some(num) => *num as i32 + 1,
        //             None => 0,
        //         })
        //         .collect(),
        //     reference: proof_line.step_ref.to_string(),
        // });

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
        preview_errors: None,
        preview_confirmations: None,
        preview_confirmations_recursive: None,
        preview_unify_markers: None,
        last_theorem_label: None,
        next_theorem_label: None,
        axiom_dependencies: Vec::new(),
        definition_dependencies: Vec::new(),
        references: Vec::new(),
        description_parsed: Vec::new(),
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
