use std::fs::read_to_string;

use sqlx::SqliteConnection;

use crate::{
    database::{
        constant::add_constant_database_raw,
        floating_hypothesis::add_floating_hypothesis_database_raw,
        variable::add_variable_database_raw,
    },
    model::{Constant, FloatingHypohesis, Header, Hypothesis, MetamathData, Theorem, Variable},
    Error,
};

pub async fn parse_mm_file(
    mm_file_path: &str,
    conn: &mut SqliteConnection,
    metamath_data: &mut MetamathData,
) -> Result<(), Error> {
    let file_content = read_to_string(mm_file_path).unwrap();

    if !file_content.is_ascii() {
        return Err(Error::InvalidCharactersError);
    }

    // Scope starting at 0, +1 for every "${", -1 for every "$}""
    let mut scope = 0;

    let mut last_comment: String = String::new();

    let mut next_label: Option<&str> = None;

    let mut next_const_index = 0;
    let mut active_consts: Vec<&str> = Vec::new();

    let mut next_var_index = 0;
    let mut active_vars: Vec<Vec<&str>> = vec![Vec::new()];
    let mut prev_variables: Vec<&str> = Vec::new();

    let mut next_float_hyp_index = 0;
    let mut active_float_hyps: Vec<Vec<RefFloatingHypothesis>> = vec![Vec::new()];
    let mut prev_float_hyps: Vec<RefFloatingHypothesis> = Vec::new();

    let mut active_disjs: Vec<Vec<String>> = vec![Vec::new()];

    let mut active_hyps: Vec<Vec<Hypothesis>> = vec![Vec::new()];

    let mut curr_header: &mut Header = &mut metamath_data.theorem_list_header;

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
            "$(" => last_comment = super::get_next_as_string_until(&mut token_iter, "$)"),
            "${" => {
                scope += 1;
                active_vars.push(Vec::new());
                active_float_hyps.push(Vec::new());
                active_disjs.push(Vec::new());
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
                active_disjs.pop();
                active_hyps.pop();
            }
            "$c" => {
                if scope != 0 {
                    return Err(Error::ConstStatementScopeError);
                }

                let mut at_least_one_symbol = false;

                while let Some(const_token) = token_iter.next() {
                    match const_token {
                        "$(" => get_next_until(&mut token_iter, "$)"),
                        "$." if at_least_one_symbol => break,
                        "$." => return Err(Error::EmptyConstStatementError),
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

                            metamath_data.constants.push(Constant {
                                symbol: const_symbol.to_string(),
                            });
                            add_constant_database_raw(conn, next_const_index, const_symbol).await?;

                            next_const_index += 1;
                            active_consts.push(const_symbol);
                            at_least_one_symbol = true;
                        }
                    }
                }
            }
            "$v" => {
                let mut at_least_one_symbol = false;

                while let Some(var_token) = token_iter.next() {
                    match var_token {
                        "$(" => get_next_until(&mut token_iter, "$)"),
                        "$." if at_least_one_symbol => break,
                        "$." => return Err(Error::EmptyVarStatementError),
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

                            if !prev_variables.contains(&var_symbol) {
                                metamath_data.variables.push(Variable {
                                    symbol: var_symbol.to_string(),
                                });
                                add_variable_database_raw(conn, next_var_index, var_symbol).await?;

                                next_var_index += 1;
                            }

                            active_vars[scope].push(var_symbol);
                            at_least_one_symbol = true;
                        }
                    }
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

                if !var_type_already_declared_previously(typecode, variable, &prev_float_hyps)? {
                    metamath_data.floating_hypotheses.push(FloatingHypohesis {
                        label: label.to_string(),
                        typecode: typecode.to_string(),
                        variable: variable.to_string(),
                    });
                    add_floating_hypothesis_database_raw(
                        conn,
                        next_float_hyp_index,
                        label,
                        typecode,
                        variable,
                    )
                    .await?;

                    next_float_hyp_index += 1;
                }

                active_float_hyps[scope].push(RefFloatingHypothesis { typecode, variable });
            }
            "$e" => {
                let label = next_label.ok_or(Error::MissingLabelError)?.to_string();
                next_label = None;

                let hypothesis = get_next_as_string_check_expression(&mut token_iter, "$.", &active_consts, &active_vars)?;

                active_hyps[scope].push(Hypothesis {
                    label,
                    hypothesis,
                });
            }
            "$d" => {
                let disj = get_next_as_string_until_check_vars(&mut token_iter, "$.", &active_vars)?;

                if !disj.contains(' ') {
                    return Err(Error::ZeroOrOneSymbolDisjError)
                }

                active_disjs[scope].push(disj);
            }
            "$a" => {
                let label = next_label.ok_or(Error::MissingLabelError)?.to_string();
                next_label = None;

                let assertion = get_next_as_string_check_expression(&mut token_iter, "$.", &active_consts, &active_vars)?;

                curr_header.theorems.push(Theorem {
                    name: label,
                    description: last_comment.clone(),
                    disjoints: active_disjs.clone().into_iter().flatten().collect(),
                    hypotheses: active_hyps.clone().into_iter().flatten().collect(),
                    assertion,
                    proof: None
                });
            }
            label /*if next_label.is_none()*/ => next_label = Some(label),
            _unknown_token => {} //return Err(Error::TokenOutsideStatementError),
        }
        tokens_processed += 1;
        if tokens_processed % 100_000 == 0 {
            println!(
                "Tokens processed: {}, Consts: {}, Outermost Vars: {}",
                tokens_processed,
                active_consts.len(),
                active_vars[0].len(),
            );
        }
    }

    Ok(())
}

fn get_next_until(token_iter: &mut std::str::SplitWhitespace, until: &str) {
    while let Some(token) = token_iter.next() {
        if token == until {
            break;
        }
    }
}

fn get_next_as_str_vec_until<'a>(
    token_iter: &'a mut std::str::SplitWhitespace,
    until: &str,
) -> Vec<&'a str> {
    let mut result: Vec<&str> = Vec::new();
    while let Some(token) = token_iter.next() {
        if token == until {
            break;
        } else {
            result.push(token);
        }
    }
    result
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

// Checks if variable is declared by hypothesis in prev_float_hyps and returns an Error if it has, but with a different typecode
fn var_type_already_declared_previously(
    typecode: &str,
    variable: &str,
    prev_float_hyps: &Vec<RefFloatingHypothesis>,
) -> Result<bool, Error> {
    for float_hyp in prev_float_hyps {
        if float_hyp.variable == variable {
            if float_hyp.typecode != typecode {
                return Err(Error::VarDeclaredMultipleTypesError);
            }
            return Ok(true);
        }
    }
    Ok(false)
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

struct RefFloatingHypothesis<'a> {
    pub typecode: &'a str,
    pub variable: &'a str,
}
