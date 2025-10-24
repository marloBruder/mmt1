use std::collections::HashSet;

use crate::{
    editor::on_edit::DetailedError,
    metamath::mmp_parser::LocateAfterRef,
    model::{
        Comment, Constant, DatabaseElement, FloatingHypothesis, HeaderPath, MetamathData,
        Statement, Theorem, Variable,
    },
    util, Error,
};

use super::{
    MmpLabel, MmpParserStage1Success, MmpParserStage2Success, MmpParserStage3,
    MmpParserStage3Comment, MmpParserStage3Fail, MmpParserStage3Header, MmpParserStage3Success,
    MmpParserStage3Theorem, MmpStatement, ProofLine,
};

pub fn stage_3<'a>(
    stage_1: &MmpParserStage1Success<'a>,
    stage_2: &MmpParserStage2Success<'a>,
    mm_data: &MetamathData,
) -> Result<MmpParserStage3<'a>, Error> {
    match stage_2.label {
        Some(MmpLabel::Header { header_path, title }) => {
            stage_3_header(stage_1, stage_2, mm_data, header_path, title)
        }
        Some(MmpLabel::Axiom(axiom_label)) => {
            stage_3_theorem(stage_1, stage_2, axiom_label, true, mm_data)
        }
        Some(MmpLabel::Theorem(theorem_label)) => {
            stage_3_theorem(stage_1, stage_2, theorem_label, false, mm_data)
        }
        None => stage_3_no_label(stage_1, stage_2, mm_data),
    }
}

fn stage_3_header<'a>(
    stage_1: &MmpParserStage1Success<'a>,
    stage_2: &MmpParserStage2Success<'a>,
    mm_data: &MetamathData,
    header_path: &'a str,
    title: &'a str,
) -> Result<MmpParserStage3<'a>, Error> {
    let mut errors: Vec<DetailedError> =
        calc_header_statement_out_of_place_errors(stage_1, stage_2);

    let mut parent_header_path =
        HeaderPath::from_str(header_path).ok_or(Error::InternalLogicError)?;
    let header_i = parent_header_path
        .path
        .pop()
        .ok_or(Error::InternalLogicError)?;

    if parent_header_path
        .resolve(&mm_data.database_header)
        // Allow header_i == len()
        .is_none_or(|parent_header| parent_header.subheaders.len() < header_i)
    {
        errors.push(calc_label_error(
            stage_1,
            stage_2,
            Error::InvalidNewHeaderPathError,
        )?);
    }

    Ok(if errors.is_empty() {
        MmpParserStage3::Success(MmpParserStage3Success::Header(MmpParserStage3Header {
            parent_header_path,
            header_i,
            title: util::str_to_space_seperated_string(title),
            description: stage_2.comments.first().unwrap_or(&"").to_string(),
        }))
    } else {
        MmpParserStage3::Fail(MmpParserStage3Fail { errors })
    })
}

fn calc_statement_out_of_place_errors(
    errors: &mut Vec<DetailedError>,
    stage_1: &MmpParserStage1Success,
    stage_2: &MmpParserStage2Success,
    error_type: Error,
    out_of_place_statement_type: MmpStatement,
) {
    for (&statement_str, (statement_type, line_number)) in
        stage_1.statements.iter().zip(&stage_2.statements)
    {
        if *statement_type == out_of_place_statement_type {
            let last_non_whitespace_pos = util::last_non_whitespace_pos(statement_str);

            errors.push(DetailedError {
                error_type,
                start_line_number: *line_number,
                start_column: 1,
                end_line_number: *line_number + last_non_whitespace_pos.0 - 1,
                end_column: last_non_whitespace_pos.1 + 1,
            });
        }
    }
}

