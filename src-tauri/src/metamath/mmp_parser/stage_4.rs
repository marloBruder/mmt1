use crate::{
    editor::on_edit::DetailedError,
    metamath::mmp_parser::MmpStatement,
    model::{MetamathData, ParseTree, SymbolNumberMapping, Theorem},
    util, Error,
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
    let mut preview_errors: Vec<(bool, bool, bool, bool)> = Vec::new();
    let mut preview_confirmations: Vec<bool> = Vec::new();
    let mut preview_confirmations_recursive: Vec<bool> = Vec::new();

    let mut proof_lines_parsed: Vec<ProofLineParsed> = Vec::new();

    let distinct_variable_pairs = util::calc_distinct_variable_pairs(&stage_2.distinct_vars);

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

        let mut preview_error = (false, false, false, false);

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

            preview_error.0 = true;
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
                        .take(i)
                        .enumerate()
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

                            preview_error.1 = true;
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

            preview_error.2 = true;
        }

        // Check if non-empty and non-hypothesis refs are valid theorem labels and save the theorem
        let mut theorem: Option<&Theorem> = None;

        if !proof_line.is_hypothesis && proof_line.step_ref != "" {
            if let Some(theorem_ref) = mm_data
                .database_header
                .theorem_locate_after_iter(stage_2.locate_after)
                .find(|t| t.label == proof_line.step_ref)
            {
                theorem = Some(theorem_ref);
            } else {
                errors.push(DetailedError {
                    error_type: Error::MmpStepRefNotALabelError,
                    start_line_number: line_number,
                    start_column: step_prefix_len - proof_line.step_ref.len() as u32 + 1,
                    end_line_number: line_number,
                    end_column: step_prefix_len + 1,
                });

                preview_error.2 = true;
            }
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
                // let last_non_whitespace_pos = stage_2::last_non_whitespace_pos(statement_str);

                // errors.push(DetailedError {
                //     error_type: Error::MissingMmpStepExpressionError,
                //     start_line_number: line_number,
                //     start_column: last_non_whitespace_pos.1 + 1,
                //     end_line_number: line_number,
                //     end_column: last_non_whitespace_pos.1 + 2,
                // });

                // preview_error.3 = true;
            }
            Err(Error::NonSymbolInExpressionError) => {
                errors.append(&mut calc_non_symbol_in_expression_errors(
                    proof_line.expression,
                    &mm_data.optimized_data.symbol_number_mapping,
                    line_number,
                    // step_prefix.len() as u32,
                    step_prefix_len,
                ));

                preview_error.3 = true;
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

                preview_error.3 = true;
            }
            Err(_) => {
                return Err(Error::InternalLogicError);
            }
        }

        //calc previw_confirmation
        let mut preview_confirmation = false;
        let mut preview_confirmation_recursive = false;

        if !preview_error.0 && !preview_error.1 && !preview_error.2 && !preview_error.3 {
            if let Some(theorem_ref) = theorem {
                // map hypotheses_parsed to the parse trees of the hypotheses
                // if a hypothesis-parsed is "?" (hyp == None), return Err(None)
                // If a hypothesis does not have a parse tree, also return Err(None)
                let proof_line_parse_trees_res = hypotheses_parsed
                    .iter()
                    .map(|hyp| match hyp {
                        Some(index) => Ok(proof_lines_parsed
                            .get(*index)
                            .ok_or(Some(Error::InternalLogicError))?
                            .parse_tree
                            .as_ref()
                            .ok_or(None)?),
                        None => Err(None),
                    })
                    .collect::<Result<Vec<&ParseTree>, Option<Error>>>();

                match proof_line_parse_trees_res {
                    // If one of the hyps was "?" or if it pointed to a proof_line_parsed without a parse tree, do nothing
                    Err(None) => {}
                    // Return potential InternalLogicError
                    Err(Some(err)) => return Err(err),
                    Ok(mut proof_line_parse_trees) => {
                        if let Some(parse_tree_ref) = parse_tree.as_ref() {
                            proof_line_parse_trees.push(parse_tree_ref);

                            if let Some(optimized_theorem_data) =
                                mm_data.optimized_data.theorem_data.get(&theorem_ref.label)
                            {
                                let parse_trees = optimized_theorem_data
                                    .parse_trees
                                    .as_ref()
                                    .ok_or(Error::InternalLogicError)?;

                                let mut theorem_parse_trees: Vec<&ParseTree> =
                                    parse_trees.hypotheses_parsed.iter().collect();
                                theorem_parse_trees.push(&parse_trees.assertion_parsed);

                                if ParseTree::are_substitutions(
                                    &theorem_parse_trees,
                                    &proof_line_parse_trees,
                                    &optimized_theorem_data.distinct_variable_pairs,
                                    &distinct_variable_pairs,
                                    &mm_data.optimized_data.grammar,
                                    &mm_data.optimized_data.symbol_number_mapping,
                                )? {
                                    preview_confirmation = true;

                                    if hypotheses_parsed.iter().all(|hyp| {
                                        hyp.is_some_and(|index| {
                                            preview_confirmations_recursive
                                                .get(index)
                                                .is_some_and(|&pre_con_rec| pre_con_rec)
                                        })
                                    }) {
                                        preview_confirmation_recursive = true;
                                    }
                                }
                            }
                        }
                    }
                }
            } else if proof_line.is_hypothesis {
                preview_confirmation = true;
                preview_confirmation_recursive = true;
            }
        }

        proof_lines_parsed.push(ProofLineParsed {
            hypotheses_parsed,
            parse_tree,
        });
        preview_errors.push(preview_error);
        preview_confirmations.push(preview_confirmation);
        preview_confirmations_recursive.push(preview_confirmation_recursive);
    }

    Ok(if errors.is_empty() {
        MmpParserStage4::Success(MmpParserStage4Success {
            distinct_variable_pairs,
            proof_lines_parsed,
            preview_errors,
            preview_confirmations,
            preview_confirmations_recursive,
        })
    } else {
        MmpParserStage4::Fail(MmpParserStage4Fail {
            errors,
            preview_errors,
            preview_confirmations,
            preview_confirmations_recursive,
        })
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
