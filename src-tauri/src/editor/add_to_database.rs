use std::fs;

use tauri::async_runtime::Mutex;

use crate::{
    metamath::{
        mm_parser::{html_validation, MmParser, StatementProcessed},
        mmp_parser::{
            self, LocateAfterRef, MmpParserStage1, MmpParserStage2, MmpParserStage2Success,
            MmpParserStage3, MmpParserStage3Header, MmpParserStage3Success, MmpParserStage3Theorem,
            MmpParserStage4, MmpParserStage5, MmpParserStage6,
        },
    },
    model::{DatabaseElement, Header, HeaderPath, Hypothesis, MetamathData, Statement, Theorem},
    util, AppState, Error, ProofFormatOption,
};

#[tauri::command]
pub async fn add_to_database_preview(
    state: tauri::State<'_, Mutex<AppState>>,
    text: &str,
    override_proof_format: Option<ProofFormatOption>,
) -> Result<(String, String), Error> {
    let app_state = state.lock().await;
    let mm_data = app_state.metamath_data.as_ref().ok_or(Error::NoMmDbError)?;
    let settings = &app_state.settings;

    let stage_0 = mmp_parser::new(text);

    let MmpParserStage1::Success(stage_1_success) = stage_0.next_stage()? else {
        return Err(Error::CantAddToDatabaseError);
    };

    let MmpParserStage2::Success(stage_2_success) = stage_1_success.next_stage()? else {
        return Err(Error::CantAddToDatabaseError);
    };

    let MmpParserStage3::Success(stage_3_success) =
        stage_2_success.next_stage(&stage_1_success, mm_data)?
    else {
        return Err(Error::CantAddToDatabaseError);
    };

    let stage_3_theorem = match stage_3_success {
        MmpParserStage3Success::Empty => return Err(Error::MmpFileEmptyError),
        MmpParserStage3Success::Header(stage_3_header) => {
            return Ok(add_header_preview(
                &mm_data.database_path,
                &mm_data.database_header,
                stage_3_header,
            )?)
        }
        MmpParserStage3Success::Comment(stage_3_comment) => {
            return Ok(add_statement_preview(
                &mm_data.database_path,
                &mm_data.database_header,
                stage_2_success.locate_after,
                Statement::CommentStatement(stage_3_comment.comment),
            )?);
        }
        MmpParserStage3Success::Constants(constants) => {
            return Ok(add_statement_preview(
                &mm_data.database_path,
                &mm_data.database_header,
                stage_2_success.locate_after,
                Statement::ConstantStatement(constants),
            )?);
        }
        MmpParserStage3Success::Variables(variables) => {
            return Ok(add_statement_preview(
                &mm_data.database_path,
                &mm_data.database_header,
                stage_2_success.locate_after,
                Statement::VariableStatement(variables),
            )?);
        }
        MmpParserStage3Success::FloatingHypohesis(floating_hypothesis) => {
            return Ok(add_statement_preview(
                &mm_data.database_path,
                &mm_data.database_header,
                stage_2_success.locate_after,
                Statement::FloatingHypohesisStatement(floating_hypothesis),
            )?);
        }
        MmpParserStage3Success::Theorem(s3t) => s3t,
    };

    let MmpParserStage4::Success(stage_4_success) =
        stage_3_theorem.next_stage(&stage_1_success, &stage_2_success, mm_data)?
    else {
        return Err(Error::CantAddToDatabaseError);
    };

    let stage_5 = stage_4_success.next_stage(&stage_2_success, &stage_3_theorem, mm_data)?;

    let mut new_settings = settings.clone();
    if let Some(pf) = override_proof_format {
        new_settings.proof_format = pf;
    }

    let stage_6 = stage_5.next_stage(&stage_4_success, mm_data, &new_settings)?;

    let locate_after = stage_2_success.locate_after;

    let Some(theorem) =
        mmp_parser_stages_to_theorem(stage_2_success, stage_3_theorem, stage_5, stage_6, mm_data)?
    else {
        return Err(Error::CantAddToDatabaseError);
    };

    Ok(add_statement_preview(
        &mm_data.database_path,
        &mm_data.database_header,
        locate_after,
        Statement::TheoremStatement(theorem),
    )?)
}

