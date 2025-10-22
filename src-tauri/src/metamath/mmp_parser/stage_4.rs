use std::collections::HashSet;

use crate::{
    editor::on_edit::DetailedError,
    metamath::mmp_parser::{MmpStatement, ProofLine, ProofLineStatus},
    model::{MetamathData, ParseTree, ProofType, SymbolNumberMapping, Theorem, TheoremType},
    util, Error,
};

use super::{
    MmpParserStage1Success, MmpParserStage2Success, MmpParserStage3Theorem, MmpParserStage4,
    MmpParserStage4Fail, MmpParserStage4Success, ProofLineParsed,
};

pub fn stage_4(
    stage_1: &MmpParserStage1Success,
    stage_2: &MmpParserStage2Success,
    stage_3: &MmpParserStage3Theorem,
    mm_data: &MetamathData,
) -> Result<MmpParserStage4, Error> {
    if !mm_data.grammar_calculations_done {
        return Ok(MmpParserStage4::Fail(MmpParserStage4Fail {
            errors: Vec::new(),
            reference_numbers: vec![None; stage_2.proof_lines.len()],
            proof_line_statuses: vec![ProofLineStatus::None; stage_2.proof_lines.len()],
        }));
    }

    if stage_3.is_axiom {
        return stage_4_axiom(stage_1, stage_2, mm_data);
    }

    let mut errors: Vec<DetailedError> = Vec::new();
    let mut reference_numbers = Vec::new();
    let mut proof_line_statuses: Vec<ProofLineStatus> = Vec::new();

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

        let mut error_status = (false, false, false, false);

        // Check duplicate step names
        if proof_line.step_name != ""
            && stage_2
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

            error_status.0 = true;
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
                        Some((i, _)) if hyp != "" => hypotheses_parsed.push(Some(i)),
                        _ => {
                            errors.push(DetailedError {
                                error_type: Error::HypNameDoesntExistError,
                                start_line_number: line_number,
                                start_column: start_column,
                                end_line_number: line_number,
                                end_column: start_column + hyp.len() as u32,
                            });

                            error_status.1 = true;
                        }
                    }
                }
                start_column += hyp.len() as u32 + 1;
            }
        }

        // Check that hypothesis lines don't have hypotheses
        if proof_line.is_hypothesis && proof_line.hypotheses != "" {
            errors.push(DetailedError {
                error_type: Error::HypothesisWithHypsError,
                start_line_number: line_number,
                start_column: step_prefix_len
                    - proof_line.step_ref.len() as u32
                    - proof_line.hypotheses.len() as u32,
                end_line_number: line_number,
                end_column: step_prefix_len - proof_line.step_ref.len() as u32,
            });

            error_status.1 = true;
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

            error_status.2 = true;
        }

        // Check if non-empty and non-hypothesis refs are valid theorem labels and save the theorem
        let mut theorem: Option<&Theorem> = None;

        if !proof_line.is_hypothesis && proof_line.step_ref != "" {
            if let Some((theorem_i, theorem_ref)) = mm_data
                .database_header
                .theorem_locate_after_iter(stage_2.locate_after)
                .enumerate()
                .find(|(_, t)| t.label == proof_line.step_ref)
            {
                theorem = Some(theorem_ref);
                reference_numbers.push(Some((theorem_i + 1) as u32));

                let theorem_data = mm_data
                    .optimized_data
                    .theorem_data
                    .get(&theorem_ref.label)
                    .ok_or(Error::InternalLogicError)?;

                if theorem_data.parse_trees.is_none() {
                    errors.push(DetailedError {
                        error_type: Error::SyntaxTheoremUsedError,
                        start_line_number: line_number,
                        start_column: step_prefix_len - proof_line.step_ref.len() as u32 + 1,
                        end_line_number: line_number,
                        end_column: step_prefix_len + 1,
                    });

                    error_status.2 = true;
                }

                if !stage_2.allow_discouraged && theorem_data.is_discouraged {
                    errors.push(DetailedError {
                        error_type: Error::DiscouragedTheoremUsedError,
                        start_line_number: line_number,
                        start_column: step_prefix_len - proof_line.step_ref.len() as u32 + 1,
                        end_line_number: line_number,
                        end_column: step_prefix_len + 1,
                    });

                    error_status.2 = true;
                }

                if !stage_2.allow_incomplete
                    && matches!(
                        theorem_data.theorem_type,
                        TheoremType::Theorem(
                            ProofType::CorrectButRecursivelyIncomplete | ProofType::Incomplete
                        )
                    )
                {
                    errors.push(DetailedError {
                        error_type: Error::IncompleteTheoremUsedError,
                        start_line_number: line_number,
                        start_column: step_prefix_len - proof_line.step_ref.len() as u32 + 1,
                        end_line_number: line_number,
                        end_column: step_prefix_len + 1,
                    });

                    error_status.2 = true;
                }

                if hypotheses_parsed.len() > theorem_ref.hypotheses.len() {
                    error_status.1 = true;
                    errors.push(DetailedError {
                        error_type: Error::TooManyHypothesesError,
                        start_line_number: line_number,
                        start_column: step_prefix_len
                            - proof_line.step_ref.len() as u32
                            - proof_line.hypotheses.len() as u32,
                        end_line_number: line_number,
                        end_column: step_prefix_len - proof_line.step_ref.len() as u32,
                    });
                }
            } else {
                errors.push(DetailedError {
                    error_type: Error::MmpStepRefNotALabelError,
                    start_line_number: line_number,
                    start_column: step_prefix_len - proof_line.step_ref.len() as u32 + 1,
                    end_line_number: line_number,
                    end_column: step_prefix_len + 1,
                });

                error_status.2 = true;
                reference_numbers.push(None);
            }
        } else {
            reference_numbers.push(None)
        }

        // Calculate parse_tree
        let mut parse_tree = None;

        calc_parse_tree_and_handle_errors(
            &mut parse_tree,
            &mut errors,
            &mut error_status,
            proof_line,
            step_prefix_len,
            statement_str,
            line_number,
            mm_data,
        )?;

        //calc previw_confirmation
        let mut correct = false;
        let mut correct_recursively = false;

        if !error_status.0 && !error_status.1 && !error_status.2 && !error_status.3 {
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

                            let optimized_theorem_data = mm_data
                                .optimized_data
                                .theorem_data
                                .get(&theorem_ref.label)
                                .ok_or(Error::InternalLogicError)?;

                            if let Some(parse_trees) = optimized_theorem_data.parse_trees.as_ref() {
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
                                    correct = true;

                                    if hypotheses_parsed.iter().all(|hyp| {
                                        hyp.is_some_and(|index| {
                                            proof_line_statuses.get(index).is_some_and(
                                                |pre_con_rec| {
                                                    matches!(
                                                        pre_con_rec,
                                                        ProofLineStatus::CorrectRecursively
                                                    )
                                                },
                                            )
                                        })
                                    }) {
                                        correct_recursively = true;
                                    }
                                }
                            }
                        }
                    }
                }
            } else if proof_line.is_hypothesis
                && proof_line.step_ref != ""
                && proof_line.expression != ""
            {
                correct = true;
                correct_recursively = true;
            }
        }

        proof_lines_parsed.push(ProofLineParsed {
            hypotheses_parsed,
            parse_tree,
        });
        proof_line_statuses.push(if correct_recursively {
            ProofLineStatus::CorrectRecursively
        } else if correct {
            ProofLineStatus::Correct
        } else if error_status.0 || error_status.1 || error_status.2 || error_status.3 {
            ProofLineStatus::Err(error_status)
        } else {
            ProofLineStatus::None
        });
    }

    Ok(if errors.is_empty() {
        MmpParserStage4::Success(MmpParserStage4Success {
            distinct_variable_pairs,
            proof_lines_parsed,
            reference_numbers,
            proof_line_statuses,
        })
    } else {
        MmpParserStage4::Fail(MmpParserStage4Fail {
            errors,
            reference_numbers,
            proof_line_statuses,
        })
    })
}

