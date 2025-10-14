use std::fs;

use tauri::async_runtime::Mutex;

use crate::{
    metamath::{
        mm_parser::{MmParser, StatementProcessed},
        mmp_parser::{
            self, LocateAfterRef, MmpParserStage1, MmpParserStage2, MmpParserStage3,
            MmpParserStage3Success, MmpParserStage4,
        },
    },
    model::{DatabaseElement, Header, Hypothesis, Statement, Theorem},
    AppState, Error,
};

#[tauri::command]
pub async fn add_to_database_preview(
    state: tauri::State<'_, Mutex<AppState>>,
    text: &str,
) -> Result<Option<(String, String)>, Error> {
    let app_state = state.lock().await;
    let mm_data = app_state.metamath_data.as_ref().ok_or(Error::NoMmDbError)?;
    let settings = &app_state.settings;

    let stage_0 = mmp_parser::new(text);

    let MmpParserStage1::Success(stage_1_success) = stage_0.next_stage()? else {
        return Ok(None);
    };

    let MmpParserStage2::Success(stage_2_success) = stage_1_success.next_stage()? else {
        return Ok(None);
    };

    let MmpParserStage3::Success(MmpParserStage3Success::Theorem(stage_3_theorem)) =
        stage_2_success.next_stage(&stage_1_success, mm_data)?
    else {
        return Ok(None);
    };

    let MmpParserStage4::Success(stage_4_success) =
        stage_3_theorem.next_stage(&stage_1_success, &stage_2_success, mm_data)?
    else {
        return Ok(None);
    };

    let stage_5 = stage_4_success.next_stage(&stage_2_success, &stage_3_theorem, mm_data)?;

    let stage_6 = stage_5.next_stage(&stage_4_success, mm_data, settings)?;

    let proof = if stage_3_theorem.is_axiom {
        None
    } else {
        let Some(proof) = stage_6.proof else {
            return Ok(None);
        };

        Some(proof)
    };

    let theorem = Theorem {
        label: stage_3_theorem.label.to_string(),
        description: stage_2_success
            .comments
            .into_iter()
            .next()
            .map(|c| c.to_string())
            .unwrap_or(String::new()),
        // todo: fix temp variables and floating hypotheses
        temp_variables: stage_3_theorem.temp_variable_statements,
        temp_floating_hypotheses: stage_3_theorem.temp_floating_hypotheses,
        distincts: stage_2_success
            .distinct_vars
            .into_iter()
            .map(|s| s.to_string())
            .collect(),
        hypotheses: stage_5
            .unify_result
            .iter()
            .filter(|ul| ul.is_hypothesis)
            .map(|ul| {
                Ok(Hypothesis {
                    label: ul.step_ref.to_string(),
                    expression: ul
                        .parse_tree
                        .as_ref()
                        .ok_or(Error::InternalLogicError)?
                        .to_expression(
                            &mm_data.optimized_data.symbol_number_mapping,
                            &mm_data.optimized_data.grammar,
                        )?,
                })
            })
            .collect::<Result<Vec<Hypothesis>, Error>>()?,
        assertion: stage_5
            .unify_result
            .iter()
            .find(|ul| ul.step_name == "qed")
            .ok_or(Error::InternalLogicError)?
            .parse_tree
            .as_ref()
            .ok_or(Error::InternalLogicError)?
            .to_expression(
                &mm_data.optimized_data.symbol_number_mapping,
                &mm_data.optimized_data.grammar,
            )?,
        proof,
    };

    Ok(Some(add_statement(
        &mm_data.database_path,
        &mm_data.database_header,
        stage_2_success.locate_after,
        Statement::TheoremStatement(theorem),
    )?))
}

fn add_statement(
    file_path: &str,
    header: &Header,
    locate_after: Option<LocateAfterRef>,
    statement: Statement,
) -> Result<(String, String), Error> {
    match locate_after {
        Some(loc_after) => add_statement_locate_after(file_path, header, loc_after, statement),
        None => add_statement_at_end(file_path, /*header,*/ statement),
    }
}

fn add_statement_locate_after(
    file_path: &str,
    header: &Header,
    locate_after: LocateAfterRef,
    statement: Statement,
) -> Result<(String, String), Error> {
    add_statement_locate_after_file(file_path, header, locate_after, &statement)
    // match add_statement_locate_after_memory(header, locate_after, statement) {
    //     None => Ok(()),
    //     Some(_) => Err(Error::InvalidLocateAfterError),
    // }
}

// fn add_statement_locate_after_memory(
//     header: &mut Header,
//     locate_after: &LocateAfterRef,
//     mut statement: Statement,
// ) -> Option<Statement> {
//     for i in 0..header.content.len() {
//         match locate_after {
//             LocateAfterRef::LocateAfterConst(s) => {
//                 if let Some(Statement::ConstantStatement(constants)) = header.content.get(i) {
//                     if constants.iter().find(|c| c.symbol == *s).is_some() {
//                         header.content.insert(i + 1, statement);
//                         return None;
//                     }
//                 }
//             }
//             LocateAfterRef::LocateAfterVar(s) => {
//                 if let Some(Statement::VariableStatement(variables)) = header.content.get(i) {
//                     if variables.iter().find(|c| c.symbol == *s).is_some() {
//                         header.content.insert(i + 1, statement);
//                         return None;
//                     }
//                 }
//             }
//             LocateAfterRef::LocateAfter(s) => {
//                 if let Some(Statement::TheoremStatement(theorem)) = header.content.get(i) {
//                     if theorem.label == *s {
//                         header.content.insert(i + 1, statement);
//                         return None;
//                     }
//                 } else if let Some(Statement::FloatingHypohesisStatement(floating_hypothesis)) =
//                     header.content.get(i)
//                 {
//                     if floating_hypothesis.label == *s {
//                         header.content.insert(i + 1, statement);
//                         return None;
//                     }
//                 }
//             }
//         }
//     }

