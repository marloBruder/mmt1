use crate::{
    editor::on_edit::DetailedError,
    model::{Comment, HeaderPath, MetamathData, Statement},
    util, Error,
};

use super::{
    stage_2, MmpLabel, MmpParserStage1Success, MmpParserStage2Success, MmpParserStage3,
    MmpParserStage3Comment, MmpParserStage3Fail, MmpParserStage3Header, MmpParserStage3Success,
    MmpStatement,
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
        Some(MmpLabel::Comment(comment_path)) => {
            stage_3_comment(stage_1, stage_2, mm_data, comment_path)
        }
        // Some(MmpLabel::Axiom(axiom_label)) => {
        //     stage_3_theorem(stage_1, stage_2, axiom_label, true, mm_data)
        // }
        // Some(MmpLabel::Theorem(theorem_label)) => {
        //     stage_3_theorem(stage_1, stage_2, theorem_label, false, mm_data)
        // }
        _ => Ok(MmpParserStage3::Fail(MmpParserStage3Fail {
            errors: Vec::new(),
        })),
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
        calc_header_and_comment_statement_out_of_place_errors(stage_1, stage_2);

    let mut parent_header_path =
        HeaderPath::from_str(header_path).ok_or(Error::InternalLogicError)?;
    let header_i = parent_header_path
        .path
        .pop()
        .ok_or(Error::InternalLogicError)?;

    if let Some(parent_header) = parent_header_path.resolve(&mm_data.database_header) {
        // Allow header_i == len()
        if parent_header.subheaders.len() < header_i {
            errors.push(calc_label_error(
                stage_1,
                stage_2,
                Error::InvalidHeaderPathError,
            )?);
        }
    } else {
        errors.push(calc_label_error(
            stage_1,
            stage_2,
            Error::InvalidHeaderPathError,
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
    stage_1: &MmpParserStage1Success,
    stage_2: &MmpParserStage2Success,
    error_type: Error,
    out_of_place_statement_type: MmpStatement,
) -> Vec<DetailedError> {
    let mut errors: Vec<DetailedError> = Vec::new();

    let mut line_number = stage_1.number_of_lines_before_first_statement;

    for (&statement_str, statement_type) in stage_1.statements.iter().zip(&stage_2.statements) {
        if *statement_type == out_of_place_statement_type {
            let last_non_whitespace_pos = stage_2::last_non_whitespace_pos(statement_str);

            errors.push(DetailedError {
                error_type,
                start_line_number: line_number,
                start_column: 1,
                end_line_number: line_number + last_non_whitespace_pos.0 - 1,
                end_column: last_non_whitespace_pos.1 + 1,
            });
        }

        line_number += stage_2::new_lines_in_str(statement_str);
    }

    errors
}

fn calc_header_and_comment_statement_out_of_place_errors(
    stage_1: &MmpParserStage1Success,
    stage_2: &MmpParserStage2Success,
) -> Vec<DetailedError> {
    let mut errors: Vec<DetailedError> = Vec::new();

    if stage_2.constants.is_some() {
        errors.append(&mut calc_statement_out_of_place_errors(
            stage_1,
            stage_2,
            Error::ConstStatementOutOfPlaceError,
            MmpStatement::Constant,
        ));
    }
    if !stage_2.variables.is_empty() {
        errors.append(&mut calc_statement_out_of_place_errors(
            stage_1,
            stage_2,
            Error::VarStatementOutOfPlaceError,
            MmpStatement::Variable,
        ));
    }
    if !stage_2.floating_hypotheses.is_empty() {
        errors.append(&mut calc_statement_out_of_place_errors(
            stage_1,
            stage_2,
            Error::FloatHypStatementsOutOfPlaceError,
            MmpStatement::FloatingHypohesis,
        ));
    }
    if stage_2.allow_discouraged {
        errors.append(&mut calc_statement_out_of_place_errors(
            stage_1,
            stage_2,
            Error::AllowDiscouragedOutOfPlaceError,
            MmpStatement::AllowDiscouraged,
        ));
    }
    if stage_2.locate_after.is_some() {
        errors.append(&mut calc_statement_out_of_place_errors(
            stage_1,
            stage_2,
            Error::LocateAfterOutOfPlaceError,
            MmpStatement::LocateAfter,
        ));
    }
    if !stage_2.distinct_vars.is_empty() {
        errors.append(&mut calc_statement_out_of_place_errors(
            stage_1,
            stage_2,
            Error::DistinctVarOutOfPlaceError,
            MmpStatement::DistinctVar,
        ));
    }
    if !stage_2.proof_lines.is_empty() {
        errors.append(&mut calc_statement_out_of_place_errors(
            stage_1,
            stage_2,
            Error::ProofLinesOutOfPlaceError,
            MmpStatement::ProofLine,
        ));
    }

    errors
}

fn calc_label_error(
    stage_1: &MmpParserStage1Success,
    stage_2: &MmpParserStage2Success,
    error_type: Error,
) -> Result<DetailedError, Error> {
    let mut line_number = stage_1.number_of_lines_before_first_statement;

    for (&statement_str, statement_type) in stage_1.statements.iter().zip(&stage_2.statements) {
        if *statement_type == MmpStatement::MmpLabel {
            let first_token_start_pos = stage_2::nth_token_start_pos(statement_str, 1);
            let first_token_end_pos = stage_2::nth_token_end_pos(statement_str, 1);

            return Ok(DetailedError {
                error_type,
                start_line_number: line_number + first_token_start_pos.0 - 1,
                start_column: first_token_start_pos.1,
                end_line_number: line_number + first_token_end_pos.0 - 1,
                end_column: first_token_end_pos.1 + 1,
            });
        }

        line_number += stage_2::new_lines_in_str(statement_str);
    }

    Err(Error::InternalLogicError)
}

fn stage_3_comment<'a>(
    stage_1: &MmpParserStage1Success<'a>,
    stage_2: &MmpParserStage2Success<'a>,
    mm_data: &MetamathData,
    comment_path: &'a str,
) -> Result<MmpParserStage3<'a>, Error> {
    let mut errors: Vec<DetailedError> =
        calc_header_and_comment_statement_out_of_place_errors(stage_1, stage_2);

    let (parent_header_path_str, comment_i_str) = comment_path
        .split_once('#')
        .ok_or(Error::InternalLogicError)?;

    let parent_header_path =
        HeaderPath::from_str(parent_header_path_str).ok_or(Error::InternalLogicError)?;
    let comment_i: usize = comment_i_str
        .parse()
        .map_err(|_| Error::InternalLogicError)?;

    if let Some(parent_header) = parent_header_path.resolve(&mm_data.database_header) {
        // comment_i should be at most 1 + count()
        if comment_i
            > 1 + parent_header
                .content
                .iter()
                .filter(|s| matches!(s, Statement::CommentStatement(_)))
                .count()
        {
            errors.push(calc_label_error(
                stage_1,
                stage_2,
                Error::InvalidCommentPathError,
            )?);
        }
    } else {
        errors.push(calc_label_error(
            stage_1,
            stage_2,
            Error::InvalidCommentPathError,
        )?);
    }

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

// fn stage_3_theorem<'a>(
//     stage_1: &MmpParserStage1Success<'a>,
//     stage_2: &MmpParserStage2Success<'a>,
//     label: &'a str,
//     is_axiom: bool,
//     mm_data: &MetamathData,
// ) -> Result<MmpParserStage3<'a>, Error> {
//     let mut errors: Vec<Error> = Vec::new();

//     if stage_2.proof_lines.iter().any(|pl| {
//         !pl.is_hypothesis
//             && mm_data
//                 .database_header
//                 .theorem_locate_after_iter(stage_2.locate_after)
//                 .all(|t| t.label != pl.step_ref)
//     }) {
//         errors.push(Error::MmpStepRefNotALabelError);
//     }

//     Ok(MmpParserStage3::Fail(MmpParserStage3Fail { errors }))
// }