fn calc_parse_tree_and_handle_errors(
    parse_tree: &mut Option<ParseTree>,
    errors: &mut Vec<DetailedError>,
    error_status: &mut (bool, bool, bool, bool),
    proof_line: &ProofLine,
    step_prefix_len: u32,
    statement_str: &str,
    line_number: u32,
    mm_data: &MetamathData,
) -> Result<(), Error> {
    match mm_data.expression_to_parse_tree(proof_line.expression) {
        Ok(pt) => *parse_tree = Some(pt),
        Err(Error::MissingExpressionError) => {}
        Err(Error::NonSymbolInExpressionError) => {
            errors.append(
                &mut calc_non_symbol_in_expression_and_invalid_work_variable_errors(
                    proof_line.expression,
                    &mm_data.optimized_data.symbol_number_mapping,
                    line_number,
                    // step_prefix.len() as u32,
                    step_prefix_len,
                ),
            );

            error_status.3 = true;
        }
        Err(Error::ExpressionParseError) => {
            let last_non_whitespace_pos = util::last_non_whitespace_pos(statement_str);

            errors.push(DetailedError {
                error_type: Error::ExpressionParseError,
                start_line_number: line_number,
                start_column: last_non_whitespace_pos.1 + 1,
                end_line_number: line_number,
                end_column: last_non_whitespace_pos.1 + 2,
            });

            error_status.3 = true;
        }
        Err(Error::InvalidWorkVariableError) => {
            errors.append(
                &mut calc_non_symbol_in_expression_and_invalid_work_variable_errors(
                    proof_line.expression,
                    &mm_data.optimized_data.symbol_number_mapping,
                    line_number,
                    // step_prefix.len() as u32,
                    step_prefix_len,
                ),
            );

            error_status.3 = true;
        }
        Err(Error::InvalidTypecodeError) => {
            let second_token_start_pos = util::nth_token_start_pos(statement_str, 1);
            let second_token_end_pos = util::nth_token_end_pos(statement_str, 1);

            errors.push(DetailedError {
                error_type: Error::InvalidTypecodeError,
                start_line_number: line_number + second_token_start_pos.0 - 1,
                start_column: second_token_start_pos.1,
                end_line_number: line_number + second_token_end_pos.0 - 1,
                end_column: second_token_end_pos.1 + 1,
            });

            error_status.3 = true;
        }
        Err(_) => {
            return Err(Error::InternalLogicError);
        }
    }

    Ok(())
}