//     for subheader in &mut header.subheaders {
//         statement = match add_statement_locate_after_memory(subheader, locate_after, statement) {
//             Some(s) => s,
//             None => return None,
//         };
//     }

//     Some(statement)
// }

fn add_statement_locate_after_file(
    file_path: &str,
    header: &Header,
    locate_after: LocateAfterRef,
    statement: &Statement,
) -> Result<(String, String), Error> {
    let mut mm_parser = MmParser::new(file_path, None, None)?;
    let header_iter = header.locate_after_iter(Some(locate_after));

    for database_element in header_iter {
        match database_element {
            DatabaseElement::Header(_, _) => loop {
                if let Some(StatementProcessed::HeaderStatement) =
                    mm_parser.process_next_statement()?
                {
                    break;
                }
            },
            DatabaseElement::Statement(statement) => match statement {
                Statement::CommentStatement(_) => loop {
                    if let Some(StatementProcessed::CommentStatement) =
                        mm_parser.process_next_statement()?
                    {
                        if mm_parser.get_scope() == 0 {
                            break;
                        }
                    }
                },
                Statement::ConstantStatement(_) => loop {
                    if let Some(StatementProcessed::ConstantStatement) =
                        mm_parser.process_next_statement()?
                    {
                        break;
                    }
                },
                Statement::VariableStatement(_) => loop {
                    if let Some(StatementProcessed::VariableStatement) =
                        mm_parser.process_next_statement()?
                    {
                        if mm_parser.get_scope() == 0 {
                            break;
                        }
                    }
                },
                Statement::FloatingHypohesisStatement(_) => loop {
                    if let Some(StatementProcessed::FloatingHypothesisStatement) =
                        mm_parser.process_next_statement()?
                    {
                        if mm_parser.get_scope() == 0 {
                            break;
                        }
                    }
                },
                Statement::TheoremStatement(_) => loop {
                    if let Some(StatementProcessed::TheoremStatement) =
                        mm_parser.process_next_statement()?
                    {
                        break;
                    }
                },
            },
        }
    }

    while mm_parser.get_scope() != 0 {
        match mm_parser.process_next_statement()? {
            Some(StatementProcessed::ClosingScopeStatement)
            | Some(StatementProcessed::OpeningScopeStatement)
            | Some(StatementProcessed::VariableStatement)
            | Some(StatementProcessed::FloatingHypothesisStatement)
            | Some(StatementProcessed::DistinctVariableStatement)
            | Some(StatementProcessed::EssentialHypothesisStatement)
            | Some(StatementProcessed::CommentStatement) => {}
            Some(StatementProcessed::TheoremStatement)
            | Some(StatementProcessed::HeaderStatement) => {
                return Err(Error::AddingToInnerScopeError)
            }
            // Should never happen
            Some(StatementProcessed::ConstantStatement) | None => {}
        }
    }

    let (file_content, next_token_i) = mm_parser.consume_early_and_return_file_content();

    let mut new_file_content = file_content.clone();

    new_file_content.insert_str(next_token_i, "\n\n");

    statement.insert_mm_string(&mut new_file_content, next_token_i + 2);

    // fs::write(file_path, &file_content).or(Err(Error::FileWriteError))?;

    Ok((file_content, new_file_content))
}

fn add_statement_at_end(
    file_path: &str,
    // header: &Header,
    statement: Statement,
) -> Result<(String, String), Error> {
    add_statement_at_end_file(file_path, &statement)
    // add_statement_at_end_memory(header, statement);
    // Ok(())
}

// fn add_statement_at_end_memory(header: &mut Header, statement: Statement) {
//     let mut last_header = header;

//     while last_header.subheaders.len() > 0 {
//         last_header = last_header.subheaders.last_mut().unwrap();
//     }

//     last_header.content.push(statement);
// }

fn add_statement_at_end_file(
    file_path: &str,
    statement: &Statement,
) -> Result<(String, String), Error> {
    let file_content = fs::read_to_string(file_path).or(Err(Error::FileReadError))?;
    let mut new_file_content = file_content.clone();

    loop {
        match new_file_content.pop() {
            Some(c) if c.is_whitespace() => {}
            Some(c) => {
                new_file_content.push(c);
                break;
            }
            None => break,
        }
    }

    if !new_file_content.is_empty() {
        new_file_content.push_str("\n\n");
    }

    statement.write_mm_string(&mut new_file_content);

    // fs::write(file_path, new_file_content).or(Err(Error::FileWriteError))?;

    Ok((file_content, new_file_content))
}