fn calc_header_statement_out_of_place_errors(
    stage_1: &MmpParserStage1Success,
    stage_2: &MmpParserStage2Success,
) -> Vec<DetailedError> {
    let mut errors: Vec<DetailedError> = Vec::new();

    if stage_2.constants.is_some() {
        calc_statement_out_of_place_errors(
            &mut errors,
            stage_1,
            stage_2,
            Error::ConstStatementOutOfPlaceError,
            MmpStatement::Constant,
        );
    }
    if !stage_2.variables.is_empty() {
        calc_statement_out_of_place_errors(
            &mut errors,
            stage_1,
            stage_2,
            Error::VarStatementOutOfPlaceError,
            MmpStatement::Variable,
        );
    }
    if !stage_2.floating_hypotheses.is_empty() {
        calc_statement_out_of_place_errors(
            &mut errors,
            stage_1,
            stage_2,
            Error::FloatHypStatementOutOfPlaceError,
            MmpStatement::FloatingHypohesis,
        );
    }
    if stage_2.allow_discouraged {
        calc_statement_out_of_place_errors(
            &mut errors,
            stage_1,
            stage_2,
            Error::AllowDiscouragedOutOfPlaceError,
            MmpStatement::AllowDiscouraged,
        );
    }
    if stage_2.allow_incomplete {
        calc_statement_out_of_place_errors(
            &mut errors,
            stage_1,
            stage_2,
            Error::AllowIncompleteOutOfPlaceError,
            MmpStatement::AllowIncomplete,
        );
    }
    if stage_2.locate_after.is_some() {
        calc_statement_out_of_place_errors(
            &mut errors,
            stage_1,
            stage_2,
            Error::LocateAfterOutOfPlaceError,
            MmpStatement::LocateAfter,
        );
    }
    if !stage_2.distinct_vars.is_empty() {
        calc_statement_out_of_place_errors(
            &mut errors,
            stage_1,
            stage_2,
            Error::DistinctVarOutOfPlaceError,
            MmpStatement::DistinctVar,
        );
    }
    if !stage_2.proof_lines.is_empty() {
        calc_statement_out_of_place_errors(
            &mut errors,
            stage_1,
            stage_2,
            Error::ProofLinesOutOfPlaceError,
            MmpStatement::ProofLine,
        );
    }

    errors
}

fn calc_label_error(
    stage_1: &MmpParserStage1Success,
    stage_2: &MmpParserStage2Success,
    error_type: Error,
) -> Result<DetailedError, Error> {
    for (&statement_str, (statement_type, line_number)) in
        stage_1.statements.iter().zip(&stage_2.statements)
    {
        if *statement_type == MmpStatement::MmpLabel {
            let first_token_start_pos = util::nth_token_start_pos(statement_str, 1);
            let first_token_end_pos = util::nth_token_end_pos(statement_str, 1);

            return Ok(DetailedError {
                error_type,
                start_line_number: *line_number + first_token_start_pos.0 - 1,
                start_column: first_token_start_pos.1,
                end_line_number: *line_number + first_token_end_pos.0 - 1,
                end_column: first_token_end_pos.1 + 1,
            });
        }
    }

    Err(Error::InternalLogicError)
}

