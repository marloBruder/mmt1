use std::fs::read_to_string;

use tauri::async_runtime::Mutex;

use crate::{
    model::{
        Comment, Constant, FloatingHypohesis, Header, HeaderRepresentation, HtmlRepresentation,
        Hypothesis, MetamathData, Statement::*, Theorem, Variable,
    },
    AppState, Error,
};

#[tauri::command]
pub async fn open_metamath_database(
    state: tauri::State<'_, Mutex<AppState>>,
    mm_file_path: &str,
) -> Result<(HeaderRepresentation, Vec<HtmlRepresentation>), Error> {
    let mut metamath_data: MetamathData = Default::default();

    parse_mm_file(mm_file_path, &mut metamath_data).await?;

    let mut app_state = state.lock().await;

    let header_rep = metamath_data.database_header.to_representation();

    let html_reps = metamath_data.html_representations.clone();

    app_state.metamath_data = Some(metamath_data);

    Ok((header_rep, html_reps))
}

pub async fn parse_mm_file(
    mm_file_path: &str,
    metamath_data: &mut MetamathData,
) -> Result<(), Error> {
    let file_content = read_to_string(mm_file_path).unwrap();

    if !file_content.is_ascii() {
        return Err(Error::InvalidCharactersError);
    }

    metamath_data.database_path = mm_file_path.to_string();

    // Scope starting at 0, +1 for every "${", -1 for every "$}""
    let mut scope = 0;

    // let mut last_comment: String = String::new();

    let mut next_label: Option<&str> = None;

    let mut active_consts: Vec<&str> = Vec::new();

    let mut active_vars: Vec<Vec<&str>> = vec![Vec::new()];
    let mut prev_variables: Vec<&str> = Vec::new();

    let mut active_float_hyps: Vec<Vec<RefFloatingHypothesis>> = vec![Vec::new()];
    let mut prev_float_hyps: Vec<RefFloatingHypothesis> = Vec::new();

    let mut active_dists: Vec<Vec<String>> = vec![Vec::new()];

    let mut active_hyps: Vec<Vec<Hypothesis>> = vec![Vec::new()];

    let mut curr_header: &mut Header = &mut metamath_data.database_header;

    let mut token_iter = file_content.split_whitespace();

    // let mut token_iter = file_content
    //     .lines()
    //     .enumerate()
    //     .flat_map(|(line_number, line)| {
    //         line.split_whitespace()
    //             .map(move |token| (line_number + 1, token))
    //     });

    let mut tokens_processed: i64 = 0;

    while let Some(token) = token_iter.next() {
        match token {
            "$(" => {
                let comment = super::get_next_as_string_until(&mut token_iter, "$)");
                let mut comment_iter = comment.split_whitespace();

                if let Some(first_token) = comment_iter.next() {
                    if first_token == "$t" {
                        let typesetting_tokens = super::tokenize_typesetting_text(&comment)?;
                        let mut typesetting_token_iter = typesetting_tokens.iter();

                        typesetting_token_iter.next(); // Flush out leading "$t"

                        let mut html_representations = Vec::new();

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

                            html_representations.push(HtmlRepresentation {
                                symbol: super::get_str_in_quotes(statement_tokens[1])
                                    .ok_or(Error::TypesettingFormatError)?
                                    .to_string(),
                                html,
                            })
                        }

                        metamath_data.html_representations = html_representations.clone();
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
                                let mut parent_header = &mut metamath_data.database_header;
                                // let mut actual_depth = 0;
                                for _ in 0..depth {
                                    parent_header = if parent_header.subheaders.len() != 0 {
                                        // actual_depth += 1;
                                        parent_header.subheaders.last_mut().unwrap()
                                    } else {
                                        parent_header
                                    }
                                }
                                parent_header.subheaders.push(Header {
                                    title: header_title,
                                    content: Vec::new(),
                                    subheaders: Vec::new(),
                                });
                                curr_header = parent_header.subheaders.last_mut().unwrap();
                            }
                        } else {
                            curr_header.content.push(CommentStatement(Comment {
                                text: comment.clone(),
                            }));
                        }
                    }
                }
            }
            "${" => {
                scope += 1;
                active_vars.push(Vec::new());
                active_float_hyps.push(Vec::new());
                active_dists.push(Vec::new());
                active_hyps.push(Vec::new());
            }
            "$}" => {
                if scope == 0 {
                    return Err(Error::ClosedUnopenedScopeError);
                }

                scope -= 1;

                let mut scoped_vars = active_vars.pop().unwrap();
                prev_variables.append(&mut scoped_vars);
                let mut scoped_float_hyps = active_float_hyps.pop().unwrap();
                prev_float_hyps.append(&mut scoped_float_hyps);
                active_dists.pop();
                active_hyps.pop();
            }
            "$c" => {
                if scope != 0 {
                    return Err(Error::ConstStatementScopeError);
                }

                let mut constants: Vec<Constant> = Vec::new();

                while let Some(const_token) = token_iter.next() {
                    match const_token {
                        "$(" => get_next_until(&mut token_iter, "$)"),
                        "$." => break,
                        const_symbol => {
                            if !is_valid_math_symbol(const_symbol) {
                                return Err(Error::InvalidSymbolError);
                            }

                            if active_consts.contains(&const_symbol)
                                || active_vars[0].contains(&const_symbol)
                                || prev_variables.contains(&const_symbol)
                            {
                                return Err(Error::TwiceDeclaredConstError);
                            }

                            constants.push(Constant {
                                symbol: const_symbol.to_string(),
                            });

                            active_consts.push(const_symbol);
                        }
                    }
                }

                if constants.is_empty() {
                    return Err(Error::EmptyConstStatementError);
                }

                curr_header.content.push(ConstantStatement(constants));
            }
            "$v" => {
                let mut variables: Vec<Variable> = Vec::new();

                while let Some(var_token) = token_iter.next() {
                    match var_token {
                        "$(" => get_next_until(&mut token_iter, "$)"),
                        "$." => break,
                        var_symbol => {
                            if !is_valid_math_symbol(var_symbol) {
                                return Err(Error::InvalidSymbolError);
                            }

                            if active_consts.contains(&var_symbol) {
                                return Err(Error::TwiceDeclaredVarError);
                            }

                            if is_active_variable(var_symbol, &active_vars) {
                                return Err(Error::TwiceDeclaredVarError);
                            }

                            // if !prev_variables.contains(&var_symbol) {
                            variables.push(Variable {
                                symbol: var_symbol.to_string(),
                            });
                            if scope == 0 {
                                metamath_data
                                    .optimized_data
                                    .variables
                                    .insert(var_symbol.to_string());
                            }
                            // }

                            active_vars[scope].push(var_symbol);
                        }
                    }
                }

                if variables.is_empty() {
                    return Err(Error::EmptyVarStatementError);
                }

                if scope == 0 {
                    curr_header.content.push(VariableStatement(variables));
                }
            }
            "$f" => {
                let mut non_comment_tokens: Vec<&str> = Vec::new();

                while let Some(float_hyp_token) = token_iter.next() {
                    match float_hyp_token {
                        "$(" => get_next_until(&mut token_iter, "$)"),
                        "$." => break,
                        non_comment_token => non_comment_tokens.push(non_comment_token),
                    }
                }

                if non_comment_tokens.len() != 2 {
                    return Err(Error::FloatHypStatementFormatError);
                }

                let label = next_label.ok_or(Error::MissingLabelError)?;
                next_label = None;
                let typecode = non_comment_tokens[0];
                let variable = non_comment_tokens[1];

                if !active_consts.contains(&typecode) {
                    return Err(Error::FloatHypTypecodeError);
                }

                if !is_active_variable(variable, &active_vars) {
                    return Err(Error::FloatHypVariableError);
                }

                if var_type_already_declared(variable, &active_float_hyps) {
                    return Err(Error::VarTypeDeclaredTwiceError);
                }

                if var_type_declared_previously_different_typecode(
                    typecode,
                    variable,
                    &prev_float_hyps,
                ) {
                    return Err(Error::VarDeclaredMultipleTypesError);
                }

                // TODO: check if order is same as locally declared
                if scope == 0 {
                    curr_header
                        .content
                        .push(FloatingHypohesisStatement(FloatingHypohesis {
                            label: label.to_string(),
                            typecode: typecode.to_string(),
                            variable: variable.to_string(),
                        }));
                    metamath_data
                        .optimized_data
                        .floating_hypotheses
                        .push(FloatingHypohesis {
                            label: label.to_string(),
                            typecode: typecode.to_string(),
                            variable: variable.to_string(),
                        });
                }

                active_float_hyps[scope].push(RefFloatingHypothesis { typecode, variable });
            }
            "$e" => {
                let label = next_label.ok_or(Error::MissingLabelError)?.to_string();
                next_label = None;

                let expression = get_next_as_string_check_expression(
                    &mut token_iter,
                    "$.",
                    &active_consts,
                    &active_vars,
                )?;

                active_hyps[scope].push(Hypothesis { label, expression });
            }
            "$d" => {
                let dist =
                    get_next_as_string_until_check_vars(&mut token_iter, "$.", &active_vars)?;

                if !dist.contains(' ') {
                    return Err(Error::ZeroOrOneSymbolDisjError);
                }

                active_dists[scope].push(dist);
            }
            keyword @ ("$a" | "$p") => {
                let label = next_label.ok_or(Error::MissingLabelError)?.to_string();
                next_label = None;

                let description = if let Some(CommentStatement(_)) = curr_header.content.last() {
                    if let Some(CommentStatement(comment)) = curr_header.content.pop() {
                        comment.text
                    } else {
                        // Can't happen
                        String::new()
                    }
                } else {
                    String::new()
                };

                let distincts = active_dists.clone().into_iter().flatten().collect();
                let hypotheses = active_hyps.clone().into_iter().flatten().collect();

                let assertion_end_keyword = if keyword == "$a" { "$." } else { "$=" };

                let assertion = get_next_as_string_check_expression(
                    &mut token_iter,
                    assertion_end_keyword,
                    &active_consts,
                    &active_vars,
                )?;
                let mut proof = None;

                if keyword == "$p" {
                    proof = Some(get_next_as_string_until_ignore_comments(
                        &mut token_iter,
                        "$.",
                    ));
                }

                curr_header.content.push(TheoremStatement(Theorem {
                    label,
                    description,
                    distincts,
                    hypotheses,
                    assertion,
                    proof,
                }));
            }
            label => {
                if next_label.is_none() {
                    next_label = Some(label);
                } else {
                    return Err(Error::TokenOutsideStatementError);
                }
            }
        }
        tokens_processed += 1;
        if tokens_processed % 10_000 == 0 {
            println!(
                "Tokens processed: {}, Consts: {}, Outermost Vars: {}",
                tokens_processed,
                active_consts.len(),
                active_vars[0].len(),
            );
        }
    }

    metamath_data.recalc_symbol_number_mapping_and_grammar()?;
    metamath_data.calc_optimized_theorem_data()?;

    Ok(())
}

