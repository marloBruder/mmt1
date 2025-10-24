use std::{
    fs::{self, File},
    io::{BufReader, Read},
};

use serde::Serialize;
use sha2::{Digest, Sha256};
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
    model::{
        DatabaseElement, Header, HeaderContentRepresentation, HeaderPath, Hypothesis, MetamathData,
        Statement, Theorem,
    },
    util::{self, description_parser, StrIterToSpaceSeperatedString},
    AppState, Error, ProofFormatOption,
};

#[derive(Serialize)]
pub struct AddToDatabasePreviewData {
    #[serde(rename = "oldFileContent")]
    old_file_content: String,
    #[serde(rename = "newFileContent")]
    new_file_content: String,
    #[serde(rename = "invalidHtml")]
    invalid_html: bool,
}

#[tauri::command]
pub async fn add_to_database_preview(
    state: tauri::State<'_, Mutex<AppState>>,
    text: &str,
    override_proof_format: Option<ProofFormatOption>,
) -> Result<AddToDatabasePreviewData, Error> {
    let app_state = state.lock().await;
    let mm_data = app_state.metamath_data.as_ref().ok_or(Error::NoMmDbError)?;
    let settings = &app_state.settings;

    if database_has_changed(&mm_data.database_path, &mm_data.database_hash)? {
        return Err(Error::DatabaseHasChangedError);
    }

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
            let (allowed_tags_and_attributes, allowed_css_properties) =
                html_validation::create_rule_structs();
            let invalid_html = !description_parser::parse_description(
                &stage_3_header.description,
                &mm_data.database_header,
                &allowed_tags_and_attributes,
                &allowed_css_properties,
            )
            .1
            .is_empty();

            let (old_file_content, new_file_content) = add_header_preview(
                &mm_data.database_path,
                &mm_data.database_header,
                stage_3_header,
            )?;

            return Ok(AddToDatabasePreviewData {
                old_file_content,
                new_file_content,
                invalid_html,
            });
        }
        MmpParserStage3Success::Comment(stage_3_comment) => {
            let (old_file_content, new_file_content) = add_statement_preview(
                &mm_data.database_path,
                &mm_data.database_header,
                stage_2_success.locate_after,
                Statement::CommentStatement(stage_3_comment.comment),
            )?;

            return Ok(AddToDatabasePreviewData {
                old_file_content,
                new_file_content,
                invalid_html: false,
            });
        }
        MmpParserStage3Success::Constants(constants) => {
            let (old_file_content, new_file_content) = add_statement_preview(
                &mm_data.database_path,
                &mm_data.database_header,
                stage_2_success.locate_after,
                Statement::ConstantStatement(constants),
            )?;

            return Ok(AddToDatabasePreviewData {
                old_file_content,
                new_file_content,
                invalid_html: false,
            });
        }
        MmpParserStage3Success::Variables(variables) => {
            let (old_file_content, new_file_content) = add_statement_preview(
                &mm_data.database_path,
                &mm_data.database_header,
                stage_2_success.locate_after,
                Statement::VariableStatement(variables),
            )?;

            return Ok(AddToDatabasePreviewData {
                old_file_content,
                new_file_content,
                invalid_html: false,
            });
        }
        MmpParserStage3Success::FloatingHypohesis(floating_hypothesis) => {
            let (old_file_content, new_file_content) = add_statement_preview(
                &mm_data.database_path,
                &mm_data.database_header,
                stage_2_success.locate_after,
                Statement::FloatingHypohesisStatement(floating_hypothesis),
            )?;

            return Ok(AddToDatabasePreviewData {
                old_file_content,
                new_file_content,
                invalid_html: false,
            });
        }
        MmpParserStage3Success::Theorem(s3t) => s3t,
    };

    let MmpParserStage4::Success(stage_4_success) =
        stage_3_theorem.next_stage(&stage_1_success, &stage_2_success, mm_data)?
    else {
        return Err(Error::CantAddToDatabaseError);
    };

    let stage_5 = stage_4_success.next_stage(&stage_2_success, &stage_3_theorem, mm_data, None)?;

    let mut new_settings = settings.clone();
    if let Some(pf) = override_proof_format {
        new_settings.proof_format = pf;
    }

    let stage_6 = stage_5.next_stage(&stage_3_theorem, &stage_4_success, mm_data, &new_settings)?;

    let locate_after = stage_2_success.locate_after;

    let Some(theorem) =
        mmp_parser_stages_to_theorem(stage_2_success, stage_3_theorem, stage_5, stage_6, mm_data)?
    else {
        return Err(Error::CantAddToDatabaseError);
    };

    let (allowed_tags_and_attributes, allowed_css_properties) =
        html_validation::create_rule_structs();
    let invalid_html = !description_parser::parse_description(
        &theorem.description,
        &mm_data.database_header,
        &allowed_tags_and_attributes,
        &allowed_css_properties,
    )
    .1
    .is_empty();

    let (old_file_content, new_file_content) = add_statement_preview(
        &mm_data.database_path,
        &mm_data.database_header,
        locate_after,
        Statement::TheoremStatement(theorem),
    )?;

    return Ok(AddToDatabasePreviewData {
        old_file_content,
        new_file_content,
        invalid_html,
    });
}