fn stage_3_theorem<'a>(
    stage_1: &MmpParserStage1Success<'a>,
    stage_2: &MmpParserStage2Success<'a>,
    label: &'a str,
    is_axiom: bool,
    metamath_data: &MetamathData,
) -> Result<MmpParserStage3<'a>, Error> {
    let mut errors: Vec<DetailedError> = Vec::new();

    if stage_2.constants.is_some() {
        calc_statement_out_of_place_errors(
            &mut errors,
            stage_1,
            stage_2,
            Error::ConstStatementOutOfPlaceError,
            MmpStatement::Constant,
        );
    }

    let temp_variable_statements: Vec<Vec<Variable>> = stage_2
        .variables
        .iter()
        .map(|var_statement_str| {
            var_statement_str
                .split_ascii_whitespace()
                .map(|var| Variable {
                    symbol: var.to_string(),
                })
                .collect()
        })
        .collect();

    let mut temp_floating_hypotheses = Vec::new();

    for (i, float_hyp_str) in stage_2.floating_hypotheses.iter().enumerate() {
        let mut flaot_hyp_iter = float_hyp_str.split_ascii_whitespace();

        let label = flaot_hyp_iter
            .next()
            .ok_or(Error::InternalLogicError)?
            .to_string();
        let typecode = flaot_hyp_iter
            .next()
            .ok_or(Error::InternalLogicError)?
            .to_string();
        let variable = flaot_hyp_iter
            .next()
            .ok_or(Error::InternalLogicError)?
            .to_string();

        let Some((statement_str, line_number)) = calc_statement_str_and_line_number(
            &stage_1.statements,
            &stage_2.statements,
            MmpStatement::FloatingHypohesis,
            i,
        ) else {
            return Err(Error::InternalLogicError);
        };

        if !metamath_data.symbols_not_already_taken(&vec![&label]) {
            let second_token_start_pos = util::nth_token_start_pos(statement_str, 1);
            let second_token_end_pos = util::nth_token_end_pos(statement_str, 1);

            errors.push(DetailedError {
                error_type: Error::LabelAlreadyExistsError,
                start_line_number: line_number + second_token_start_pos.0 - 1,
                start_column: second_token_start_pos.1,
                end_line_number: line_number + second_token_end_pos.0 - 1,
                end_column: second_token_end_pos.1 + 1,
            });
        }

        if metamath_data
            .database_header
            .constant_locate_after_iter(stage_2.locate_after)
            .all(|c| c.symbol != typecode)
        {
            let third_token_start_pos = util::nth_token_start_pos(statement_str, 2);
            let third_token_end_pos = util::nth_token_end_pos(statement_str, 2);

            errors.push(DetailedError {
                error_type: Error::TypecodeNotAConstantError,
                start_line_number: line_number + third_token_start_pos.0 - 1,
                start_column: third_token_start_pos.1,
                end_line_number: line_number + third_token_end_pos.0 - 1,
                end_column: third_token_end_pos.1 + 1,
            });
        }

        if metamath_data
            .database_header
            .variable_locate_after_iter(stage_2.locate_after)
            .chain(temp_variable_statements.iter().flatten())
            .all(|v| v.symbol != variable)
        {
            let fourth_token_start_pos = util::nth_token_start_pos(statement_str, 3);
            let fourth_token_end_pos = util::nth_token_end_pos(statement_str, 3);

            errors.push(DetailedError {
                error_type: Error::ExpectedActiveVariableError,
                start_line_number: line_number + fourth_token_start_pos.0 - 1,
                start_column: fourth_token_start_pos.1,
                end_line_number: line_number + fourth_token_end_pos.0 - 1,
                end_column: fourth_token_end_pos.1 + 1,
            });
        }

        if metamath_data
            .database_header
            .floating_hypohesis_iter()
            .chain(temp_floating_hypotheses.iter())
            .any(|fh| fh.variable == variable && fh.label != label)
        {
            let fourth_token_start_pos = util::nth_token_start_pos(statement_str, 3);
            let fourth_token_end_pos = util::nth_token_end_pos(statement_str, 3);

            errors.push(DetailedError {
                error_type: Error::VariableTypecodeAlreadyDeclaredError,
                start_line_number: line_number + fourth_token_start_pos.0 - 1,
                start_column: fourth_token_start_pos.1,
                end_line_number: line_number + fourth_token_end_pos.0 - 1,
                end_column: fourth_token_end_pos.1 + 1,
            });
        }

        temp_floating_hypotheses.push(FloatingHypothesis {
            label,
            typecode,
            variable,
        });
    }

    if let Some(locate_after) = stage_2.locate_after {
        let (statement_str, line_number) = calc_statement_str_and_line_number(
            &stage_1.statements,
            &stage_2.statements,
            MmpStatement::LocateAfter,
            0,
        )
        .ok_or(Error::InternalLogicError)?;

        calc_locate_after_errors(
            &mut errors,
            locate_after,
            metamath_data,
            statement_str,
            line_number,
        )?;
    }

    let (axiom_dependencies, definition_dependencies) =
        calc_dependencies(&stage_2.proof_lines, metamath_data);

    Ok(if errors.is_empty() {
        MmpParserStage3::Success(MmpParserStage3Success::Theorem(MmpParserStage3Theorem {
            is_axiom,
            label,
            temp_variable_statements,
            temp_floating_hypotheses,
            axiom_dependencies,
            definition_dependencies,
        }))
    } else {
        MmpParserStage3::Fail(MmpParserStage3Fail { errors })
    })
}