#[tauri::command]
pub async fn add_to_database(
    state: tauri::State<'_, Mutex<AppState>>,
    text: &str,
    override_proof_format: Option<ProofFormatOption>,
) -> Result<bool, Error> {
    let mut app_state = state.lock().await;
    let mut settings = app_state.settings.clone();
    let mm_data = app_state.metamath_data.as_mut().ok_or(Error::NoMmDbError)?;

    let stage_0 = mmp_parser::new(text);

    let MmpParserStage1::Success(stage_1_success) = stage_0.next_stage()? else {
        return Ok(false);
    };

    let MmpParserStage2::Success(stage_2_success) = stage_1_success.next_stage()? else {
        return Ok(false);
    };

    let MmpParserStage3::Success(stage_3_success) =
        stage_2_success.next_stage(&stage_1_success, mm_data)?
    else {
        return Ok(false);
    };

    let stage_3_theorem = match stage_3_success {
        MmpParserStage3Success::Empty => return Ok(false),
        MmpParserStage3Success::Header(stage_3_header) => {
            add_header(
                &mm_data.database_path,
                &mut mm_data.database_header,
                stage_3_header,
            )?;

            let (allowed_tags_and_attributes, allowed_css_properties) =
                html_validation::create_rule_structs();
            mm_data.calc_optimized_header_data(
                &allowed_tags_and_attributes,
                &allowed_css_properties,
            )?;

            return Ok(false);
        }
        MmpParserStage3Success::Comment(stage_3_comment) => {
            add_statement(
                &mm_data.database_path,
                &mut mm_data.database_header,
                stage_2_success.locate_after,
                Statement::CommentStatement(stage_3_comment.comment),
            )?;

            return Ok(true);
        }
        MmpParserStage3Success::Constants(constants) => {
            add_statement(
                &mm_data.database_path,
                &mut mm_data.database_header,
                stage_2_success.locate_after,
                Statement::ConstantStatement(constants),
            )?;

            return Ok(true);
        }
        MmpParserStage3Success::Variables(variables) => {
            add_statement(
                &mm_data.database_path,
                &mut mm_data.database_header,
                stage_2_success.locate_after,
                Statement::VariableStatement(variables),
            )?;

            return Ok(true);
        }
        MmpParserStage3Success::FloatingHypohesis(floating_hypothesis) => {
            add_statement(
                &mm_data.database_path,
                &mut mm_data.database_header,
                stage_2_success.locate_after,
                Statement::FloatingHypohesisStatement(floating_hypothesis),
            )?;

            return Ok(true);
        }
        MmpParserStage3Success::Theorem(s3t) => s3t,
    };

    let MmpParserStage4::Success(stage_4_success) =
        stage_3_theorem.next_stage(&stage_1_success, &stage_2_success, mm_data)?
    else {
        return Ok(false);
    };

    let stage_5 = stage_4_success.next_stage(&stage_2_success, &stage_3_theorem, mm_data)?;

    if let Some(pf) = override_proof_format {
        settings.proof_format = pf;
    }

    let stage_6 = stage_5.next_stage(&stage_4_success, mm_data, &settings)?;

    let locate_after = stage_2_success.locate_after;

    let Some(theorem) =
        mmp_parser_stages_to_theorem(stage_2_success, stage_3_theorem, stage_5, stage_6, mm_data)?
    else {
        return Ok(false);
    };

    let theorem_label = theorem.label.clone();

    add_statement(
        &mm_data.database_path,
        &mut mm_data.database_header,
        locate_after,
        Statement::TheoremStatement(theorem),
    )?;

    mm_data.update_optimized_theorem_data(&theorem_label, &settings)?;

    Ok(true)
}

