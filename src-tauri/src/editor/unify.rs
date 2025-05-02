use tauri::async_runtime::Mutex;

use crate::{AppState, Error};

// #[tauri::command]
// pub async fn unify_and_format(text: &str) -> Result<String, Error> {
//     let mut res = String::new();

//     let statements = parse_mmp::text_to_statements(text)?;
//     let mmp_structured_info = parse_mmp::statements_to_mmp_structured_info(statements)?;

//     if mmp_structured_info.statement_out_of_place() {
//         return Err(Error::StatementOutOfPlaceError);
//     }

//     if let Some(label) = mmp_structured_info.theorem_label.as_ref() {
//         res.push_str("$theorem ");
//         res.push_str(label);
//         res.push('\n');
//     }

//     if let Some(label) = mmp_structured_info.axiom_label.as_ref() {
//         res.push_str("$axiom ");
//         res.push_str(label);
//         res.push('\n');
//     }

//     Ok(res)
// }

/**
A collection of all the data needed to unify (and format) an mmp file.

Field `statements` contains the order of the statements and information about the proof steps and comments.
All other field contain information that is instead needed globally.
*/
#[derive(Debug)]
struct MmpInfoStructuredForUnify<'a> {
    pub constants: Option<&'a str>,
    pub variables: Vec<&'a str>, // Each str may contain mulitple vars
    pub floating_hypotheses: Vec<&'a str>,
    pub label: Option<MmpLabel<'a>>,
    pub allow_discouraged: bool,
    pub locate_after: Option<LocateAfterRef<'a>>,
    pub distinct_vars: Vec<&'a str>,
    pub statements: Vec<MmpStatement<'a>>,
}

