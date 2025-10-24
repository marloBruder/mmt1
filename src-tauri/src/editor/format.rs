use crate::{
    metamath::mmp_parser::{
        self, calc_indention::calc_indention, LocateAfterRef, MmpLabel, MmpParserStage1,
        MmpParserStage2, MmpStatement,
    },
    util, Error,
};

#[tauri::command]
pub async fn format(text: &str) -> Result<Option<String>, Error> {
    format_mmp_file(text)
}

pub fn format_mmp_file(text: &str) -> Result<Option<String>, Error> {
    let stage_0 = mmp_parser::new(text);

    let MmpParserStage1::Success(stage_1_success) = stage_0.next_stage()? else {
        return Ok(None);
    };

    let MmpParserStage2::Success(stage_2_success) = stage_1_success.next_stage()? else {
        return Ok(None);
    };

    let mut result_text = String::new();

    // for _ in 0..(stage_1_success.number_of_lines_before_first_statement - 1) {
    //     result_text.push('\n');
    // }

    let mut variables_iter = stage_2_success.variables.into_iter();
    let mut floating_hypotheses_iter = stage_2_success.floating_hypotheses.into_iter();
    let mut distinct_var_iter = stage_2_success.distinct_vars.into_iter();
    let mut indention_iter = calc_indention(&stage_2_success.proof_lines)?.into_iter();
    let mut proof_line_iter = stage_2_success.proof_lines.into_iter();

    for (i, (&statement, (statement_type, _))) in stage_1_success
        .statements
        .iter()
        .zip(stage_2_success.statements.iter())
        .enumerate()
    {
        match statement_type {
            MmpStatement::Constant => {
                let constants = stage_2_success.constants.ok_or(Error::InternalLogicError)?;

                result_text.push_str("$c");

                for c in constants.split_ascii_whitespace() {
                    result_text.push(' ');
                    result_text.push_str(c);
                }
            }
            MmpStatement::Variable => {
                let variables = variables_iter.next().ok_or(Error::InternalLogicError)?;

                result_text.push_str("$v");

                for v in variables.split_ascii_whitespace() {
                    result_text.push(' ');
                    result_text.push_str(v);
                }
            }
            MmpStatement::FloatingHypohesis => {
                let floating_hypothesis = floating_hypotheses_iter
                    .next()
                    .ok_or(Error::InternalLogicError)?;

                result_text.push_str("$f");

                for fh_part in floating_hypothesis.split_ascii_whitespace() {
                    result_text.push(' ');
                    result_text.push_str(fh_part);
                }
            }
            MmpStatement::MmpLabel => {
                let label = stage_2_success.label.ok_or(Error::InternalLogicError)?;

                match label {
                    MmpLabel::Header { header_path, title } => {
                        result_text.push_str("$header ");
                        result_text.push_str(header_path);

                        for title_part in title.split_ascii_whitespace() {
                            result_text.push(' ');
                            result_text.push_str(title_part);
                        }
                    }
                    MmpLabel::Axiom(label) => {
                        result_text.push_str("$axiom ");
                        result_text.push_str(label);
                    }
                    MmpLabel::Theorem(label) => {
                        result_text.push_str("$theorem ");
                        result_text.push_str(label);
                    }
                }
            }
            MmpStatement::DistinctVar => {
                let distinct_vars = distinct_var_iter.next().ok_or(Error::InternalLogicError)?;

                result_text.push_str("$d");

                for d in distinct_vars.split_ascii_whitespace() {
                    result_text.push(' ');
                    result_text.push_str(d);
                }
            }
            MmpStatement::AllowDiscouraged => {
                result_text.push_str("$allowdiscouraged");
            }
            MmpStatement::AllowIncomplete => {
                result_text.push_str("$allowincomplete");
            }
            MmpStatement::LocateAfter => {
                let locate_after = stage_2_success
                    .locate_after
                    .ok_or(Error::InternalLogicError)?;

                match locate_after {
                    LocateAfterRef::LocateAfterStart => {
                        result_text.push_str("$locateafterstart");
                    }
                    LocateAfterRef::LocateAfterHeader(header_path) => {
                        result_text.push_str("$locateafterheader ");
                        result_text.push_str(header_path);
                    }
                    LocateAfterRef::LocateAfterComment(comment_path) => {
                        result_text.push_str("$locateaftercomment ");
                        result_text.push_str(comment_path);
                    }
                    LocateAfterRef::LocateAfterConst(c) => {
                        result_text.push_str("$locateafterconst ");
                        result_text.push_str(c);
                    }
                    LocateAfterRef::LocateAfterVar(v) => {
                        result_text.push_str("$locateaftervar ");
                        result_text.push_str(v);
                    }
                    LocateAfterRef::LocateAfter(label) => {
                        result_text.push_str("$locateafter ");
                        result_text.push_str(label);
                    }
                }
            }
            MmpStatement::Proof | MmpStatement::Comment => {
                result_text.push_str(statement.trim_ascii_end());
            }
            MmpStatement::ProofLine => {
                let proof_line = proof_line_iter.next().ok_or(Error::InternalLogicError)?;
                let indention = indention_iter.next().ok_or(Error::InternalLogicError)?;

                if proof_line.advanced_unification {
                    result_text.push('!');
                }
                if proof_line.is_hypothesis {
                    result_text.push('h');
                }

                result_text.push_str(proof_line.step_name);
                result_text.push(':');
                result_text.push_str(proof_line.hypotheses);
                result_text.push(':');
                result_text.push_str(proof_line.step_ref);

                let prefix_len = statement
                    .split_ascii_whitespace()
                    .next()
                    .ok_or(Error::InternalLogicError)?
                    .len() as u32;
                if prefix_len >= 20 + indention - 1 {
                    result_text.push('\n');
                    result_text.push_str(util::spaces(20));
                    result_text.push_str(util::spaces(indention - 1));
                } else {
                    result_text.push_str(util::spaces(indention - 1 + 20 - prefix_len));
                }

                result_text.push_str(&util::str_to_space_seperated_string(proof_line.expression));
            }
        }

        if i != stage_2_success.statements.len() - 1 {
            for _ in 0..new_lines_at_end_of_str_capped(statement) {
                result_text.push('\n');
            }
        } else {
            result_text.push('\n');
        }
    }

    Ok(Some(result_text))
}

fn new_lines_at_end_of_str_capped(str: &str) -> u32 {
    std::cmp::min(2, util::new_lines_at_end_of_str(str))
}