pub fn database_has_changed(file_path: &str, database_hash: &str) -> Result<bool, Error> {
    let file = File::open(file_path).map_err(|_| Error::FileReadError)?;
    let mut reader = BufReader::new(file);

    let mut hasher = Sha256::new();
    let mut buffer = [0; 1024];

    loop {
        let bytes_read = reader.read(&mut buffer).map_err(|_| Error::FileReadError)?;

        if bytes_read == 0 {
            break;
        }

        hasher.update(&buffer[..bytes_read]);
    }

    let hasher_result = hasher.finalize();

    let new_database_hash: String = hasher_result
        .into_iter()
        .map(|byte| format!("{:02x}", byte))
        .collect();

    Ok(new_database_hash != database_hash)
}

pub enum AddToDatabaseResult {
    NewHeader {
        header_title: String,
        header_path: HeaderPath,
    },
    NewStatement {
        content_rep: HeaderContentRepresentation,
        header_path: HeaderPath,
        header_content_i: usize,
    },
}

#[tauri::command]
pub async fn add_to_database(
    state: tauri::State<'_, Mutex<AppState>>,
    text: &str,
    override_proof_format: Option<ProofFormatOption>,
) -> Result<Option<(AddToDatabaseResult, bool)>, Error> {
    let mut app_state = state.lock().await;
    let mut settings = app_state.settings.clone();
    let mm_data = app_state.metamath_data.as_mut().ok_or(Error::NoMmDbError)?;

    let stage_0 = mmp_parser::new(text);

    let MmpParserStage1::Success(stage_1_success) = stage_0.next_stage()? else {
        return Ok(None);
    };

    let MmpParserStage2::Success(stage_2_success) = stage_1_success.next_stage()? else {
        return Ok(None);
    };

    let MmpParserStage3::Success(stage_3_success) =
        stage_2_success.next_stage(&stage_1_success, mm_data)?
    else {
        return Ok(None);
    };

    let stage_3_theorem = match stage_3_success {
        MmpParserStage3Success::Empty => return Ok(None),
        MmpParserStage3Success::Header(stage_3_header) => {
            let header_title = stage_3_header.title.clone();

            let mut header_path = stage_3_header.parent_header_path.clone();
            header_path.path.push(stage_3_header.header_i);

            let new_database_hash = add_header(
                &mm_data.database_path,
                &mut mm_data.database_header,
                stage_3_header,
            )?;

            mm_data.database_hash = new_database_hash;

            let (allowed_tags_and_attributes, allowed_css_properties) =
                html_validation::create_rule_structs();
            mm_data.calc_optimized_header_data(
                &allowed_tags_and_attributes,
                &allowed_css_properties,
            )?;

            return Ok(Some((
                AddToDatabaseResult::NewHeader {
                    header_title,
                    header_path,
                },
                false,
            )));
        }
        MmpParserStage3Success::Comment(stage_3_comment) => {
            let statement = Statement::CommentStatement(stage_3_comment.comment);

            let content_rep = statement.to_header_content_representation();

            let (header_path, header_content_i, new_database_hash) = add_statement(
                &mm_data.database_path,
                &mut mm_data.database_header,
                stage_2_success.locate_after,
                statement,
            )?;

            mm_data.database_hash = new_database_hash;

            return Ok(Some((
                AddToDatabaseResult::NewStatement {
                    content_rep,
                    header_path,
                    header_content_i,
                },
                false,
            )));
        }
        MmpParserStage3Success::Constants(constants) => {
            let statement = Statement::ConstantStatement(constants);

            let content_rep = statement.to_header_content_representation();

            let (header_path, header_content_i, new_database_hash) = add_statement(
                &mm_data.database_path,
                &mut mm_data.database_header,
                stage_2_success.locate_after,
                statement,
            )?;

            mm_data.database_hash = new_database_hash;

            mm_data.grammar_calculations_done = false;

            return Ok(Some((
                AddToDatabaseResult::NewStatement {
                    content_rep,
                    header_path,
                    header_content_i,
                },
                true,
            )));
        }
        MmpParserStage3Success::Variables(variables) => {
            for var in &variables {
                mm_data.optimized_data.variables.insert(var.symbol.clone());
            }

            let statement = Statement::VariableStatement(variables);

            let content_rep = statement.to_header_content_representation();

            let (header_path, header_content_i, new_database_hash) = add_statement(
                &mm_data.database_path,
                &mut mm_data.database_header,
                stage_2_success.locate_after,
                statement,
            )?;

            mm_data.database_hash = new_database_hash;

            mm_data.grammar_calculations_done = false;

            return Ok(Some((
                AddToDatabaseResult::NewStatement {
                    content_rep,
                    header_path,
                    header_content_i,
                },
                true,
            )));
        }
        MmpParserStage3Success::FloatingHypohesis(floating_hypothesis) => {
            let statement = Statement::FloatingHypohesisStatement(floating_hypothesis);

            let content_rep = statement.to_header_content_representation();

            let (header_path, header_content_i, new_database_hash) = add_statement(
                &mm_data.database_path,
                &mut mm_data.database_header,
                stage_2_success.locate_after,
                statement,
            )?;

            mm_data.database_hash = new_database_hash;

            mm_data.recalc_optimized_floating_hypotheses_after_one_new()?;

            mm_data.grammar_calculations_done = false;

            return Ok(Some((
                AddToDatabaseResult::NewStatement {
                    content_rep,
                    header_path,
                    header_content_i,
                },
                true,
            )));
        }
        MmpParserStage3Success::Theorem(s3t) => s3t,
    };

    let MmpParserStage4::Success(stage_4_success) =
        stage_3_theorem.next_stage(&stage_1_success, &stage_2_success, mm_data)?
    else {
        return Ok(None);
    };

    let stage_5 = stage_4_success.next_stage(&stage_2_success, &stage_3_theorem, mm_data, None)?;

    if let Some(pf) = override_proof_format {
        settings.proof_format = pf;
    }

    let stage_6 = stage_5.next_stage(&stage_3_theorem, &stage_4_success, mm_data, &settings)?;

    let locate_after = stage_2_success.locate_after;

    let Some(theorem) =
        mmp_parser_stages_to_theorem(stage_2_success, stage_3_theorem, stage_5, stage_6, mm_data)?
    else {
        return Ok(None);
    };

    let theorem_label = theorem.label.clone();

    let statement = Statement::TheoremStatement(theorem);

    let content_rep = statement.to_header_content_representation();

    let (header_path, header_content_i, new_database_hash) = add_statement(
        &mm_data.database_path,
        &mut mm_data.database_header,
        locate_after,
        statement,
    )?;

    mm_data.database_hash = new_database_hash;

    let is_syntax_axiom = mm_data.update_optimized_theorem_data(&theorem_label, &settings)?;

    if is_syntax_axiom {
        mm_data.grammar_calculations_done = false;
    }

    Ok(Some((
        AddToDatabaseResult::NewStatement {
            content_rep,
            header_path,
            header_content_i,
        },
        is_syntax_axiom,
    )))
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
        assertion: if stage_3_theorem.is_axiom {
            stage_2_success
                .proof_lines
                .iter()
                .find(|pl| pl.step_name == "qed")
                .ok_or(Error::InternalLogicError)?
                .expression
                .split_ascii_whitespace()
                .fold_to_space_seperated_string()
        } else {
            stage_5
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
                )?
        },
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
            add_statement_locate_after_file(file_path, header, loc_after, &statement, false)
        }
        None => add_statement_at_end_file(file_path, &statement, false),
    }
}