fn calc_locate_after_errors(
    errors: &mut Vec<DetailedError>,
    locate_after: LocateAfterRef,
    metamath_data: &MetamathData,
    statement_str: &str,
    line_number: u32,
) -> Result<(), Error> {
    match locate_after {
        LocateAfterRef::LocateAfterStart => {}
        LocateAfterRef::LocateAfterHeader(header_path) => {
            if HeaderPath::from_str(header_path)
                .ok_or(Error::InternalLogicError)?
                .resolve(&metamath_data.database_header)
                .is_none()
            {
                let second_token_start_pos = util::nth_token_start_pos(statement_str, 1);
                let second_token_end_pos = util::nth_token_end_pos(statement_str, 1);

                errors.push(DetailedError {
                    error_type: Error::InvalidHeaderPathError,
                    start_line_number: line_number + second_token_start_pos.0 - 1,
                    start_column: second_token_start_pos.1,
                    end_line_number: line_number + second_token_end_pos.0 - 1,
                    end_column: second_token_end_pos.1 + 1,
                });
            }
        }
        LocateAfterRef::LocateAfterComment(comment_path) => {
            let (header_path, comment_i_str) = comment_path
                .split_once('#')
                .ok_or(Error::InternalLogicError)?;
            let comment_i = comment_i_str
                .parse::<usize>()
                .map_err(|_| Error::InternalLogicError)?;

            if HeaderPath::from_str(header_path)
                .ok_or(Error::InternalLogicError)?
                .resolve(&metamath_data.database_header)
                .is_none_or(|header| {
                    header
                        .content
                        .iter()
                        .filter(|s| matches!(s, Statement::CommentStatement(_)))
                        .count()
                        < comment_i
                })
            {
                let second_token_start_pos = util::nth_token_start_pos(statement_str, 1);
                let second_token_end_pos = util::nth_token_end_pos(statement_str, 1);

                errors.push(DetailedError {
                    error_type: Error::InvalidCommentPathError,
                    start_line_number: line_number + second_token_start_pos.0 - 1,
                    start_column: second_token_start_pos.1,
                    end_line_number: line_number + second_token_end_pos.0 - 1,
                    end_column: second_token_end_pos.1 + 1,
                });
            }
        }
        LocateAfterRef::LocateAfterConst(const_symbol) => {
            if metamath_data
                .database_header
                .constant_iter()
                .all(|c| c.symbol != const_symbol)
            {
                let second_token_start_pos = util::nth_token_start_pos(statement_str, 1);
                let second_token_end_pos = util::nth_token_end_pos(statement_str, 1);

                errors.push(DetailedError {
                    error_type: Error::NotAConstantError,
                    start_line_number: line_number + second_token_start_pos.0 - 1,
                    start_column: second_token_start_pos.1,
                    end_line_number: line_number + second_token_end_pos.0 - 1,
                    end_column: second_token_end_pos.1 + 1,
                });
            }
        }
        LocateAfterRef::LocateAfterVar(var_symbol) => {
            if metamath_data
                .database_header
                .variable_iter()
                .all(|v| v.symbol != var_symbol)
            {
                let second_token_start_pos = util::nth_token_start_pos(statement_str, 1);
                let second_token_end_pos = util::nth_token_end_pos(statement_str, 1);

                errors.push(DetailedError {
                    error_type: Error::NotAVariableError,
                    start_line_number: line_number + second_token_start_pos.0 - 1,
                    start_column: second_token_start_pos.1,
                    end_line_number: line_number + second_token_end_pos.0 - 1,
                    end_column: second_token_end_pos.1 + 1,
                });
            }
        }
        LocateAfterRef::LocateAfter(label) => {
            if metamath_data
                .database_header
                .floating_hypohesis_iter()
                .all(|fh| fh.label != label)
                && metamath_data
                    .database_header
                    .theorem_iter()
                    .all(|t| t.label != label)
            {
                let second_token_start_pos = util::nth_token_start_pos(statement_str, 1);
                let second_token_end_pos = util::nth_token_end_pos(statement_str, 1);

                errors.push(DetailedError {
                    error_type: Error::NotAValidLabelError,
                    start_line_number: line_number + second_token_start_pos.0 - 1,
                    start_column: second_token_start_pos.1,
                    end_line_number: line_number + second_token_end_pos.0 - 1,
                    end_column: second_token_end_pos.1 + 1,
                });
            }
        }
    }

    Ok(())
}

fn calc_dependencies(
    proof_lines: &Vec<ProofLine>,
    metamath_data: &MetamathData,
) -> (Vec<(String, u32)>, Vec<(String, u32)>) {
    let mut already_seen: HashSet<&str> = HashSet::new();

    let step_refs: Vec<&str> = proof_lines
        .iter()
        .filter_map(|pl| {
            if pl.is_hypothesis {
                None
            } else {
                if already_seen.contains(&pl.step_ref) {
                    None
                } else {
                    already_seen.insert(pl.step_ref);
                    Some(pl.step_ref)
                }
            }
        })
        .collect();

    let (axiom_theorem_indexes, definition_theorem_indexes) =
        Theorem::calc_dependencies_from_labels(&step_refs, &metamath_data.optimized_data);

    (
        metamath_data
            .database_header
            .theorem_i_vec_to_theorem_label_vec(&axiom_theorem_indexes)
            // Should never be the case
            .unwrap_or(Vec::new()),
        metamath_data
            .database_header
            .theorem_i_vec_to_theorem_label_vec(&definition_theorem_indexes)
            // Should never be the case
            .unwrap_or(Vec::new()),
    )
}

