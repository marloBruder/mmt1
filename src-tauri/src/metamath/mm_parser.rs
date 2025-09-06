use std::{
    collections::{HashMap, HashSet},
    fs,
    sync::Arc,
};

use tauri::{async_runtime::Mutex, AppHandle, Emitter};

use crate::{
    model::{
        ColorInformation, Comment, Constant, FloatingHypothesis, Header, HeaderPath,
        HeaderRepresentation, HtmlRepresentation, Hypothesis, MetamathData, OptimizedMetamathData,
        Statement, SymbolNumberMapping, Theorem, TheoremParseTrees, Variable, VariableColor,
    },
    util::{self, earley_parser_optimized::Grammar},
    AppState, Error,
};

pub mod html_validation;

#[tauri::command]
pub async fn open_metamath_database(
    state: tauri::State<'_, Mutex<AppState>>,
    app: AppHandle,
    mm_file_path: &str,
) -> Result<(u32, Vec<HtmlRepresentation>, Vec<(String, String)>), Error> {
    // let metamath_data = MmParser::process_database(mm_file_path)?;

    let mut app_state = state.lock().await;
    let database_id = app_state.id_manager.get_next_id();
    app_state.stop_temp_database_calculations = Arc::new(std::sync::Mutex::new(false));
    let stop = app_state.stop_temp_database_calculations.clone();
    drop(app_state);

    let mut mm_parser = MmParser::new(mm_file_path, Some(app), Some(stop))?;
    mm_parser.process_all_statements()?;
    let (metamath_data, invalid_html, invalid_description_html) =
        mm_parser.consume_early_before_grammar_calculations(database_id)?;

    let mut app_state = state.lock().await;
    app_state.temp_metamath_data = Some(metamath_data);

    Ok((database_id, invalid_html, invalid_description_html))
}