fn add_statement(
    file_path: &str,
    header: &mut Header,
    locate_after: Option<LocateAfterRef>,
    statement: Statement,
) -> Result<(HeaderPath, usize, String), Error> {
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
) -> Result<(HeaderPath, usize, String), Error> {
    let (new_database_hash, _) =
        add_statement_locate_after_file(file_path, header, locate_after, &statement, true)?;
    let (header_path, header_content_i) =
        add_statement_locate_after_memory(header, locate_after, statement, &mut HeaderPath::new())
            .map_err(|_| Error::InternalLogicError)?;

    Ok((header_path, header_content_i, new_database_hash))
}

fn add_statement_locate_after_memory(
    header: &mut Header,
    locate_after: LocateAfterRef,
    mut statement: Statement,
    header_path: &mut HeaderPath,
) -> Result<(HeaderPath, usize), Statement> {
    match locate_after {
        LocateAfterRef::LocateAfterStart => {
            header.content.insert(0, statement);
            return Ok((header_path.clone(), 0));
        }
        LocateAfterRef::LocateAfterHeader(header_path_str) => {
            if header_path_str == header_path.to_string() {
                header.content.insert(0, statement);
                return Ok((header_path.clone(), 0));
            }
        }
        LocateAfterRef::LocateAfterComment(comment_path_str) => {
            let header_path_string = header_path.to_string();

            let mut comment_i = 0;

            for i in 0..header.content.len() {
                if let Some(Statement::CommentStatement(_)) = header.content.get(i) {
                    comment_i += 1;

                    if comment_path_str == format!("{}#{}", header_path_string, comment_i) {
                        header.content.insert(i + 1, statement);
                        return Ok((header_path.clone(), i + 1));
                    }
                }
            }
        }
        LocateAfterRef::LocateAfterConst(const_str) => {
            for i in 0..header.content.len() {
                if let Some(Statement::ConstantStatement(constants)) = header.content.get(i) {
                    if constants.iter().find(|c| c.symbol == *const_str).is_some() {
                        header.content.insert(i + 1, statement);
                        return Ok((header_path.clone(), i + 1));
                    }
                }
            }
        }
        LocateAfterRef::LocateAfterVar(var_str) => {
            for i in 0..header.content.len() {
                if let Some(Statement::VariableStatement(variables)) = header.content.get(i) {
                    if variables.iter().find(|c| c.symbol == *var_str).is_some() {
                        header.content.insert(i + 1, statement);
                        return Ok((header_path.clone(), i + 1));
                    }
                }
            }
        }
        LocateAfterRef::LocateAfter(label_str) => {
            for i in 0..header.content.len() {
                if let Some(Statement::TheoremStatement(theorem)) = header.content.get(i) {
                    if theorem.label == *label_str {
                        header.content.insert(i + 1, statement);
                        return Ok((header_path.clone(), i + 1));
                    }
                } else if let Some(Statement::FloatingHypohesisStatement(floating_hypothesis)) =
                    header.content.get(i)
                {
                    if floating_hypothesis.label == *label_str {
                        header.content.insert(i + 1, statement);
                        return Ok((header_path.clone(), i + 1));
                    }
                }
            }
        }
    }

    for (i, subheader) in header.subheaders.iter_mut().enumerate() {
        header_path.path.push(i);
        statement = match add_statement_locate_after_memory(
            subheader,
            locate_after,
            statement,
            header_path,
        ) {
            Ok(hp) => return Ok(hp),
            Err(s) => s,
        };
        header_path.path.pop();
    }

    Err(statement)
}