fn stage_3_no_label<'a>(
    stage_1: &MmpParserStage1Success<'a>,
    stage_2: &MmpParserStage2Success<'a>,
    mm_data: &MetamathData,
) -> Result<MmpParserStage3<'a>, Error> {
    if !stage_2.floating_hypotheses.is_empty() {
        stage_3_floating_hypothesis(stage_1, stage_2, mm_data)
    } else if !stage_2.variables.is_empty() {
        stage_3_variables(stage_1, stage_2, mm_data)
    } else if stage_2.constants.is_some() {
        stage_3_constants(stage_1, stage_2, mm_data)
    } else if !stage_2.comments.is_empty() {
        stage_3_comment(stage_1, stage_2, mm_data)
    } else {
        Ok(MmpParserStage3::Success(MmpParserStage3Success::Empty))
    }
}

fn stage_3_floating_hypothesis<'a>(
    stage_1: &MmpParserStage1Success<'a>,
    stage_2: &MmpParserStage2Success<'a>,
    mm_data: &MetamathData,
) -> Result<MmpParserStage3<'a>, Error> {
    let mut errors: Vec<DetailedError> = Vec::new();

    if stage_2.floating_hypotheses.len() > 1 {
        calc_statement_out_of_place_errors(
            &mut errors,
            stage_1,
            stage_2,
            Error::FloatHypStatementOutOfPlaceError,
            MmpStatement::Constant,
        );
        // The first flaoting hypothesis should not be marked as an error
        errors.swap_remove(0);
    }
    if stage_2.constants.is_some() {
        calc_statement_out_of_place_errors(
            &mut errors,
            stage_1,
            stage_2,
            Error::ConstStatementOutOfPlaceError,
            MmpStatement::Constant,
        );
    }
    if !stage_2.variables.is_empty() {
        calc_statement_out_of_place_errors(
            &mut errors,
            stage_1,
            stage_2,
            Error::VarStatementOutOfPlaceError,
            MmpStatement::Variable,
        );
    }
    if stage_2.allow_discouraged {
        calc_statement_out_of_place_errors(
            &mut errors,
            stage_1,
            stage_2,
            Error::AllowDiscouragedOutOfPlaceError,
            MmpStatement::AllowDiscouraged,
        );
    }
    if stage_2.allow_incomplete {
        calc_statement_out_of_place_errors(
            &mut errors,
            stage_1,
            stage_2,
            Error::AllowIncompleteOutOfPlaceError,
            MmpStatement::AllowIncomplete,
        );
    }
    if !stage_2.distinct_vars.is_empty() {
        calc_statement_out_of_place_errors(
            &mut errors,
            stage_1,
            stage_2,
            Error::DistinctVarOutOfPlaceError,
            MmpStatement::DistinctVar,
        );
    }
    if !stage_2.proof_lines.is_empty() {
        calc_statement_out_of_place_errors(
            &mut errors,
            stage_1,
            stage_2,
            Error::ProofLinesOutOfPlaceError,
            MmpStatement::ProofLine,
        );
    }

    if let Some(locate_after) = stage_2.locate_after {
        let (statement_str, line_number) = calc_statement_str_and_line_number(
            &stage_1.statements,
            &stage_2.statements,
            MmpStatement::LocateAfter,
            0,
        )
        .ok_or(Error::InternalLogicError)?;

        calc_locate_after_errors(
            &mut errors,
            locate_after,
            mm_data,
            statement_str,
            line_number,
        )?;
    }

    let float_hyp_str = *stage_2
        .floating_hypotheses
        .get(0)
        .ok_or(Error::InternalLogicError)?;

    let mut flaot_hyp_iter = float_hyp_str.split_ascii_whitespace();

    let label = flaot_hyp_iter
        .next()
        .ok_or(Error::InternalLogicError)?
        .to_string();
    let typecode = flaot_hyp_iter
        .next()
        .ok_or(Error::InternalLogicError)?
        .to_string();
    let variable = flaot_hyp_iter
        .next()
        .ok_or(Error::InternalLogicError)?
        .to_string();

    let Some((statement_str, line_number)) = calc_statement_str_and_line_number(
        &stage_1.statements,
        &stage_2.statements,
        MmpStatement::FloatingHypohesis,
        0,
    ) else {
        return Err(Error::InternalLogicError);
    };

    if !mm_data.symbols_not_already_taken(&vec![&label])
        && !mm_data
            .database_header
            .floating_hypohesis_iter()
            .any(|fh| fh.label == label)
    {
        let second_token_start_pos = util::nth_token_start_pos(statement_str, 1);
        let second_token_end_pos = util::nth_token_end_pos(statement_str, 1);

        errors.push(DetailedError {
            error_type: Error::LabelAlreadyExistsError,
            start_line_number: line_number + second_token_start_pos.0 - 1,
            start_column: second_token_start_pos.1,
            end_line_number: line_number + second_token_end_pos.0 - 1,
            end_column: second_token_end_pos.1 + 1,
        });
    }

    if mm_data
        .database_header
        .constant_locate_after_iter(stage_2.locate_after)
        .all(|c| c.symbol != typecode)
    {
        let third_token_start_pos = util::nth_token_start_pos(statement_str, 2);
        let third_token_end_pos = util::nth_token_end_pos(statement_str, 2);

        errors.push(DetailedError {
            error_type: Error::TypecodeNotAConstantError,
            start_line_number: line_number + third_token_start_pos.0 - 1,
            start_column: third_token_start_pos.1,
            end_line_number: line_number + third_token_end_pos.0 - 1,
            end_column: third_token_end_pos.1 + 1,
        });
    }

    if mm_data
        .database_header
        .variable_locate_after_iter(stage_2.locate_after)
        .all(|v| v.symbol != variable)
    {
        let fourth_token_start_pos = util::nth_token_start_pos(statement_str, 3);
        let fourth_token_end_pos = util::nth_token_end_pos(statement_str, 3);

        errors.push(DetailedError {
            error_type: Error::ExpectedActiveVariableError,
            start_line_number: line_number + fourth_token_start_pos.0 - 1,
            start_column: fourth_token_start_pos.1,
            end_line_number: line_number + fourth_token_end_pos.0 - 1,
            end_column: fourth_token_end_pos.1 + 1,
        });
    }

    if mm_data
        .database_header
        .floating_hypohesis_iter()
        .any(|fh| fh.variable == variable && fh.label != label)
    {
        let fourth_token_start_pos = util::nth_token_start_pos(statement_str, 3);
        let fourth_token_end_pos = util::nth_token_end_pos(statement_str, 3);

        errors.push(DetailedError {
            error_type: Error::VariableTypecodeAlreadyDeclaredError,
            start_line_number: line_number + fourth_token_start_pos.0 - 1,
            start_column: fourth_token_start_pos.1,
            end_line_number: line_number + fourth_token_end_pos.0 - 1,
            end_column: fourth_token_end_pos.1 + 1,
        });
    }

    Ok(if errors.is_empty() {
        MmpParserStage3::Success(MmpParserStage3Success::FloatingHypohesis(
            FloatingHypothesis {
                label,
                typecode,
                variable,
            },
        ))
    } else {
        MmpParserStage3::Fail(MmpParserStage3Fail { errors })
    })
}

