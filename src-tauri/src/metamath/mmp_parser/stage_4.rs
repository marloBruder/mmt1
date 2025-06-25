use crate::{
    editor::on_edit::DetailedError,
    metamath::mmp_parser::MmpStatement,
    model::{MetamathData, SymbolNumberMapping},
    Error,
};

use super::{
    stage_2, MmpParserStage1Success, MmpParserStage2Success, MmpParserStage3Theorem,
    MmpParserStage4, MmpParserStage4Fail, MmpParserStage4Success, ProofLineParsed,
};

pub fn stage_4(
    stage_1: &MmpParserStage1Success,
    stage_2: &MmpParserStage2Success,
    _stage_3: &MmpParserStage3Theorem,
    mm_data: &MetamathData,
) -> Result<MmpParserStage4, Error> {
    let mut errors: Vec<DetailedError> = Vec::new();

    let mut proof_lines_parsed: Vec<ProofLineParsed> = Vec::new();

    for (i, (proof_line, (statement_str, line_number))) in stage_2
        .proof_lines
        .iter()
        .zip(
            stage_1
                .statements
                .iter()
                .zip(stage_2.statements.iter())
                .filter_map(|(str, (st, ln))| match st {
                    MmpStatement::ProofLine => Some((*str, *ln)),
                    _ => None,
                }),
        )
        .enumerate()
    {
        let step_prefix_len = statement_str
            .split_ascii_whitespace()
            .next()
            .ok_or(Error::InternalLogicError)?
            .len() as u32;

        // Check duplicate step names
        if stage_2
            .proof_lines
            .iter()
            .take(i)
            .any(|pl| pl.step_name == proof_line.step_name)
        {
            errors.push(DetailedError {
                error_type: Error::DuplicateStepNameError,
                start_line_number: line_number,
                start_column: 1
                    + proof_line.advanced_unification as u32
                    + proof_line.is_hypothesis as u32,
                end_line_number: line_number,
                end_column: 1
                    + proof_line.advanced_unification as u32
                    + proof_line.is_hypothesis as u32
                    + proof_line.step_name.len() as u32,
            });
        }

        // Calculate hypotheses_parsed
        let mut hypotheses_parsed: Vec<Option<usize>> = Vec::new();

        if proof_line.hypotheses != "" {
            let mut start_column = 1
                + proof_line.advanced_unification as u32
                + proof_line.is_hypothesis as u32
                + proof_line.step_name.len() as u32
                + 1;
            for hyp in proof_line.hypotheses.split(',') {
                if hyp == "?" {
                    hypotheses_parsed.push(None);
                } else {
                    match stage_2
                        .proof_lines
                        .iter()
                        .enumerate()
                        .take(i)
                        .find(|(_, pl)| pl.step_name == hyp)
                    {
                        Some((i, _)) => hypotheses_parsed.push(Some(i)),
                        None => {
                            errors.push(DetailedError {
                                error_type: Error::HypNameDoesntExistError,
                                start_line_number: line_number,
                                start_column: start_column,
                                end_line_number: line_number,
                                end_column: start_column + hyp.len() as u32,
                            });
                        }
                    }
                }
                start_column += hyp.len() as u32 + 1;
            }
        }

        // Check duplicate hypothesis names
        if proof_line.is_hypothesis
            && proof_line.step_ref != ""
            && stage_2
                .proof_lines
                .iter()
                .take(i)
                .any(|pl| pl.is_hypothesis && pl.step_ref == proof_line.step_ref)
        {
            errors.push(DetailedError {
                error_type: Error::DuplicateHypLabelsError,
                start_line_number: line_number,
                start_column: step_prefix_len - proof_line.step_ref.len() as u32 + 1,
                end_line_number: line_number,
                end_column: step_prefix_len + 1,
            });
        }

        // Check if non-hypothesis refs are valid labels
        if !proof_line.is_hypothesis
            && proof_line.step_ref != ""
            && !mm_data
                .database_header
                .theorem_locate_after_iter(stage_2.locate_after)
                .any(|t| t.label == proof_line.step_ref)
        {
            errors.push(DetailedError {
                error_type: Error::MmpStepRefNotALabelError,
                start_line_number: line_number,
                start_column: step_prefix_len - proof_line.step_ref.len() as u32 + 1,
                end_line_number: line_number,
                end_column: step_prefix_len + 1,
            });
        }

        // Calculate parse_tree
        let mut parse_tree = None;

        match mm_data
            .optimized_data
            .symbol_number_mapping
            .expression_to_parse_tree(proof_line.expression, &mm_data.optimized_data.grammar)
        {
            Ok(pt) => parse_tree = Some(pt),
            Err(Error::MissingExpressionError) => {
                let last_non_whitespace_pos = stage_2::last_non_whitespace_pos(statement_str);

                errors.push(DetailedError {
                    error_type: Error::MissingMmpStepExpressionError,
                    start_line_number: line_number,
                    start_column: last_non_whitespace_pos.1 + 1,
                    end_line_number: line_number,
                    end_column: last_non_whitespace_pos.1 + 2,
                });
            }
            Err(Error::NonSymbolInExpressionError) => {
                errors.append(&mut calc_non_symbol_in_expression_errors(
                    proof_line.expression,
                    &mm_data.optimized_data.symbol_number_mapping,
                    line_number,
                    // step_prefix.len() as u32,
                    step_prefix_len,
                ));
            }
            Err(Error::ExpressionParseError) => {
                let last_non_whitespace_pos = stage_2::last_non_whitespace_pos(statement_str);

                errors.push(DetailedError {
                    error_type: Error::ExpressionParseError,
                    start_line_number: line_number,
                    start_column: last_non_whitespace_pos.1 + 1,
                    end_line_number: line_number,
                    end_column: last_non_whitespace_pos.1 + 2,
                });
            }
            Err(_) => {
                return Err(Error::InternalLogicError);
            }
        }

        proof_lines_parsed.push(ProofLineParsed {
            hypotheses_parsed,
            parse_tree,
        });
    }

    Ok(if errors.is_empty() {
        MmpParserStage4::Success(MmpParserStage4Success { proof_lines_parsed })
    } else {
        MmpParserStage4::Fail(MmpParserStage4Fail { errors })
    })
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

        if char == '\n' {
            line += 1;
            column = 0;
        }
    }

    errors
}