fn get_next_until(token_iter: &mut std::str::SplitWhitespace, until: &str) {
    while let Some(token) = token_iter.next() {
        if token == until {
            break;
        }
    }
}

fn is_valid_math_symbol(symbol: &str) -> bool {
    for byte in symbol.bytes() {
        match byte {
            33..=35 | 37..=126 => {}
            _ => return false,
        }
    }
    true
}

fn is_active_variable(symbol: &str, active_vars: &Vec<Vec<&str>>) -> bool {
    for scope_active_vars in active_vars {
        if scope_active_vars.contains(&symbol) {
            return true;
        }
    }
    false
}

fn var_type_already_declared(
    variable: &str,
    active_float_hyps: &Vec<Vec<RefFloatingHypothesis>>,
) -> bool {
    for scope_active_float_hyps in active_float_hyps {
        for float_hyp in scope_active_float_hyps {
            if float_hyp.variable == variable {
                return true;
            }
        }
    }
    false
}

// Checks if variable is declared by hypothesis in prev_float_hyps
// and returns an Error if it has been declared with a different typecode
// fn var_type_already_declared_previously(
//     typecode: &str,
//     variable: &str,
//     prev_float_hyps: &Vec<RefFloatingHypothesis>,
// ) -> Result<bool, Error> {
//     for float_hyp in prev_float_hyps {
//         if float_hyp.variable == variable {
//             if float_hyp.typecode != typecode {
//                 return Err(Error::VarDeclaredMultipleTypesError);
//             }
//             return Ok(true);
//         }
//     }
//     Ok(false)
// }