fn calc_statement_str_and_line_number<'a>(
    statement_strs: &Vec<&'a str>,
    statements: &Vec<(MmpStatement, u32)>,
    searched_for_statement_type: MmpStatement,
    statement_i: usize,
) -> Option<(&'a str, u32)> {
    statement_strs
        .iter()
        .zip(statements.iter())
        .filter(|(_, (st, _))| *st == searched_for_statement_type)
        .nth(statement_i)
        .map(|(s, (_, ln))| (*s, *ln))
}

fn stage_3_variables<'a>(
    stage_1: &MmpParserStage1Success<'a>,
    stage_2: &MmpParserStage2Success<'a>,
    mm_data: &MetamathData,
) -> Result<MmpParserStage3<'a>, Error> {
    let mut errors: Vec<DetailedError> = Vec::new();

    if stage_2.variables.len() > 1 {
        calc_statement_out_of_place_errors(
            &mut errors,
            stage_1,
            stage_2,
            Error::VarStatementOutOfPlaceError,
            MmpStatement::Variable,
        );
        // The first flaoting hypothesis should not be marked as an error
        errors.swap_remove(0);
    }
    if stage_2.constants.is_some() {
        calc_statement_out_of_place_errors(
            &mut errors,
            stage_1,
            stage_2,
            Error::ConstStatementOutOfPlaceError,
            MmpStatement::Constant,
        );
    }
    if stage_2.allow_discouraged {
        calc_statement_out_of_place_errors(
            &mut errors,
            stage_1,
            stage_2,
            Error::AllowDiscouragedOutOfPlaceError,
            MmpStatement::AllowDiscouraged,
        );
    }
    if stage_2.allow_incomplete {
        calc_statement_out_of_place_errors(
            &mut errors,
            stage_1,
            stage_2,
            Error::AllowIncompleteOutOfPlaceError,
            MmpStatement::AllowIncomplete,
        );
    }
    if !stage_2.distinct_vars.is_empty() {
        calc_statement_out_of_place_errors(
            &mut errors,
            stage_1,
            stage_2,
            Error::DistinctVarOutOfPlaceError,
            MmpStatement::DistinctVar,
        );
    }
    if !stage_2.proof_lines.is_empty() {
        calc_statement_out_of_place_errors(
            &mut errors,
            stage_1,
            stage_2,
            Error::ProofLinesOutOfPlaceError,
            MmpStatement::ProofLine,
        );
    }

    if let Some(locate_after) = stage_2.locate_after {
        let (statement_str, line_number) = calc_statement_str_and_line_number(
            &stage_1.statements,
            &stage_2.statements,
            MmpStatement::LocateAfter,
            0,
        )
        .ok_or(Error::InternalLogicError)?;

        calc_locate_after_errors(
            &mut errors,
            locate_after,
            mm_data,
            statement_str,
            line_number,
        )?;
    }

    let variables_str = *stage_2.variables.get(0).ok_or(Error::InternalLogicError)?;

    let Some((_, start_line_number)) = calc_statement_str_and_line_number(
        &stage_1.statements,
        &stage_2.statements,
        MmpStatement::Variable,
        0,
    ) else {
        return Err(Error::InternalLogicError);
    };

    calc_non_new_math_symbol_errors(
        &mut errors,
        variables_str,
        start_line_number,
        2, // "$v".len()
        mm_data,
    );

    let variables: Vec<Variable> = variables_str
        .split_ascii_whitespace()
        .map(|s| Variable {
            symbol: s.to_string(),
        })
        .collect();

    Ok(if errors.is_empty() {
        MmpParserStage3::Success(MmpParserStage3Success::Variables(variables))
    } else {
        MmpParserStage3::Fail(MmpParserStage3Fail { errors })
    })
}