#[derive(Debug)]
enum MmpLabel<'a> {
    Theorem(&'a str),
    Axiom(&'a str),
    Header { header_pos: &'a str, title: &'a str },
}

#[derive(Debug)]
enum LocateAfterRef<'a> {
    LocateAfter(&'a str),
    LocateAfterConst(&'a str),
    LocateAfterVar(&'a str),
}

#[derive(Debug)]
enum MmpStatement<'a> {
    MmpLabel,
    DistinctVar,
    AllowDiscouraged,
    LocateAfter,
    Constant,
    Variable,
    FloatingHypohesis,
    ProofLine(ProofLine<'a>),
    Comment(&'a str),
}

#[derive(Debug)]
struct ProofLine<'a> {
    is_hypothesis: bool,
    step_name: &'a str,
    hypotheses: &'a str,
    step_ref: &'a str,
    expression: &'a str,
}

#[tauri::command]
pub async fn unify(state: tauri::State<'_, Mutex<AppState>>, text: &str) -> Result<String, Error> {
    let mut app_state = state.lock().await;
    let mm_data = app_state.metamath_data.as_mut().ok_or(Error::NoMmDbError)?;

    if !text.is_ascii() {
        return Err(Error::InvalidCharactersError);
    }

    let mut res = String::new();

    let (whitespace_before_first_statement, statement_strs) = text_to_statement_strs(text)?;

    res.push_str(whitespace_before_first_statement);

    let mmp_info_structured_for_unify =
        statement_strs_to_mmp_info_structured_for_unify(&statement_strs)?;

    println!("{:?}", mmp_info_structured_for_unify);

    let mut previous_proof_lines: Vec<&ProofLine> = Vec::new();

    for (i, &statement_str) in statement_strs.iter().enumerate() {
        let mut statement_already_added = false;

        if let Some(MmpStatement::ProofLine(proof_line)) =
            mmp_info_structured_for_unify.statements.get(i)
        {
            let parse_tree = mm_data
                .optimized_data
                .symbol_number_mapping
                .expression_to_parse_tree(proof_line.expression, &mm_data.optimized_data.grammar)?;

            if proof_line.step_ref == "" {
                for theorem in mm_data.database_header.theorem_iter() {
                    if let Some(theorem_data) =
                        mm_data.optimized_data.theorem_data.get(&theorem.label)
                    {
                        if theorem_data.hypotheses_parsed.is_empty()
                            && theorem_data
                                .assertion_parsed
                                .is_substitution(&parse_tree, &mm_data.optimized_data.grammar)?
                        {
                            let mut new_statement_string = format!(
                                "{}{}:{}:{}",
                                if proof_line.is_hypothesis { "h" } else { "" },
                                proof_line.step_name,
                                proof_line.hypotheses,
                                theorem.label,
                            );

                            if new_statement_string.len() < 20 {
                                new_statement_string.push_str(
                                    "                    "
                                        .split_at_checked(20 - new_statement_string.len())
                                        .ok_or(Error::InternalLogicError)?
                                        .0,
                                );
                            } else {
                                new_statement_string.push_str("\n                    ");
                            }

                            new_statement_string.push_str(proof_line.expression.trim_ascii_start());
                            res.push_str(&new_statement_string);
                            statement_already_added = true;
                            break;
                        }
                    }
                }
            }

            previous_proof_lines.push(proof_line);
        }

        if !statement_already_added {
            res.push_str(statement_str);
        }
    }

    Ok(res)
}

// If successful, returns a tuple (a,b) where a is the whitespace before the first line and b is a vec of all the lines
fn text_to_statement_strs(text: &str) -> Result<(&str, Vec<&str>), Error> {
    let mut statements = Vec::new();

    let mut text_i: usize = 0;
    let text_bytes = text.as_bytes();

    while text_bytes
        .get(text_i)
        .is_some_and(|c| c.is_ascii_whitespace())
    {
        text_i += 1;
    }

    let whitespace_before_first_statement = text.get(0..text_i).ok_or(Error::InternalLogicError)?;

    if text_i != 0 && text_bytes.get(text_i - 1).is_some_and(|c| *c != b'\n') {
        return Err(Error::WhitespaceBeforeFirstTokenError);
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

    Ok((whitespace_before_first_statement, statements))
}

fn statement_strs_to_mmp_info_structured_for_unify<'a>(
    statement_strs: &Vec<&'a str>,
) -> Result<MmpInfoStructuredForUnify<'a>, Error> {
    let mut label: Option<MmpLabel<'a>> = None;
    let mut allow_discouraged: bool = false;
    let mut locate_after: Option<LocateAfterRef<'a>> = None;
    let mut distinct_vars: Vec<&'a str> = Vec::new();
    let mut constants: Option<&'a str> = None;
    let mut variables: Vec<&'a str> = Vec::new();
    let mut floating_hypotheses: Vec<&'a str> = Vec::new();
    let mut statements: Vec<MmpStatement<'a>> = Vec::with_capacity(statement_strs.len());

    for &statement_str in statement_strs {
        let mut token_iter = statement_str.split_ascii_whitespace();

        match token_iter.next().ok_or(Error::InternalLogicError)? {
            "$c" => {
                if constants.is_some() {
                    return Err(Error::TooManyConstStatementsError);
                }

                constants = Some(
                    statement_str
                        .get(3..statement_str.len())
                        .ok_or(Error::InternalLogicError)?,
                );

                if token_iter.next().is_none() {
                    return Err(Error::EmptyConstStatementError);
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
                    return Err(Error::EmptyVarStatementError);
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
                if token_iter.next().is_none()
                    || token_iter.next().is_none()
                    || token_iter.next().is_none()
                    || token_iter.next().is_some()
                {
                    return Err(Error::FloatHypStatementFormatError);
                }

                statements.push(MmpStatement::FloatingHypohesis);
            }
            "$theorem" => {
                if label.is_some() {
                    return Err(Error::StatementOutOfPlaceError);
                }

                label = Some(MmpLabel::Theorem(
                    token_iter.next().ok_or(Error::MissingTheoremLabelError)?,
                ));

                if token_iter.next().is_some() {
                    return Err(Error::TooManyTheoremLabelTokensError);
                }

                statements.push(MmpStatement::MmpLabel);
            }
            "$axiom" => {
                if label.is_some() {
                    return Err(Error::MultipleAxiomLabelError);
                }

                label = Some(MmpLabel::Axiom(
                    token_iter.next().ok_or(Error::MissingAxiomLabelError)?,
                ));

                if token_iter.next().is_some() {
                    return Err(Error::TooManyAxiomLabelTokensError);
                }

                statements.push(MmpStatement::MmpLabel);
            }
            "$header" => {
                if label.is_some() {
                    return Err(Error::MultipleHeaderStatementError);
                }

                let header_pos = token_iter.next().ok_or(Error::TooFewHeaderTokensError)?;

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
                    .ok_or(Error::TooFewHeaderTokensError)?;

                // make sure there follows at least one token
                if token_iter.next().is_none() {
                    return Err(Error::TooFewHeaderTokensError);
                }

                label = Some(MmpLabel::Header { header_pos, title });

                statements.push(MmpStatement::MmpLabel);
            }
            "$d" => {
                distinct_vars.push(
                    statement_str
                        .get(3..statement_str.len())
                        .ok_or(Error::InternalLogicError)?,
                );

                // make sure there are at least two more tokens
                if token_iter.next().is_none() || token_iter.next().is_none() {
                    return Err(Error::ZeroOrOneSymbolDisjError);
                }

                statements.push(MmpStatement::DistinctVar);
            }
            "$allowdiscouraged" => {
                if allow_discouraged {
                    return Err(Error::MultipleAllowDiscouragedError);
                }

                allow_discouraged = true;

                if token_iter.next().is_some() {
                    return Err(Error::TokensAfterAllowDiscouragedError);
                }

                statements.push(MmpStatement::AllowDiscouraged);
            }
            "$locateafter" => {
                if locate_after.is_some() {
                    return Err(Error::MultipleLocateAfterError);
                }

                locate_after = Some(LocateAfterRef::LocateAfter(
                    token_iter
                        .next()
                        .ok_or(Error::MissingLocateAfterLabelError)?,
                ));

                if token_iter.next().is_some() {
                    return Err(Error::TooManyLocateAfterTokensError);
                }

                statements.push(MmpStatement::LocateAfter);
            }
            "$locateafterconst" => {
                if locate_after.is_some() {
                    return Err(Error::MultipleLocateAfterError);
                }

                locate_after = Some(LocateAfterRef::LocateAfterConst(
                    token_iter
                        .next()
                        .ok_or(Error::MissingLocateAfterLabelError)?,
                ));

                if token_iter.next().is_some() {
                    return Err(Error::TooManyLocateAfterTokensError);
                }

                statements.push(MmpStatement::LocateAfter);
            }
            "$locateaftervar" => {
                if locate_after.is_some() {
                    return Err(Error::MultipleLocateAfterError);
                }

                locate_after = Some(LocateAfterRef::LocateAfterVar(
                    token_iter
                        .next()
                        .ok_or(Error::MissingLocateAfterLabelError)?,
                ));

                if token_iter.next().is_some() {
                    return Err(Error::TooManyLocateAfterTokensError);
                }

                statements.push(MmpStatement::LocateAfter);
            }
            t if t.starts_with('*') => {
                statements.push(MmpStatement::Comment(
                    statement_str
                        .get(1..statement_str.len())
                        .ok_or(Error::InternalLogicError)?,
                ));
            }
            t if t.starts_with('$') => return Err(Error::InvalidDollarTokenError),
            step_prefix => {
                let prefix_parts: Vec<&str> = step_prefix.split(':').collect();
                if prefix_parts.len() != 3 {
                    return Err(Error::InvalidMmj2StepPrefixError);
                }

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
                    return Err(Error::InvalidMmj2StepPrefixError);
                }

                let hypotheses = *prefix_parts.get(1).unwrap();

                let step_ref = *prefix_parts.get(2).unwrap();

                let expression = statement_str
                    .get(step_prefix.len()..statement_str.len())
                    .ok_or(Error::InternalLogicError)?;

                if token_iter.next().is_none() {
                    return Err(Error::MissingMmj2StepExpressionError);
                }

                statements.push(MmpStatement::ProofLine(ProofLine {
                    is_hypothesis,
                    step_name,
                    hypotheses,
                    step_ref,
                    expression,
                }));
            }
        }
    }

    Ok(MmpInfoStructuredForUnify {
        label,
        allow_discouraged,
        locate_after,
        distinct_vars,
        constants,
        variables,
        floating_hypotheses,
        statements,
    })
}