#[tauri::command]
pub async fn cancel_open_metamath_database(
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<(), Error> {
    let mut app_state = state.lock().await;

    *app_state
        .stop_temp_database_calculations
        .lock()
        .or(Err(Error::InternalLogicError))? = true;
    app_state.temp_metamath_data = None;

    Ok(())
}

#[tauri::command]
pub async fn confirm_open_metamath_database(
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<
    (
        u32,
        u32,
        HeaderRepresentation,
        Vec<HtmlRepresentation>,
        Vec<ColorInformation>,
    ),
    Error,
> {
    let mut app_state = state.lock().await;

    let metamath_data = app_state
        .temp_metamath_data
        .take()
        .ok_or(Error::InternalLogicError)?;

    let database_id = metamath_data.database_id;

    let theorem_amount = metamath_data.optimized_data.theorem_amount;

    let header_rep = metamath_data.database_header.to_representation();

    let html_reps = metamath_data.html_representations.clone();

    let color_information = metamath_data.calc_color_information(true);

    app_state.metamath_data = Some(metamath_data);

    *app_state
        .stop_database_calculations
        .lock()
        .or(Err(Error::InternalLogicError))? = true;
    app_state.stop_database_calculations = app_state.stop_temp_database_calculations.clone();

    Ok((
        database_id,
        theorem_amount,
        header_rep,
        html_reps,
        color_information,
    ))
}

#[tauri::command]
pub async fn perform_grammar_calculations(
    state: tauri::State<'_, Mutex<AppState>>,
    database_id: u32,
    app: AppHandle,
) -> Result<(), Error> {
    let app_state = state.lock().await;
    let mm_data = if app_state
        .temp_metamath_data
        .as_ref()
        .is_some_and(|mm| mm.database_id == database_id)
    {
        app_state
            .temp_metamath_data
            .as_ref()
            .ok_or(Error::InternalLogicError)?
    } else if app_state
        .metamath_data
        .as_ref()
        .is_some_and(|mm| mm.database_id == database_id)
    {
        app_state
            .metamath_data
            .as_ref()
            .ok_or(Error::InternalLogicError)?
    } else {
        return Ok(());
    };

    let stop = app_state.stop_temp_database_calculations.clone();
    let theorem_amount = mm_data.optimized_data.theorem_amount;
    let database_header = mm_data.database_header.clone();
    let floating_hypotheses = mm_data.optimized_data.floating_hypotheses.clone();

    drop(app_state);

    let symbol_number_mapping = SymbolNumberMapping::calc_mapping(&database_header);

    let Some((grammar, parse_trees)) = Grammar::calc_grammar_and_parse_trees(
        &database_header,
        &symbol_number_mapping,
        &floating_hypotheses,
        theorem_amount,
        database_id,
        Some(app),
        Some(stop),
    )?
    else {
        return Ok(());
    };

    let mut app_state = state.lock().await;
    let mm_data = if app_state
        .temp_metamath_data
        .as_ref()
        .is_some_and(|mm| mm.database_id == database_id)
    {
        app_state
            .temp_metamath_data
            .as_mut()
            .ok_or(Error::InternalLogicError)?
    } else if app_state
        .metamath_data
        .as_ref()
        .is_some_and(|mm| mm.database_id == database_id)
    {
        app_state
            .metamath_data
            .as_mut()
            .ok_or(Error::InternalLogicError)?
    } else {
        return Ok(());
    };

    mm_data.grammar_calculations_done = true;
    mm_data.optimized_data.symbol_number_mapping = symbol_number_mapping;
    mm_data.optimized_data.grammar = grammar;
    for (label, assertion_parsed, hypotheses_parsed) in parse_trees {
        mm_data
            .optimized_data
            .theorem_data
            .get_mut(label)
            .ok_or(Error::InternalLogicError)?
            .parse_trees = Some(TheoremParseTrees {
            assertion_parsed,
            hypotheses_parsed,
        })
    }

    Ok(())
}

pub struct MmParser {
    file_content: String,
    next_token_i: usize,
    database_path: String,
    database_header: Header,
    html_representations: Vec<HtmlRepresentation>,
    variable_colors: Vec<VariableColor>,
    alt_variable_colors: Vec<VariableColor>,
    curr_header_path: HeaderPath,
    scope: usize,
    active_consts: HashSet<String>,
    active_vars: Vec<HashSet<String>>,
    active_float_hyps: Vec<Vec<FloatingHypothesis>>,
    active_dists: Vec<Vec<String>>,
    active_hyps: Vec<Vec<Hypothesis>>,
    prev_variables: HashSet<String>,
    prev_float_hyps: Vec<FloatingHypothesis>,
    next_label: Option<String>,
    next_description: Option<String>,
    theorem_amount: u32,
    curr_line_amount: u32,
    total_line_amount: u32,
    last_progress_reported: u8,
    app: Option<AppHandle>,
    // Only used when using the process_all_statements or consume_early_before_grammar_calculations functions
    stop: Option<Arc<std::sync::Mutex<bool>>>,
    invalid_html: Vec<HtmlRepresentation>,
    html_allowed_tags_and_attributes: HashMap<String, HashSet<String>>,
    css_allowed_properties: HashSet<String>,
}

pub enum StatementProcessed {
    OpeningScopeStatement,
    ClosingScopeStatement,
    CommentStatement,
    HeaderStatement,
    ConstantStatement,
    VariableStatement,
    FloatingHypothesisStatement,
    EssentialHypothesisStatement,
    DistinctVariableStatement,
    TheoremStatement, // Includes Axiom Statements
}

impl MmParser {
    pub fn new(
        file_path: &str,
        app: Option<AppHandle>,
        stop: Option<Arc<std::sync::Mutex<bool>>>,
    ) -> Result<MmParser, Error> {
        let file_content = fs::read_to_string(file_path).or(Err(Error::FileReadError))?;

        if !file_content.is_ascii() {
            return Err(Error::InvalidCharactersError);
        }

        let total_line_amount = file_content.lines().count() as u32;

        let (html_allowed_tags_and_attributes, css_allowed_properties) =
            html_validation::create_rule_structs();

        Ok(MmParser {
            file_content,
            next_token_i: 0,
            database_header: Header::default(),
            database_path: file_path.to_string(),
            html_representations: Vec::new(),
            variable_colors: Vec::new(),
            alt_variable_colors: Vec::new(),
            curr_header_path: HeaderPath::default(),
            scope: 0,
            active_consts: HashSet::new(),
            active_vars: vec![HashSet::new()],
            active_float_hyps: vec![Vec::new()],
            active_dists: vec![Vec::new()],
            active_hyps: vec![Vec::new()],
            prev_variables: HashSet::new(),
            prev_float_hyps: Vec::new(),
            next_label: None,
            next_description: None,
            theorem_amount: 0,
            curr_line_amount: 0,
            total_line_amount,
            last_progress_reported: 0,
            app,
            stop,
            invalid_html: Vec::new(),
            html_allowed_tags_and_attributes,
            css_allowed_properties,
        })
    }

    // pub fn process_database(file_path: &str) -> Result<MetamathData, Error> {
    //     let mm_parser = MmParser::new(file_path)?;

    //     Ok(mm_parser.process_all_statements_and_consume()?)
    // }

    pub fn process_next_statement(&mut self) -> Result<Option<StatementProcessed>, Error> {
        let mut comment_processed = false;

        if let Some(token) = self.advance_next_token() {
            let statement_processed = match token {
                "$(" => {
                    comment_processed = true;
                    self.process_comment_statement()?
                }
                "${" => self.process_opening_scope_statement(),
                "$}" => self.process_closing_scope_statement()?,
                "$c" => self.process_constant_statement()?,
                "$v" => self.process_variable_statement()?,
                "$f" => self.process_floating_hypothesis_statement()?,
                "$e" => self.process_essential_hypothesis_statement()?,
                "$d" => self.process_distinct_variable_statement()?,
                keyword @ ("$a" | "$p") => {
                    let is_axiom = keyword == "$a";
                    self.process_theorem_statement(is_axiom)?
                }
                label => {
                    let label_string = label.to_string();
                    if self.next_label.is_none() {
                        self.next_label = Some(label_string);

                        let statement_processed = self.process_next_statement()?;

                        // next_label was not used by process_next_statement, therefore it should not have been there
                        if self.next_label.is_some() {
                            return Err(Error::TokenOutsideStatementError);
                        }

                        match statement_processed {
                            Some(sp) => sp,
                            // there was no follow up token, therefore label did not belong to any valid statement
                            None => return Err(Error::TokenOutsideStatementError),
                        }
                    } else {
                        // there was another token before label, this should not happen
                        return Err(Error::TokenOutsideStatementError);
                    }
                }
            };

            if !comment_processed {
                self.next_description = None;
            }

            if let Some(ref app_handle) = self.app {
                // Should always produce value between 0 and 100
                let curr_progress = ((self.curr_line_amount * 100) / self.total_line_amount) as u8;
                if self.last_progress_reported < curr_progress {
                    self.last_progress_reported = curr_progress;
                    app_handle.emit("mm-parser-progress", curr_progress).ok();
                }
            }

            Ok(Some(statement_processed))
        } else {
            Ok(None)
        }
    }

    pub fn process_all_statements(&mut self) -> Result<(), Error> {
        let mut statements_processed = 0;
        while let Some(_) = self.process_next_statement()? {
            statements_processed += 1;
            if statements_processed % 10_000 == 0 {
                println!("Statements processed: {}", statements_processed);
            }

            if let Some(ref stop_arc) = self.stop {
                let stop_bool = stop_arc.lock().or(Err(Error::InternalLogicError))?;
                if *stop_bool {
                    return Err(Error::OpenDatabaseStoppedEarlyError);
                }
            }
        }

        Ok(())
    }

    // fn consume(self) -> Result<MetamathData, Error> {
    //     let mut metamath_data = MetamathData {
    //         database_path: self.database_path,
    //         database_header: self.database_header,
    //         html_representations: self.html_representations,
    //         optimized_data: OptimizedMetamathData {
    //             floating_hypotheses: self
    //                 .active_float_hyps
    //                 .into_iter()
    //                 .next()
    //                 .ok_or(Error::InternalLogicError)?,
    //             theorem_amount: self.theorem_amount,
    //             ..Default::default()
    //         },
    //         variable_colors: self.variable_colors,
    //         alt_variable_colors: self.alt_variable_colors,
    //     };

    //     metamath_data.recalc_symbol_number_mapping_and_grammar()?;

    //     Ok(metamath_data)
    // }

    pub fn consume_early_and_return_file_content(self) -> (String, usize) {
        if let Some(ref app_handle) = self.app {
            app_handle.emit("mm-parser-progress", 100).ok();
        }

        (self.file_content, self.next_token_i)
    }

    fn consume_early_before_grammar_calculations(
        self,
        database_id: u32,
    ) -> Result<(MetamathData, Vec<HtmlRepresentation>, Vec<(String, String)>), Error> {
        if let Some(ref app_handle) = self.app {
            app_handle.emit("mm-parser-progress", 100).ok();
        }

        let mut metamath_data = MetamathData {
            database_id,
            database_path: self.database_path,
            database_header: self.database_header,
            html_representations: self.html_representations,
            optimized_data: OptimizedMetamathData {
                floating_hypotheses: self
                    .active_float_hyps
                    .into_iter()
                    .next()
                    .ok_or(Error::InternalLogicError)?,
                theorem_amount: self.theorem_amount,
                theorem_data: HashMap::new(),
                symbol_number_mapping: SymbolNumberMapping::default(),
                grammar: Grammar::default(),
            },
            grammar_calculations_done: false,
            variable_colors: self.variable_colors,
            alt_variable_colors: self.alt_variable_colors,
        };

        let invalid_description_html = metamath_data.calc_optimized_theorem_data(
            self.app.as_ref(),
            &self.html_allowed_tags_and_attributes,
            &self.css_allowed_properties,
            self.stop,
        )?;

        if let Some(ref app_handle) = self.app {
            app_handle
                .emit("calc-optimized-theorem-data-progress", 100)
                .ok();
        }

        Ok((metamath_data, self.invalid_html, invalid_description_html))
    }

    // pub fn process_all_statements_and_consume(mut self) -> Result<MetamathData, Error> {
    //     self.process_all_statements()?;

    //     self.consume()
    // }

    fn advance_next_token(&mut self) -> Option<&str> {
        let string_bytes = self.file_content.as_bytes();

        while string_bytes
            .get(self.next_token_i)
            .is_some_and(|c| c.is_ascii_whitespace())
        {
            if string_bytes
                .get(self.next_token_i)
                .is_some_and(|c| *c == b'\n')
            {
                self.curr_line_amount += 1;
            }
            self.next_token_i += 1;
        }

        let start_token_i = self.next_token_i;

        while string_bytes
            .get(self.next_token_i)
            .is_some_and(|c| !c.is_ascii_whitespace())
        {
            self.next_token_i += 1;
        }

        if start_token_i == self.next_token_i {
            return None;
        }

        Some(&self.file_content[start_token_i..self.next_token_i])
    }

    fn curr_header(&mut self) -> Result<&mut Header, Error> {
        self.curr_header_path
            .resolve_mut(&mut self.database_header)
            .ok_or(Error::InternalLogicError)
    }

    fn process_comment_statement(&mut self) -> Result<StatementProcessed, Error> {
        let comment = self.advance_until_end_of_comment()?.to_string();

        if let Some(first_token) = comment.split_whitespace().next() {
            if first_token == "$t" {
                self.process_typesetting_comment(&comment)?;
            } else if first_token == "$j" {
                self.process_additional_information_comment(&comment)?;
            } else {
                let mut depth = -1;
                let mut curr_heading = "";
                let headings = ["####", "#*#*", "=-=-", "-.-."];

                for (index, heading) in headings.iter().enumerate() {
                    if first_token.starts_with(heading) {
                        curr_heading = heading;
                        depth = index as i32;
                    }
                }

                if depth != -1 {
                    self.process_header_comment(&comment, curr_heading, depth)?;
                    return Ok(StatementProcessed::HeaderStatement);
                }
            }
        }

        self.next_description = Some(comment.clone());
        if self.scope == 0 {
            self.curr_header()?
                .content
                .push(Statement::CommentStatement(Comment { text: comment }));
        }
        Ok(StatementProcessed::CommentStatement)
    }

    fn advance_until_end_of_comment(&mut self) -> Result<&str, Error> {
        let start_token_i = self.next_token_i;
        while let Some(token) = self.advance_next_token() {
            if token == "$)" {
                return Ok(&self.file_content[start_token_i..(self.next_token_i - 2)]);
            }
        }

        Err(Error::UnclosedCommentError)
    }

    fn process_additional_information_comment(&mut self, comment: &str) -> Result<(), Error> {
        let typesetting_tokens = super::tokenize_typesetting_text(comment)?;
        let mut typesetting_token_iter = typesetting_tokens.iter();

        typesetting_token_iter.next(); // Flush out leading "$j"

        loop {
            let mut statement_tokens: Vec<&str> = Vec::new();
            while let Some(&typesetting_token) = typesetting_token_iter.next() {
                if !typesetting_token.starts_with("/*") {
                    if typesetting_token != ";" {
                        statement_tokens.push(typesetting_token);
                    } else {
                        break;
                    }
                }
            }

            if statement_tokens.len() == 0 {
                break;
            }

            match statement_tokens[0] {
                keyword @ ("varcolorcode" | "altvarcolorcode") => {
                    if statement_tokens.len() != 4 || statement_tokens[2] != "as" {
                        return Err(Error::AdditionalInfoCommentFormatError);
                    }

                    let typecode = super::get_str_in_quotes(statement_tokens[1])
                        .ok_or(Error::AdditionalInfoCommentFormatError)?;
                    let color = super::get_str_in_quotes(statement_tokens[3])
                        .ok_or(Error::AdditionalInfoCommentFormatError)?;

                    if color.len() != 6
                        || !color
                            .chars()
                            .all(|c| matches!(c, '0'..='9' | 'a'..='f' | 'A'..='F'))
                    {
                        return Err(Error::InvalidColorCodeError);
                    }

                    let variable_colors = if keyword == "varcolorcode" {
                        &mut self.variable_colors
                    } else {
                        &mut self.alt_variable_colors
                    };

                    if variable_colors.iter().any(|vc| vc.typecode == typecode) {
                        return Err(Error::AdditionalInfoCommentFormatError);
                    }

                    variable_colors.push(VariableColor { typecode, color });
                }
                _ => {}
            }
        }

        Ok(())
    }

    fn process_typesetting_comment(&mut self, comment: &str) -> Result<(), Error> {
        let typesetting_tokens = super::tokenize_typesetting_text(comment)?;
        let mut typesetting_token_iter = typesetting_tokens.iter();

        typesetting_token_iter.next(); // Flush out leading "$t"

        loop {
            let mut statement_tokens: Vec<&str> = Vec::new();
            while let Some(&typesetting_token) = typesetting_token_iter.next() {
                if !typesetting_token.starts_with("/*") {
                    if typesetting_token != ";" {
                        statement_tokens.push(typesetting_token);
                    } else {
                        break;
                    }
                }
            }

            if statement_tokens.len() == 0 {
                break;
            }

            if statement_tokens[0] != "althtmldef" {
                continue;
            }

            if statement_tokens.len() < 4
                || statement_tokens.len() % 2 != 0
                || statement_tokens[2] != "as"
            {
                return Err(Error::TypesettingFormatError);
            }

            let mut html: String = super::get_str_in_quotes(statement_tokens[3])
                .ok_or(Error::TypesettingFormatError)?;

            let mut next_html_index = 5;

            while next_html_index < statement_tokens.len() {
                if statement_tokens[next_html_index - 1] != "+" {
                    return Err(Error::TypesettingFormatError);
                }
                html.push_str(
                    &super::get_str_in_quotes(statement_tokens[next_html_index])
                        .ok_or(Error::TypesettingFormatError)?,
                );

                next_html_index += 2;
            }

            let html_rep = HtmlRepresentation {
                symbol: super::get_str_in_quotes(statement_tokens[1])
                    .ok_or(Error::TypesettingFormatError)?
                    .to_string(),
                html,
            };

            if !html_validation::verify_html(
                &*html_rep.html,
                &self.html_allowed_tags_and_attributes,
                &self.css_allowed_properties,
            ) {
                self.invalid_html.push(html_rep.clone());
            }
            self.html_representations.push(html_rep);
        }

        Ok(())
    }

    fn process_header_comment(
        &mut self,
        comment: &str,
        curr_heading: &str,
        depth: i32,
    ) -> Result<(), Error> {
        let mut comment_iter = comment.split_ascii_whitespace();
        comment_iter.next(); // Flush out leading header marker

        let mut header_title = String::new();
        let mut header_closed = false;
        while let Some(token) = comment_iter.next() {
            if token.starts_with(curr_heading) {
                header_closed = true;
                break;
            }
            header_title.push_str(token);
            header_title.push(' ');
        }
        header_title.pop();

        if header_closed {
            let mut parent_header = &mut self.database_header;
            let mut next_header_path = HeaderPath::default();
            // let mut actual_depth = 0;
            for _ in 0..depth {
                if parent_header.subheaders.len() != 0 {
                    next_header_path
                        .path
                        .push(parent_header.subheaders.len() - 1);
                    parent_header = parent_header.subheaders.last_mut().unwrap()
                }
            }
            next_header_path.path.push(parent_header.subheaders.len());
            parent_header.subheaders.push(Header {
                title: header_title,
                content: Vec::new(),
                subheaders: Vec::new(),
            });
            self.next_description = None;
            self.curr_header_path = next_header_path;
            Ok(())
        } else {
            Err(Error::UnclosedHeaderError)
        }
    }

    fn process_opening_scope_statement(&mut self) -> StatementProcessed {
        self.scope += 1;
        self.active_vars.push(HashSet::new());
        self.active_float_hyps.push(Vec::new());
        self.active_dists.push(Vec::new());
        self.active_hyps.push(Vec::new());

        StatementProcessed::OpeningScopeStatement
    }

    fn process_closing_scope_statement(&mut self) -> Result<StatementProcessed, Error> {
        if self.scope == 0 {
            return Err(Error::ClosedUnopenedScopeError);
        }

        self.scope -= 1;

        self.active_vars
            .pop()
            .ok_or(Error::InternalLogicError)?
            .into_iter()
            .for_each(|s| {
                self.prev_variables.insert(s);
            });

        self.prev_float_hyps.append(
            &mut self
                .active_float_hyps
                .pop()
                .ok_or(Error::InternalLogicError)?,
        );

        self.active_dists.pop();
        self.active_hyps.pop();

        Ok(StatementProcessed::ClosingScopeStatement)
    }

    fn process_constant_statement(&mut self) -> Result<StatementProcessed, Error> {
        if self.scope != 0 {
            return Err(Error::ConstStatementScopeError);
        }

        let mut constants: Vec<Constant> = Vec::new();

        while let Some(const_token) = self.advance_next_token() {
            match const_token {
                "$(" => {
                    self.advance_until_end_of_comment()?;
                }
                "$." => break,
                const_symbol => {
                    if !util::is_valid_math_symbol(const_symbol) {
                        return Err(Error::InvalidSymbolError);
                    }

                    let const_symbol_string = const_symbol.to_string();

                    if self.active_consts.contains(&const_symbol_string)
                        || self.active_vars[0].contains(&const_symbol_string)
                        || self.prev_variables.contains(&const_symbol_string)
                    {
                        return Err(Error::TwiceDeclaredConstError);
                    }

                    constants.push(Constant {
                        symbol: const_symbol_string.clone(),
                    });

                    self.active_consts.insert(const_symbol_string);
                }
            }
        }

        if constants.is_empty() {
            return Err(Error::EmptyConstStatementError);
        }

        self.curr_header()?
            .content
            .push(Statement::ConstantStatement(constants));

        Ok(StatementProcessed::ConstantStatement)
    }

    fn process_variable_statement(&mut self) -> Result<StatementProcessed, Error> {
        let mut variables: Vec<Variable> = Vec::new();

        while let Some(var_token) = self.advance_next_token() {
            match var_token {
                "$(" => {
                    self.advance_until_end_of_comment()?;
                }
                "$." => break,
                var_symbol => {
                    if !util::is_valid_math_symbol(var_symbol) {
                        return Err(Error::InvalidSymbolError);
                    }

                    let var_symbol_string = var_symbol.to_string();

                    if self.active_consts.contains(&var_symbol_string) {
                        return Err(Error::TwiceDeclaredVarError);
                    }

                    if self.is_active_variable(&var_symbol_string) {
                        return Err(Error::TwiceDeclaredVarError);
                    }

                    // if !prev_variables.contains(&var_symbol) {
                    variables.push(Variable {
                        symbol: var_symbol_string.clone(),
                    });
                    // if self.scope == 0 {
                    //     metamath_data
                    //         .optimized_data
                    //         .variables
                    //         .insert(var_symbol_string.clone());
                    // }
                    // }

                    self.active_vars[self.scope].insert(var_symbol_string);
                }
            }
        }

        if variables.is_empty() {
            return Err(Error::EmptyVarStatementError);
        }

        if self.scope == 0 {
            self.curr_header()?
                .content
                .push(Statement::VariableStatement(variables));
        }

        Ok(StatementProcessed::VariableStatement)
    }

    fn is_active_variable(&self, symbol: &String) -> bool {
        self.active_vars.iter().any(|av| av.contains(symbol))
    }

    fn process_floating_hypothesis_statement(&mut self) -> Result<StatementProcessed, Error> {
        let mut non_comment_tokens: Vec<String> = Vec::new();

        while let Some(float_hyp_token) = self.advance_next_token() {
            match float_hyp_token {
                "$(" => {
                    self.advance_until_end_of_comment()?;
                }
                "$." => break,
                non_comment_token => non_comment_tokens.push(non_comment_token.to_string()),
            }
        }

        if non_comment_tokens.len() != 2 {
            return Err(Error::FloatHypStatementFormatError);
        }

        let mut non_comment_tokens_iter = non_comment_tokens.into_iter();

        let label = self
            .next_label
            .as_ref()
            .ok_or(Error::MissingLabelError)?
            .clone();
        self.next_label = None;
        let typecode = non_comment_tokens_iter
            .next()
            .ok_or(Error::InternalLogicError)?;
        let variable = non_comment_tokens_iter
            .next()
            .ok_or(Error::InternalLogicError)?;

        if !self.active_consts.contains(&typecode) {
            return Err(Error::FloatHypTypecodeError);
        }

        if !self.is_active_variable(&variable) {
            return Err(Error::FloatHypVariableError);
        }

        if self.var_type_already_declared(&variable) {
            return Err(Error::VarTypeDeclaredTwiceError);
        }

        if self.var_type_declared_previously_different_typecode(&typecode, &variable) {
            return Err(Error::VarDeclaredMultipleTypesError);
        }

        // TODO: check if order is same as locally declared
        if self.scope == 0 {
            self.curr_header()?
                .content
                .push(Statement::FloatingHypohesisStatement(FloatingHypothesis {
                    label: label.clone(),
                    typecode: typecode.clone(),
                    variable: variable.clone(),
                }));
        }

        self.active_float_hyps[self.scope].push(FloatingHypothesis {
            label,
            typecode,
            variable,
        });

        Ok(StatementProcessed::FloatingHypothesisStatement)
    }

    fn var_type_already_declared(&self, variable: &str) -> bool {
        self.active_float_hyps
            .iter()
            .any(|afh| afh.iter().any(|fh| fh.variable == variable))
    }

    fn var_type_declared_previously_different_typecode(
        &self,
        typecode: &str,
        variable: &str,
    ) -> bool {
        self.prev_float_hyps
            .iter()
            .any(|fh| fh.variable == variable && fh.typecode != typecode)
    }

    fn process_essential_hypothesis_statement(&mut self) -> Result<StatementProcessed, Error> {
        let label = self
            .next_label
            .as_ref()
            .ok_or(Error::MissingLabelError)?
            .clone();
        self.next_label = None;

        let expression = self.advance_expression_until("$.")?;

        self.active_hyps[self.scope].push(Hypothesis { label, expression });

        Ok(StatementProcessed::EssentialHypothesisStatement)
    }

    fn advance_expression_until(&mut self, until: &str) -> Result<String, Error> {
        let mut res = String::new();

        let mut first = true;

        while let Some(token) = self.advance_next_token() {
            if token == "$(" {
                self.advance_until_end_of_comment()?;
                continue;
            }

            if token == until {
                break;
            }

            let token_string = token.to_string();

            // If first is true, fail if token is not a const, else fail if it is neither a const nor a var
            if (!self.is_active_variable(&token_string) || first)
                && !self.active_consts.contains(&token_string)
            {
                return Err(Error::NonSymbolInExpressionError);
            }

            if !first {
                res.push(' ');
            }

            res.push_str(&token_string);
            first = false;
        }

        Ok(res)
    }

    fn process_distinct_variable_statement(&mut self) -> Result<StatementProcessed, Error> {
        let dist = self.advance_until_end_of_distinct_statement()?;

        if !dist.contains(' ') {
            return Err(Error::ZeroOrOneSymbolDisjError);
        }

        self.active_dists[self.scope].push(dist);

        Ok(StatementProcessed::DistinctVariableStatement)
    }

    fn advance_until_end_of_distinct_statement(&mut self) -> Result<String, Error> {
        let mut res = String::new();

        while let Some(token) = self.advance_next_token() {
            if token == "$(" {
                self.advance_until_end_of_comment()?;
                continue;
            }

            if token == "$." {
                break;
            }

            let token_string = token.to_string();

            if !self.is_active_variable(&token_string) {
                return Err(Error::NonVarInDisjError);
            }

            res.push_str(&token_string);
            res.push(' ');
        }

        res.pop();
        Ok(res)
    }

    fn process_theorem_statement(&mut self, is_axiom: bool) -> Result<StatementProcessed, Error> {
        let label = self
            .next_label
            .as_ref()
            .ok_or(Error::MissingLabelError)?
            .clone();
        self.next_label = None;

        let description = match self.next_description.clone() {
            Some(d) => {
                if self.scope == 0 {
                    self.curr_header()?.content.pop();
                }
                d
            }
            None => String::new(),
        };

        let distincts = self.active_dists.clone().into_iter().flatten().collect();
        let hypotheses = self.active_hyps.clone().into_iter().flatten().collect();

        let assertion_end_keyword = if is_axiom { "$." } else { "$=" };

        let assertion = self.advance_expression_until(assertion_end_keyword)?;

        let proof = if !is_axiom {
            Some(self.advance_statement_ignore_comments()?)
        } else {
            None
        };

        self.curr_header()?
            .content
            .push(Statement::TheoremStatement(Theorem {
                label,
                description,
                distincts,
                hypotheses,
                assertion,
                proof,
            }));
        self.theorem_amount += 1;

        Ok(StatementProcessed::TheoremStatement)
    }

    fn advance_statement_ignore_comments(&mut self) -> Result<String, Error> {
        let mut res = String::new();

        while let Some(token) = self.advance_next_token() {
            if token == "$(" {
                self.advance_until_end_of_comment()?;
                continue;
            }

            if token == "$." {
                break;
            }

            res.push_str(token);
            res.push(' ');
        }

        res.pop();
        Ok(res)
    }

    pub fn get_scope(&self) -> usize {
        return self.scope;
    }
}