fn calc_non_symbol_in_expression_and_invalid_work_variable_errors(
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
                if let Some(error) = check_symbol_for_error(
                    &current_token,
                    symbol_number_mapping,
                    line,
                    column,
                    current_token_start_column,
                ) {
                    errors.push(error);
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

fn check_symbol_for_error(
    symbol: &str,
    symbol_number_mapping: &SymbolNumberMapping,
    line: u32,
    column: u32,
    current_token_start_column: u32,
) -> Option<DetailedError> {
    if let Some((before, after)) = symbol.split_once('$') {
        if symbol_number_mapping
            .numbers
            .get(before)
            .is_none_or(|num| !symbol_number_mapping.is_variable(*num))
            || after.starts_with('+')
            || after.parse::<u32>().is_err()
        {
            return Some(DetailedError {
                error_type: Error::InvalidWorkVariableError,
                start_line_number: line,
                start_column: current_token_start_column,
                end_line_number: line,
                end_column: column,
            });
        }
    } else {
        if symbol_number_mapping.numbers.get(symbol).is_none() {
            return Some(DetailedError {
                error_type: Error::NonSymbolInExpressionError,
                start_line_number: line,
                start_column: current_token_start_column,
                end_line_number: line,
                end_column: column,
            });
        }
    }

    None
}

pub fn stage_4_axiom(
    stage_1: &MmpParserStage1Success,
    stage_2: &MmpParserStage2Success,
    mm_data: &MetamathData,
) -> Result<MmpParserStage4, Error> {
    let mut errors: Vec<DetailedError> = Vec::new();

    let mut proof_lines_parsed: Vec<ProofLineParsed> = Vec::new();

    let is_syntax_axiom = stage_2
        .proof_lines
        .iter()
        .find(|pl| pl.step_name == "qed")
        .is_some_and(|pl| {
            pl.expression
                .split_ascii_whitespace()
                .next()
                .is_some_and(|pl_typecode| {
                    mm_data
                        .syntax_typecodes
                        .iter()
                        .any(|st| st.typecode == pl_typecode)
                })
        });

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

        let mut error_status = (false, false, false, false);
        let mut parse_tree = None;

        // Check duplicate step names
        if proof_line.step_name != ""
            && stage_2
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

        if proof_line.is_hypothesis {
            // Perform hypothesis specific checks
            if proof_line.hypotheses != "" {
                errors.push(DetailedError {
                    error_type: Error::AxiomStepWithHypError,
                    start_line_number: line_number,
                    start_column: step_prefix_len
                        - proof_line.step_ref.len() as u32
                        - proof_line.hypotheses.len() as u32,
                    end_line_number: line_number,
                    end_column: step_prefix_len - proof_line.step_ref.len() as u32,
                });
            }

            if !is_syntax_axiom {
                calc_parse_tree_and_handle_errors(
                    &mut parse_tree,
                    &mut errors,
                    &mut error_status,
                    proof_line,
                    step_prefix_len,
                    statement_str,
                    line_number,
                    mm_data,
                )?;
            } else {
                let last_non_whitespace_pos = util::last_non_whitespace_pos(statement_str);

                errors.push(DetailedError {
                    error_type: Error::SyntaxAxiomWithHypothesesError,
                    start_line_number: line_number,
                    start_column: 0,
                    end_line_number: line_number + last_non_whitespace_pos.0 - 1,
                    end_column: last_non_whitespace_pos.1 + 1,
                });
            }
        } else if proof_line.step_name == "qed" {
            // Perform qed step sepcific checks
            if proof_line.hypotheses != "" {
                errors.push(DetailedError {
                    error_type: Error::AxiomStepWithHypError,
                    start_line_number: line_number,
                    start_column: step_prefix_len
                        - proof_line.step_ref.len() as u32
                        - proof_line.hypotheses.len() as u32,
                    end_line_number: line_number,
                    end_column: step_prefix_len - proof_line.step_ref.len() as u32,
                });
            }

            if proof_line.step_ref != "" {
                errors.push(DetailedError {
                    error_type: Error::AxiomQedStepWithRefError,
                    start_line_number: line_number,
                    start_column: step_prefix_len - proof_line.step_ref.len() as u32 + 1,
                    end_line_number: line_number,
                    end_column: step_prefix_len + 1,
                });
            }

            if !is_syntax_axiom {
                calc_parse_tree_and_handle_errors(
                    &mut parse_tree,
                    &mut errors,
                    &mut error_status,
                    proof_line,
                    step_prefix_len,
                    statement_str,
                    line_number,
                    mm_data,
                )?;
            }
        } else {
            let last_non_whitespace_pos = util::last_non_whitespace_pos(statement_str);

            errors.push(DetailedError {
                error_type: Error::InvalidMmpStepForAxiomError,
                start_line_number: line_number,
                start_column: 0,
                end_line_number: line_number + last_non_whitespace_pos.0 - 1,
                end_column: last_non_whitespace_pos.1 + 1,
            });
        }

        if parse_tree
            .as_ref()
            .is_some_and(|pt| pt.has_work_variables())
        {
            let second_token_start_pos = util::nth_token_start_pos(statement_str, 1);
            let last_non_whitespace_pos = util::last_non_whitespace_pos(statement_str);

            errors.push(DetailedError {
                error_type: Error::AxiomWithWorkVariableError,
                start_line_number: line_number + second_token_start_pos.0 - 1,
                start_column: second_token_start_pos.1,
                end_line_number: line_number + last_non_whitespace_pos.0 - 1,
                end_column: last_non_whitespace_pos.1 + 1,
            });
        }

        proof_lines_parsed.push(ProofLineParsed {
            hypotheses_parsed: Vec::new(),
            parse_tree,
        });
    }

    Ok(if errors.is_empty() {
        MmpParserStage4::Success(MmpParserStage4Success {
            distinct_variable_pairs: HashSet::new(),
            proof_lines_parsed,
            reference_numbers: Vec::new(),
            proof_line_statuses: Vec::new(),
        })
    } else {
        MmpParserStage4::Fail(MmpParserStage4Fail {
            errors,
            reference_numbers: Vec::new(),
            proof_line_statuses: Vec::new(),
        })
    })
}