fn mmp_parser_stages_to_theorem(
    stage_2_success: MmpParserStage2Success,
    stage_3_theorem: MmpParserStage3Theorem,
    stage_5: MmpParserStage5,
    stage_6: MmpParserStage6,
    mm_data: &MetamathData,
) -> Result<Option<Theorem>, Error> {
    let proof = if stage_3_theorem.is_axiom {
        None
    } else {
        let Some(proof) = stage_6.proof else {
            return Ok(None);
        };

        Some(proof)
    };

    Ok(Some(Theorem {
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
    }))
}

fn add_statement_preview(
    file_path: &str,
    header: &Header,
    locate_after: Option<LocateAfterRef>,
    statement: Statement,
) -> Result<(String, String), Error> {
    match locate_after {
        Some(loc_after) => {
            add_statement_locate_after_file(file_path, header, loc_after, &statement, false)?
                .ok_or(Error::InternalLogicError)
        }
        None => add_statement_at_end_file(file_path, &statement, false)?
            .ok_or(Error::InternalLogicError),
    }
}

fn add_statement(
    file_path: &str,
    header: &mut Header,
    locate_after: Option<LocateAfterRef>,
    statement: Statement,
) -> Result<(), Error> {
    match locate_after {
        Some(loc_after) => add_statement_locate_after(file_path, header, loc_after, statement),
        None => add_statement_at_end(file_path, header, statement),
    }
}

fn add_statement_locate_after(
    file_path: &str,
    header: &mut Header,
    locate_after: LocateAfterRef,
    statement: Statement,
) -> Result<(), Error> {
    add_statement_locate_after_file(file_path, header, locate_after, &statement, true)?;
    add_statement_locate_after_memory(header, locate_after, statement);
    Ok(())
}

fn add_statement_locate_after_memory(
    header: &mut Header,
    locate_after: LocateAfterRef,
    mut statement: Statement,
) -> Option<Statement> {
    for i in 0..header.content.len() {
        match locate_after {
            LocateAfterRef::LocateAfterConst(s) => {
                if let Some(Statement::ConstantStatement(constants)) = header.content.get(i) {
                    if constants.iter().find(|c| c.symbol == *s).is_some() {
                        header.content.insert(i + 1, statement);
                        return None;
                    }
                }
            }
            LocateAfterRef::LocateAfterVar(s) => {
                if let Some(Statement::VariableStatement(variables)) = header.content.get(i) {
                    if variables.iter().find(|c| c.symbol == *s).is_some() {
                        header.content.insert(i + 1, statement);
                        return None;
                    }
                }
            }
            LocateAfterRef::LocateAfter(s) => {
                if let Some(Statement::TheoremStatement(theorem)) = header.content.get(i) {
                    if theorem.label == *s {
                        header.content.insert(i + 1, statement);
                        return None;
                    }
                } else if let Some(Statement::FloatingHypohesisStatement(floating_hypothesis)) =
                    header.content.get(i)
                {
                    if floating_hypothesis.label == *s {
                        header.content.insert(i + 1, statement);
                        return None;
                    }
                }
            }
        }
    }

    for subheader in &mut header.subheaders {
        statement = match add_statement_locate_after_memory(subheader, locate_after, statement) {
            Some(s) => s,
            None => return None,
        };
    }

    Some(statement)
}

fn add_statement_locate_after_file(
    file_path: &str,
    header: &Header,
    locate_after: LocateAfterRef,
    statement: &Statement,
    write_to_file: bool,
) -> Result<Option<(String, String)>, Error> {
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

    let (mut file_content, next_token_i) = mm_parser.consume_early_and_return_file_content();

    let old_file_content = if write_to_file {
        None
    } else {
        Some(file_content.clone())
    };

    file_content.insert_str(next_token_i, "\n\n");
    statement.insert_mm_string(&mut file_content, next_token_i + 2);

    if write_to_file {
        fs::write(file_path, &file_content).or(Err(Error::FileWriteError))?;
        Ok(None)
    } else {
        Ok(Some((old_file_content.unwrap(), file_content)))
    }
}

fn add_statement_at_end(
    file_path: &str,
    header: &mut Header,
    statement: Statement,
) -> Result<(), Error> {
    add_statement_at_end_file(file_path, &statement, true)?;
    add_statement_at_end_memory(header, statement);
    Ok(())
}