fn calc_non_new_math_symbol_errors(
    errors: &mut Vec<DetailedError>,
    math_symbols_str: &str,
    start_line: u32,
    start_column: u32,
    mm_data: &MetamathData,
) {
    let mut line = start_line;
    let mut column = start_column;

    let mut current_token_start_column = column;

    let mut current_token = String::new();
    let mut tokens_seen: Vec<String> = Vec::new();

    let mut seeing_token = false;

    for char in math_symbols_str.chars() {
        column += 1;

        if char == '\n' {
            line += 1;
            column = 0;
        }

        if char.is_ascii_whitespace() {
            if seeing_token {
                // if current_token.starts_with('$')
                //     || symbol_number_mapping.numbers.get(&current_token).is_none()
                // if mm_data
                //     .database_header
                //     .math_symbol_locate_after_iter(locate_after)
                //     .all(|symbol| symbol != current_token)
                if !mm_data.symbols_not_already_taken(&vec![&current_token]) {
                    errors.push(DetailedError {
                        error_type: Error::SymbolAlreadyExistsError,
                        start_line_number: line,
                        start_column: current_token_start_column,
                        end_line_number: line,
                        end_column: column,
                    });
                }
                if !util::is_valid_math_symbol(&current_token) {
                    errors.push(DetailedError {
                        error_type: Error::InvalidMathSymbolError,
                        start_line_number: line,
                        start_column: current_token_start_column,
                        end_line_number: line,
                        end_column: column,
                    });
                }
                if tokens_seen.iter().any(|s| *s == current_token) {
                    errors.push(DetailedError {
                        error_type: Error::TwiceDeclaredMathSymbolError,
                        start_line_number: line,
                        start_column: current_token_start_column,
                        end_line_number: line,
                        end_column: column,
                    });
                }

                tokens_seen.push(current_token);
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
}

fn stage_3_constants<'a>(
    stage_1: &MmpParserStage1Success<'a>,
    stage_2: &MmpParserStage2Success<'a>,
    mm_data: &MetamathData,
) -> Result<MmpParserStage3<'a>, Error> {
    let mut errors: Vec<DetailedError> = Vec::new();

    if stage_2.allow_discouraged {
        calc_statement_out_of_place_errors(
            &mut errors,
            stage_1,
            stage_2,
            Error::AllowDiscouragedOutOfPlaceError,
            MmpStatement::AllowDiscouraged,
        );
    }
    if stage_2.allow_incomplete {
        calc_statement_out_of_place_errors(
            &mut errors,
            stage_1,
            stage_2,
            Error::AllowIncompleteOutOfPlaceError,
            MmpStatement::AllowIncomplete,
        );
    }
    if !stage_2.distinct_vars.is_empty() {
        calc_statement_out_of_place_errors(
            &mut errors,
            stage_1,
            stage_2,
            Error::DistinctVarOutOfPlaceError,
            MmpStatement::DistinctVar,
        );
    }
    if !stage_2.proof_lines.is_empty() {
        calc_statement_out_of_place_errors(
            &mut errors,
            stage_1,
            stage_2,
            Error::ProofLinesOutOfPlaceError,
            MmpStatement::ProofLine,
        );
    }

    if let Some(locate_after) = stage_2.locate_after {
        let (statement_str, line_number) = calc_statement_str_and_line_number(
            &stage_1.statements,
            &stage_2.statements,
            MmpStatement::LocateAfter,
            0,
        )
        .ok_or(Error::InternalLogicError)?;

        calc_locate_after_errors(
            &mut errors,
            locate_after,
            mm_data,
            statement_str,
            line_number,
        )?;
    }

    let constants_str = stage_2.constants.ok_or(Error::InternalLogicError)?;

    let Some((_, start_line_number)) = calc_statement_str_and_line_number(
        &stage_1.statements,
        &stage_2.statements,
        MmpStatement::Constant,
        0,
    ) else {
        return Err(Error::InternalLogicError);
    };

    calc_non_new_math_symbol_errors(
        &mut errors,
        constants_str,
        start_line_number,
        2, // "$c".len()
        mm_data,
    );

    let constants: Vec<Constant> = constants_str
        .split_ascii_whitespace()
        .map(|s| Constant {
            symbol: s.to_string(),
        })
        .collect();

    Ok(if errors.is_empty() {
        MmpParserStage3::Success(MmpParserStage3Success::Constants(constants))
    } else {
        MmpParserStage3::Fail(MmpParserStage3Fail { errors })
    })
}

fn stage_3_comment<'a>(
    stage_1: &MmpParserStage1Success<'a>,
    stage_2: &MmpParserStage2Success<'a>,
    mm_data: &MetamathData,
    // comment_path: &'a str,
) -> Result<MmpParserStage3<'a>, Error> {
    let mut errors: Vec<DetailedError> = Vec::new();

    if stage_2.allow_discouraged {
        calc_statement_out_of_place_errors(
            &mut errors,
            stage_1,
            stage_2,
            Error::AllowDiscouragedOutOfPlaceError,
            MmpStatement::AllowDiscouraged,
        );
    }
    if stage_2.allow_incomplete {
        calc_statement_out_of_place_errors(
            &mut errors,
            stage_1,
            stage_2,
            Error::AllowIncompleteOutOfPlaceError,
            MmpStatement::AllowIncomplete,
        );
    }
    if !stage_2.distinct_vars.is_empty() {
        calc_statement_out_of_place_errors(
            &mut errors,
            stage_1,
            stage_2,
            Error::DistinctVarOutOfPlaceError,
            MmpStatement::DistinctVar,
        );
    }
    if !stage_2.proof_lines.is_empty() {
        calc_statement_out_of_place_errors(
            &mut errors,
            stage_1,
            stage_2,
            Error::ProofLinesOutOfPlaceError,
            MmpStatement::ProofLine,
        );
    }

    if let Some(locate_after) = stage_2.locate_after {
        let (statement_str, line_number) = calc_statement_str_and_line_number(
            &stage_1.statements,
            &stage_2.statements,
            MmpStatement::LocateAfter,
            0,
        )
        .ok_or(Error::InternalLogicError)?;

        calc_locate_after_errors(
            &mut errors,
            locate_after,
            mm_data,
            statement_str,
            line_number,
        )?;
    }

    let mut parent_header_path = HeaderPath::new();
    let mut comment_i = 0;

    for database_element in mm_data
        .database_header
        .locate_after_iter(stage_2.locate_after)
    {
        match database_element {
            DatabaseElement::Header(_, depth) => {
                util::calc_next_header_path(&mut parent_header_path, depth)?;
                comment_i = 0;
            }
            DatabaseElement::Statement(Statement::CommentStatement(_)) => {
                comment_i += 1;
            }
            _ => {}
        }
    }

    comment_i += 1;

    Ok(if errors.is_empty() {
        MmpParserStage3::Success(MmpParserStage3Success::Comment(MmpParserStage3Comment {
            parent_header_path,
            comment_i,
            comment: Comment {
                text: stage_2.comments.first().unwrap_or(&"").to_string(),
            },
        }))
    } else {
        MmpParserStage3::Fail(MmpParserStage3Fail { errors })
    })
}