fn var_type_declared_previously_different_typecode(
    typecode: &str,
    variable: &str,
    prev_float_hyps: &Vec<RefFloatingHypothesis>,
) -> bool {
    for float_hyp in prev_float_hyps {
        if float_hyp.variable == variable && float_hyp.typecode != typecode {
            return true;
        }
    }
    false
}

fn get_next_as_string_until_check_vars(
    token_iter: &mut std::str::SplitWhitespace,
    until: &str,
    active_vars: &Vec<Vec<&str>>,
) -> Result<String, Error> {
    let mut res = String::new();

    while let Some(token) = token_iter.next() {
        if token == "$(" {
            get_next_until(token_iter, "$)");
            continue;
        }

        if token == until {
            break;
        }

        if !is_active_variable(token, active_vars) {
            return Err(Error::NonVarInDisjError);
        }

        res.push_str(token);
        res.push(' ');
    }

    res.pop();
    Ok(res)
}

fn get_next_as_string_check_expression(
    token_iter: &mut std::str::SplitWhitespace,
    until: &str,
    active_consts: &Vec<&str>,
    active_vars: &Vec<Vec<&str>>,
) -> Result<String, Error> {
    let mut res = String::new();

    let mut first = true;

    while let Some(token) = token_iter.next() {
        if token == "$(" {
            get_next_until(token_iter, "$)");
            continue;
        }

        if token == until {
            break;
        }

        // If first is true, fail if token is not a const, else fail if it is neither a const nor a var
        if (!is_active_variable(token, active_vars) || first) && !active_consts.contains(&token) {
            return Err(Error::NonSymbolInExpressionError);
        }

        res.push_str(token);
        res.push(' ');
        first = false;
    }

    res.pop();
    Ok(res)
}

fn get_next_as_string_until_ignore_comments(
    token_iter: &mut std::str::SplitWhitespace,
    until: &str,
) -> String {
    let mut res = String::new();

    while let Some(token) = token_iter.next() {
        if token == "$(" {
            get_next_until(token_iter, "$)");
            continue;
        }

        if token == until {
            break;
        }

        res.push_str(token);
        res.push(' ');
    }

    res.pop();
    res
}

struct RefFloatingHypothesis<'a> {
    pub typecode: &'a str,
    pub variable: &'a str,
}