fn add_statement_at_end_memory(header: &mut Header, statement: Statement) {
    let mut last_header = header;

    while last_header.subheaders.len() > 0 {
        last_header = last_header.subheaders.last_mut().unwrap();
    }

    last_header.content.push(statement);
}

fn add_statement_at_end_file(
    file_path: &str,
    statement: &Statement,
    write_to_file: bool,
) -> Result<Option<(String, String)>, Error> {
    let mut file_content = fs::read_to_string(file_path).or(Err(Error::FileReadError))?;

    let old_file_content = if write_to_file {
        None
    } else {
        Some(file_content.clone())
    };

    loop {
        match file_content.pop() {
            Some(c) if c.is_whitespace() => {}
            Some(c) => {
                file_content.push(c);
                break;
            }
            None => break,
        }
    }

    if !file_content.is_empty() {
        file_content.push_str("\n\n");
    }

    statement.write_mm_string(&mut file_content);
    file_content.push_str("\n");

    if write_to_file {
        fs::write(file_path, file_content).or(Err(Error::FileWriteError))?;
        Ok(None)
    } else {
        Ok(Some((old_file_content.unwrap(), file_content)))
    }
}

fn add_header_preview(
    file_path: &str,
    header: &Header,
    stage_3_header: MmpParserStage3Header,
) -> Result<(String, String), Error> {
    Ok(add_header_file(file_path, header, &stage_3_header, false)?
        .ok_or(Error::InternalLogicError)?)
}

fn add_header(
    file_path: &str,
    header: &mut Header,
    stage_3_header: MmpParserStage3Header,
) -> Result<(), Error> {
    add_header_file(file_path, header, &stage_3_header, true)?;
    add_header_memory(header, stage_3_header)?;

    Ok(())
}

fn add_header_file(
    file_path: &str,
    header: &Header,
    stage_3_header: &MmpParserStage3Header,
    write_to_file: bool,
) -> Result<Option<(String, String)>, Error> {
    let mut mm_parser = MmParser::new(file_path, None, None)?;

    let mut current_header_path = HeaderPath { path: Vec::new() };
    let mut target_header_path = stage_3_header.parent_header_path.clone();
    target_header_path.path.push(stage_3_header.header_i);

    for database_element in header.iter() {
        match database_element {
            DatabaseElement::Header(_, depth) => {
                util::calc_next_header_path(&mut current_header_path, depth)?;

                if current_header_path < target_header_path {
                    loop {
                        if let Some(StatementProcessed::HeaderStatement) =
                            mm_parser.process_next_statement()?
                        {
                            break;
                        }
                    }
                } else {
                    break;
                }
            }
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

    let (mut file_content, next_token_i) = mm_parser.consume_early_and_return_file_content();

    let old_file_content = if write_to_file {
        None
    } else {
        Some(file_content.clone())
    };

    file_content.insert_str(next_token_i, "\n\n");
    Header::insert_mm_string(
        target_header_path.path.len() as u32,
        &stage_3_header.title,
        &stage_3_header.description,
        &mut file_content,
        next_token_i + 2,
    )?;

    if write_to_file {
        fs::write(file_path, &file_content).or(Err(Error::FileWriteError))?;
        Ok(None)
    } else {
        Ok(Some((old_file_content.unwrap(), file_content)))
    }
}

fn add_header_memory(
    database_header: &mut Header,
    stage_3_header: MmpParserStage3Header,
) -> Result<(), Error> {
    let parent_header = stage_3_header
        .parent_header_path
        .resolve_mut(database_header)
        .ok_or(Error::InternalLogicError)?;

    if stage_3_header.header_i <= parent_header.subheaders.len() {
        parent_header.subheaders.insert(
            stage_3_header.header_i,
            Header {
                title: stage_3_header.title,
                description: stage_3_header.description,
                content: Vec::new(),
                subheaders: Vec::new(),
            },
        );
    } else if stage_3_header.header_i > parent_header.subheaders.len() {
        return Err(Error::InvalidHeaderPathError);
    }

    Ok(())
}
