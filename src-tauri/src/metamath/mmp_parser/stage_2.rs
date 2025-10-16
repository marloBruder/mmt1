use super::{
    LocateAfterRef, MmpLabel, MmpParserStage1Success, MmpParserStage2, MmpParserStage2Fail,
    MmpParserStage2Success, MmpStatement, ProofLine,
};
use crate::{editor::on_edit::DetailedError, model::HeaderPath, util, Error};

pub fn stage_2<'a>(stage_1: &MmpParserStage1Success<'a>) -> Result<MmpParserStage2<'a>, Error> {
    let mut label: Option<MmpLabel> = None;
    let mut allow_discouraged: bool = false;
    let mut locate_after: Option<LocateAfterRef> = None;
    let mut distinct_vars: Vec<&str> = Vec::new();
    let mut constants: Option<&str> = None;
    let mut variables: Vec<&str> = Vec::new();
    let mut floating_hypotheses: Vec<&str> = Vec::new();
    let mut proof_lines: Vec<ProofLine> = Vec::new();
    let mut comments: Vec<&str> = Vec::new();
    let mut statements: Vec<(MmpStatement, u32)> = Vec::with_capacity(stage_1.statements.len());

    let mut errors: Vec<DetailedError> = Vec::new();

    let mut current_line: u32 = stage_1.number_of_lines_before_first_statement;

    for &statement_str in &stage_1.statements {
        let mut token_iter = statement_str.split_ascii_whitespace();

        let last_non_whitespace_pos = util::last_non_whitespace_pos(statement_str);

        match token_iter.next().ok_or(Error::InternalLogicError)? {
            "$c" => {
                if constants.is_some() {
                    errors.push(DetailedError {
                        error_type: Error::TooManyConstStatementsError,
                        start_line_number: current_line,
                        start_column: 1,
                        end_line_number: current_line + last_non_whitespace_pos.0 - 1,
                        end_column: last_non_whitespace_pos.1 + 1,
                    });
                }

                if token_iter.next().is_some() {
                    constants = Some(&statement_str[2..statement_str.len()]);
                } else {
                    errors.push(DetailedError {
                        error_type: Error::EmptyConstStatementError,
                        start_line_number: current_line,
                        start_column: 1,
                        end_line_number: current_line + last_non_whitespace_pos.0 - 1,
                        end_column: last_non_whitespace_pos.1 + 1,
                    });
                }

                statements.push((MmpStatement::Constant, current_line));
            }
            "$v" => {
                if token_iter.next().is_some() {
                    variables.push(&statement_str[2..statement_str.len()]);
                } else {
                    errors.push(DetailedError {
                        error_type: Error::EmptyVarStatementError,
                        start_line_number: current_line,
                        start_column: 1,
                        end_line_number: current_line + last_non_whitespace_pos.0 - 1,
                        end_column: last_non_whitespace_pos.1 + 1,
                    });
                }

                statements.push((MmpStatement::Variable, current_line));
            }
            "$f" => {
                // token_iter should only have exactly three more token
                let first_token = token_iter.next();
                let second_token = token_iter.next();
                let third_token = token_iter.next();
                let fourth_token = token_iter.next();

                if first_token.is_none() {
                    errors.push(DetailedError {
                        error_type: Error::FloatHypStatementFormatError,
                        start_line_number: current_line,
                        start_column: 1,
                        end_line_number: current_line,
                        end_column: 3, // Length of "$f" + 1
                    });
                } else if second_token.is_none() || third_token.is_none() {
                    let second_token_start_pos = util::nth_token_start_pos(statement_str, 1);

                    errors.push(DetailedError {
                        error_type: Error::FloatHypStatementFormatError,
                        start_line_number: current_line + second_token_start_pos.0 - 1,
                        start_column: second_token_start_pos.1,
                        end_line_number: current_line + last_non_whitespace_pos.0 - 1,
                        end_column: last_non_whitespace_pos.1 + 1,
                    });
                } else if fourth_token.is_some() {
                    let fifth_token_start_pos = util::nth_token_start_pos(statement_str, 4);

                    errors.push(DetailedError {
                        error_type: Error::FloatHypStatementFormatError,
                        start_line_number: current_line + fifth_token_start_pos.0 - 1,
                        start_column: fifth_token_start_pos.1,
                        end_line_number: current_line + last_non_whitespace_pos.0 - 1,
                        end_column: last_non_whitespace_pos.1 + 1,
                    });
                } else if !util::is_valid_label(first_token.ok_or(Error::InternalLogicError)?) {
                    let second_token_start_pos = util::nth_token_start_pos(statement_str, 1);
                    let second_token_end_pos = util::nth_token_end_pos(statement_str, 1);

                    errors.push(DetailedError {
                        error_type: Error::InvalidLabelError,
                        start_line_number: current_line + second_token_start_pos.0 - 1,
                        start_column: second_token_start_pos.1,
                        end_line_number: current_line + second_token_end_pos.0 - 1,
                        end_column: second_token_end_pos.1 + 1,
                    });
                } else {
                    floating_hypotheses.push(&statement_str[3..statement_str.len()]);
                }

                statements.push((MmpStatement::FloatingHypohesis, current_line));
            }
            "$header" => {
                if label.is_some() {
                    errors.push(DetailedError {
                        error_type: Error::MultipleMmpLabelsError,
                        start_line_number: current_line,
                        start_column: 1,
                        end_line_number: current_line + last_non_whitespace_pos.0 - 1,
                        end_column: last_non_whitespace_pos.1 + 1,
                    });
                }

                if let Some(header_path) = token_iter.next() {
                    if HeaderPath::from_str(header_path).is_none() {
                        let second_token_start_pos = util::nth_token_start_pos(statement_str, 1);
                        let second_token_end_pos = util::nth_token_end_pos(statement_str, 1);

                        errors.push(DetailedError {
                            error_type: Error::InvalidHeaderPathFormatError,
                            start_line_number: current_line + second_token_start_pos.0 - 1,
                            start_column: second_token_start_pos.1,
                            end_line_number: current_line + second_token_end_pos.0 - 1,
                            end_column: second_token_end_pos.1 + 1,
                        });
                    }

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

                        let title = &statement_str[(statement_i + 1)..statement_str.len()];

                        label = Some(MmpLabel::Header { header_path, title });

                        statements.push((MmpStatement::MmpLabel, current_line));
                    } else {
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
            "$comment" => {
                if label.is_some() {
                    errors.push(DetailedError {
                        error_type: Error::MultipleMmpLabelsError,
                        start_line_number: current_line,
                        start_column: 1,
                        end_line_number: current_line + last_non_whitespace_pos.0 - 1,
                        end_column: last_non_whitespace_pos.1 + 1,
                    });
                }

                if let Some(comment_path) = token_iter.next() {
                    if let Some((header_path, comment_num)) = comment_path.split_once('#') {
                        if HeaderPath::from_str(header_path).is_none()
                            || comment_num.parse::<usize>().is_err()
                            || comment_num.contains('+')
                        {
                            let second_token_start_pos =
                                util::nth_token_start_pos(statement_str, 1);
                            let second_token_end_pos = util::nth_token_end_pos(statement_str, 1);

                            errors.push(DetailedError {
                                error_type: Error::InvalidCommentPathFormatError,
                                start_line_number: current_line + second_token_start_pos.0 - 1,
                                start_column: second_token_start_pos.1,
                                end_line_number: current_line + second_token_end_pos.0 - 1,
                                end_column: second_token_end_pos.1 + 1,
                            });
                        }
                    } else {
                        let second_token_start_pos = util::nth_token_start_pos(statement_str, 1);
                        let second_token_end_pos = util::nth_token_end_pos(statement_str, 1);

                        errors.push(DetailedError {
                            error_type: Error::InvalidCommentPathFormatError,
                            start_line_number: current_line + second_token_start_pos.0 - 1,
                            start_column: second_token_start_pos.1,
                            end_line_number: current_line + second_token_end_pos.0 - 1,
                            end_column: second_token_end_pos.1 + 1,
                        });
                    }

                    label = Some(MmpLabel::Comment(comment_path));
                } else {
                    errors.push(DetailedError {
                        error_type: Error::MissingCommentPathError,
                        start_line_number: current_line,
                        start_column: 1,
                        end_line_number: current_line,
                        end_column: 9, // Length of "$theorem" + 1
                    });
                    // Make sure label is set to Some(_) so that future label statements will be flagged as errors
                    // Since return_info is false, the content within Some(_) does not matter
                    label = Some(MmpLabel::Theorem(""));
                }

                if token_iter.next().is_some() {
                    let third_token_start_pos = util::nth_token_start_pos(statement_str, 2);

                    errors.push(DetailedError {
                        error_type: Error::TooManyCommentPathTokensError,
                        start_line_number: current_line + third_token_start_pos.0 - 1,
                        start_column: third_token_start_pos.1,
                        end_line_number: current_line + last_non_whitespace_pos.0 - 1,
                        end_column: last_non_whitespace_pos.1 + 1,
                    });
                }

                statements.push((MmpStatement::MmpLabel, current_line));
            }
            "$axiom" => {
                if label.is_some() {
                    errors.push(DetailedError {
                        error_type: Error::MultipleMmpLabelsError,
                        start_line_number: current_line,
                        start_column: 1,
                        end_line_number: current_line + last_non_whitespace_pos.0 - 1,
                        end_column: last_non_whitespace_pos.1 + 1,
                    });
                }

                if let Some(axiom_label) = token_iter.next() {
                    if !util::is_valid_label(axiom_label) {
                        let second_token_start_pos = util::nth_token_start_pos(statement_str, 1);
                        let second_token_end_pos = util::nth_token_end_pos(statement_str, 1);

                        errors.push(DetailedError {
                            error_type: Error::InvalidLabelError,
                            start_line_number: current_line + second_token_start_pos.0 - 1,
                            start_column: second_token_start_pos.1,
                            end_line_number: current_line + second_token_end_pos.0 - 1,
                            end_column: second_token_end_pos.1 + 1,
                        });
                    }

                    label = Some(MmpLabel::Axiom(axiom_label));
                } else {
                    errors.push(DetailedError {
                        error_type: Error::MissingAxiomLabelError,
                        start_line_number: current_line,
                        start_column: 1,
                        end_line_number: current_line,
                        end_column: 9, // Length of "$theorem" + 1
                    });
                    // Make sure label is set to Some(_) so that future label statements will be flagged as errors
                    // Since return_info is false, the content within Some(_) does not matter
                    label = Some(MmpLabel::Theorem(""));
                }

                if token_iter.next().is_some() {
                    let third_token_start_pos = util::nth_token_start_pos(statement_str, 2);

                    errors.push(DetailedError {
                        error_type: Error::TooManyAxiomLabelTokensError,
                        start_line_number: current_line + third_token_start_pos.0 - 1,
                        start_column: third_token_start_pos.1,
                        end_line_number: current_line + last_non_whitespace_pos.0 - 1,
                        end_column: last_non_whitespace_pos.1 + 1,
                    });
                }

                statements.push((MmpStatement::MmpLabel, current_line));
            }
            "$theorem" => {
                if label.is_some() {
                    errors.push(DetailedError {
                        error_type: Error::MultipleMmpLabelsError,
                        start_line_number: current_line,
                        start_column: 1,
                        end_line_number: current_line + last_non_whitespace_pos.0 - 1,
                        end_column: last_non_whitespace_pos.1 + 1,
                    });
                }

                if let Some(theorem_label) = token_iter.next() {
                    if !util::is_valid_label(theorem_label) {
                        let second_token_start_pos = util::nth_token_start_pos(statement_str, 1);
                        let second_token_end_pos = util::nth_token_end_pos(statement_str, 1);

                        errors.push(DetailedError {
                            error_type: Error::InvalidLabelError,
                            start_line_number: current_line + second_token_start_pos.0 - 1,
                            start_column: second_token_start_pos.1,
                            end_line_number: current_line + second_token_end_pos.0 - 1,
                            end_column: second_token_end_pos.1 + 1,
                        });
                    }

                    label = Some(MmpLabel::Theorem(theorem_label));
                } else {
                    errors.push(DetailedError {
                        error_type: Error::MissingTheoremLabelError,
                        start_line_number: current_line,
                        start_column: 1,
                        end_line_number: current_line,
                        end_column: 9, // Length of "$theorem" + 1
                    });
                    // Make sure label is set to Some(_) so that future label statements will be flagged as errors
                    // Since return_info is false, the content within Some(_) does not matter
                    label = Some(MmpLabel::Theorem(""));
                }

                if token_iter.next().is_some() {
                    let third_token_start_pos = util::nth_token_start_pos(statement_str, 2);

                    errors.push(DetailedError {
                        error_type: Error::TooManyTheoremLabelTokensError,
                        start_line_number: current_line + third_token_start_pos.0 - 1,
                        start_column: third_token_start_pos.1,
                        end_line_number: current_line + last_non_whitespace_pos.0 - 1,
                        end_column: last_non_whitespace_pos.1 + 1,
                    });
                }

                statements.push((MmpStatement::MmpLabel, current_line));
            }
            "$d" => {
                // make sure there are at least two more tokens
                if token_iter.next().is_none() || token_iter.next().is_none() {
                    errors.push(DetailedError {
                        error_type: Error::ZeroOrOneSymbolDisjError,
                        start_line_number: current_line,
                        start_column: 1,
                        end_line_number: current_line + last_non_whitespace_pos.0 - 1,
                        end_column: last_non_whitespace_pos.1 + 1,
                    });
                } else {
                    distinct_vars.push(&statement_str[2..statement_str.len()]);
                }

                statements.push((MmpStatement::DistinctVar, current_line));
            }
            "$allowdiscouraged" => {
                if allow_discouraged {
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
                    let second_token_start_pos = util::nth_token_start_pos(statement_str, 1);

                    errors.push(DetailedError {
                        error_type: Error::TokensAfterAllowDiscouragedError,
                        start_line_number: current_line + second_token_start_pos.0 - 1,
                        start_column: second_token_start_pos.1,
                        end_line_number: current_line + last_non_whitespace_pos.0 - 1,
                        end_column: last_non_whitespace_pos.1 + 1,
                    });
                }

                statements.push((MmpStatement::AllowDiscouraged, current_line));
            }
            "$locateafter" => {
                if locate_after.is_some() {
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
                    let third_token_start_pos = util::nth_token_start_pos(statement_str, 2);

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

                statements.push((MmpStatement::LocateAfter, current_line));
            }
            "$locateafterconst" => {
                if locate_after.is_some() {
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
                    let third_token_start_pos = util::nth_token_start_pos(statement_str, 2);

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

                statements.push((MmpStatement::LocateAfter, current_line));
            }
            "$locateaftervar" => {
                if locate_after.is_some() {
                    errors.push(DetailedError {
                        error_type: Error::MultipleLocateAfterError,
                        start_line_number: current_line,
                        start_column: 1,
                        end_line_number: current_line + last_non_whitespace_pos.0 - 1,
                        end_column: last_non_whitespace_pos.1 + 1,
                    });
                }

                if let Some(locate_after_variable) = token_iter.next() {
                    locate_after = Some(LocateAfterRef::LocateAfterVar(locate_after_variable));
                } else {
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
                    let third_token_start_pos = util::nth_token_start_pos(statement_str, 2);

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

                statements.push((MmpStatement::LocateAfter, current_line));
            }
            t if t.starts_with('*') => {
                statements.push((MmpStatement::Comment, current_line));
                comments.push(&statement_str[1..statement_str.len()]);
            }
            t if t.starts_with('$') => {
                let first_token_end_pos = util::nth_token_end_pos(statement_str, 0);

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
                if prefix_parts.len() > 3 {
                    let first_token_end_pos = util::nth_token_end_pos(statement_str, 0);

                    errors.push(DetailedError {
                        error_type: Error::InvalidMmpStepPrefixFormatError,
                        start_line_number: current_line,
                        start_column: 1,
                        end_line_number: current_line + first_token_end_pos.0 - 1,
                        end_column: first_token_end_pos.1 + 1,
                    });
                }

                let prefix_step_name = prefix_parts.get(0).ok_or(Error::InternalLogicError)?;

                let mut advanced_unification = false;
                let mut is_hypothesis = false;

                let step_name = if prefix_step_name.starts_with('h') {
                    is_hypothesis = true;
                    prefix_step_name.split_at(1).1
                } else if prefix_step_name.starts_with('!') {
                    advanced_unification = true;
                    let new_step_name = prefix_step_name.split_at(1).1;
                    if new_step_name.starts_with('h') {
                        errors.push(DetailedError {
                            error_type: Error::InvalidMmpStepNameStartsWithHError,
                            start_line_number: current_line,
                            start_column: 2,
                            end_line_number: current_line,
                            end_column: new_step_name.len() as u32 + 2,
                        });
                    }
                    new_step_name
                } else {
                    prefix_step_name
                };

                if !is_valid_step_name(step_name) {
                    // || step_name == "" {
                    errors.push(DetailedError {
                        error_type: Error::InvalidMmpStepNameError,
                        start_line_number: current_line,
                        start_column: 1 + is_hypothesis as u32,
                        end_line_number: current_line,
                        end_column: is_hypothesis as u32 + step_name.len() as u32 + 1,
                    });
                }

                // if proof_lines.iter().any(|pl| pl.step_name == step_name) {
                //     errors.push(DetailedError {
                //         error_type: Error::DuplicateStepNameError,
                //         start_line_number: current_line,
                //         start_column: 1 + is_hypothesis as u32,
                //         end_line_number: current_line,
                //         end_column: is_hypothesis as u32 + step_name.len() as u32 + 1,
                //     });
                // }

                let hypotheses = if prefix_parts.len() == 3 {
                    *prefix_parts.get(1).ok_or(Error::InternalLogicError)?
                } else {
                    ""
                };

                // let mut hypotheses_parsed: Vec<Option<usize>> = Vec::new();

                // if !hypotheses.is_empty() {
                //     let mut start_column = 1 + prefix_step_name.len() as u32 + 1;
                //     for hyp in hypotheses.split(',') {
                //         if hyp == "?" {
                //             hypotheses_parsed.push(None);
                //         } else {
                //             match proof_lines
                //                 .iter()
                //                 .enumerate()
                //                 .find(|(_, pl)| pl.step_name == hyp)
                //             {
                //                 Some((i, _)) => hypotheses_parsed.push(Some(i)),
                //                 None => {
                //                     errors.push(DetailedError {
                //                         error_type: Error::HypNameDoesntExistError,
                //                         start_line_number: current_line,
                //                         start_column: start_column,
                //                         end_line_number: current_line,
                //                         end_column: start_column + hyp.len() as u32,
                //                     });
                //                 }
                //             }
                //         }
                //         start_column += hyp.len() as u32 + 1;
                //     }
                // }

                let step_ref = if prefix_parts.len() == 3 {
                    *prefix_parts.get(2).ok_or(Error::InternalLogicError)?
                } else if prefix_parts.len() == 2 {
                    *prefix_parts.get(1).ok_or(Error::InternalLogicError)?
                } else {
                    ""
                };

                // if is_hypothesis
                //     && step_ref != ""
                //     && proof_lines
                //         .iter()
                //         .any(|pl| pl.is_hypothesis && pl.step_ref == step_ref)
                // {
                //     let start_column =
                //         1 + prefix_step_name.len() as u32 + 1 + (prefix_parts.len() == 3) as u32;

                //     errors.push(DetailedError {
                //         error_type: Error::DuplicateHypLabelsError,
                //         start_line_number: current_line,
                //         start_column: start_column,
                //         end_line_number: current_line,
                //         end_column: start_column + step_ref.len() as u32,
                //     });
                // }

                let expression = &statement_str[step_prefix.len()..statement_str.len()];

                statements.push((MmpStatement::ProofLine, current_line));
                proof_lines.push(ProofLine {
                    advanced_unification,
                    is_hypothesis,
                    step_name,
                    hypotheses,
                    step_ref,
                    expression,
                });

                // if token_iter.next().is_none() {
                //     errors.push(DetailedError {
                //         error_type: Error::MissingMmpStepExpressionError,
                //         start_line_number: current_line,
                //         start_column: last_non_whitespace_pos.1,
                //         end_line_number: current_line,
                //         end_column: last_non_whitespace_pos.1 + 1,
                //     });
                // }

                // // This will be overritten if expression is convertable to a parse tree
                // let mut parse_tree = ParseTree {
                //     rule: 0,
                //     nodes: Vec::new(),
                // };

                // match mm_data
                //     .optimized_data
                //     .symbol_number_mapping
                //     .expression_to_parse_tree(expression, &mm_data.optimized_data.grammar)
                // {
                //     Ok(pt) => parse_tree = pt,
                //     Err(Error::MissingExpressionError) => {
                //         errors.push(DetailedError {
                //             error_type: Error::MissingMmpStepExpressionError,
                //             start_line_number: current_line,
                //             start_column: last_non_whitespace_pos.1 + 1,
                //             end_line_number: current_line,
                //             end_column: last_non_whitespace_pos.1 + 2,
                //         });
                //     }
                //     Err(Error::NonSymbolInExpressionError) => {
                //         errors.append(&mut calc_non_symbol_in_expression_errors(
                //             expression,
                //             &mm_data.optimized_data.symbol_number_mapping,
                //             current_line,
                //             step_prefix.len() as u32,
                //         ));
                //     }
                //     Err(Error::ExpressionParseError) => {
                //         errors.push(DetailedError {
                //             error_type: Error::ExpressionParseError,
                //             start_line_number: current_line,
                //             start_column: last_non_whitespace_pos.1 + 1,
                //             end_line_number: current_line,
                //             end_column: last_non_whitespace_pos.1 + 2,
                //         });
                //     }
                //     Err(_) => {
                //         return Err(Error::InternalLogicError);
                //     }
                // };
            }
        }

        current_line += util::new_lines_in_str(statement_str);
    }

    Ok(if errors.is_empty() {
        MmpParserStage2::Success(MmpParserStage2Success {
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
        MmpParserStage2::Fail(MmpParserStage2Fail { errors })
    })
}

fn is_valid_step_name(step_name: &str) -> bool {
    step_name
        .chars()
        .all(|c| matches!(c, 'a'..='z' | 'A'..='Z' | '0'..='9' ))
}

// fn calc_non_symbol_in_expression_errors(
//     expression: &str,
//     symbol_number_mapping: &SymbolNumberMapping,
//     first_line: u32,
//     first_line_offset: u32,
// ) -> Vec<DetailedError> {
//     let mut errors = Vec::new();

//     let mut line = first_line;
//     let mut column = first_line_offset;

//     let mut current_token_start_column = column;

//     let mut current_token = String::new();

//     let mut seeing_token = false;

//     for char in expression.chars() {
//         column += 1;

//         if char == '\n' {
//             line += 1;
//             column = 0;
//         }

//         if char.is_ascii_whitespace() {
//             if seeing_token {
//                 if current_token.starts_with('$')
//                     || symbol_number_mapping.numbers.get(&current_token).is_none()
//                 {
//                     errors.push(DetailedError {
//                         error_type: Error::NonSymbolInExpressionError,
//                         start_line_number: line,
//                         start_column: current_token_start_column,
//                         end_line_number: line,
//                         end_column: column,
//                     });
//                 }

//                 current_token = String::new();
//             }
//             seeing_token = false;
//         } else {
//             if !seeing_token {
//                 current_token_start_column = column;
//             }
//             seeing_token = true;
//             current_token.push(char)
//         }
//     }

//     errors
// }