fn add_statement_locate_after_file(
    file_path: &str,
    header: &Header,
    locate_after: LocateAfterRef,
    statement: &Statement,
    write_to_file: bool,
) -> Result<(String, String), Error> {
    let mut mm_parser = MmParser::new(file_path, None, None)?;

    for database_element in header.locate_after_iter(Some(locate_after)) {
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
        let new_database_hash = util::str_to_hash_string(&file_content);

        fs::write(file_path, &file_content).or(Err(Error::FileWriteError))?;
        Ok((new_database_hash, String::new()))
    } else {
        Ok((old_file_content.unwrap(), file_content))
    }
}

fn add_statement_at_end(
    file_path: &str,
    header: &mut Header,
    statement: Statement,
) -> Result<(HeaderPath, usize, String), Error> {
    let (new_database_hash, _) = add_statement_at_end_file(file_path, &statement, true)?;
    let (header_path, header_content_i) = add_statement_at_end_memory(header, statement);
    Ok((header_path, header_content_i, new_database_hash))
}

fn add_statement_at_end_memory(header: &mut Header, statement: Statement) -> (HeaderPath, usize) {
    let mut last_header = header;
    let mut header_path = HeaderPath::new();

    while last_header.subheaders.len() > 0 {
        header_path.path.push(last_header.subheaders.len() - 1);
        last_header = last_header.subheaders.last_mut().unwrap();
    }

    let header_content_i = last_header.content.len();
    last_header.content.push(statement);

    (header_path, header_content_i)
}

fn add_statement_at_end_file(
    file_path: &str,
    statement: &Statement,
    write_to_file: bool,
) -> Result<(String, String), Error> {
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
        let new_database_hash = util::str_to_hash_string(&file_content);

        fs::write(file_path, file_content).or(Err(Error::FileWriteError))?;

        Ok((new_database_hash, String::new()))
    } else {
        Ok((old_file_content.unwrap(), file_content))
    }
}

fn add_header_preview(
    file_path: &str,
    header: &Header,
    stage_3_header: MmpParserStage3Header,
) -> Result<(String, String), Error> {
    add_header_file(file_path, header, &stage_3_header, false)
}

fn add_header(
    file_path: &str,
    header: &mut Header,
    stage_3_header: MmpParserStage3Header,
) -> Result<String, Error> {
    let (new_database_hash, _) = add_header_file(file_path, header, &stage_3_header, true)?;
    add_header_memory(header, stage_3_header)?;

    Ok(new_database_hash)
}

fn add_header_file(
    file_path: &str,
    header: &Header,
    stage_3_header: &MmpParserStage3Header,
    write_to_file: bool,
) -> Result<(String, String), Error> {
    let mut mm_parser = MmParser::new(file_path, None, None)?;

    let mut current_header_path = HeaderPath::new();
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
        let new_database_hash = util::str_to_hash_string(&file_content);

        fs::write(file_path, &file_content).or(Err(Error::FileWriteError))?;
        Ok((new_database_hash, String::new()))
    } else {
        Ok((old_file_content.unwrap(), file_content))
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
        return Err(Error::InternalLogicError);
    }

    Ok(())
}

impl serde::Serialize for AddToDatabaseResult {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        use serde::ser::SerializeStruct;

        match self {
            Self::NewHeader {
                header_title,
                header_path,
            } => {
                let mut state = serializer.serialize_struct("NewHeader", 2)?;
                state.serialize_field("headerTitle", header_title)?;
                state.serialize_field("headerPath", header_path)?;
                state.serialize_field("discriminator", "NewHeader")?;
                state.end()
            }
            Self::NewStatement {
                content_rep,
                header_path,
                header_content_i,
            } => {
                let mut state = serializer.serialize_struct("NewStatement", 3)?;
                state.serialize_field("contentRep", content_rep)?;
                state.serialize_field("headerPath", header_path)?;
                state.serialize_field("headerContentI", header_content_i)?;
                state.serialize_field("discriminator", "NewStatement")?;
                state.end()
            }
        }
    }
}
