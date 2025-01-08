use std::fs::read_to_string;

use sqlx::SqliteConnection;

use crate::{
    database::{constant::add_constant_database, variable::add_variable_database},
    model::{Constant, MetamathData, Variable},
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

    let mut last_comment_str_vec: Vec<&str> = Vec::new();

    let mut next_const_index = 0;
    let mut next_var_index = 0;
    let mut symbols_declared: Vec<Vec<&str>> = vec![Vec::new()];
    let mut prev_variables: Vec<&str> = Vec::new();

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
            "$(" => last_comment_str_vec = get_next_as_str_vec_until(&mut token_iter, "$)"),
            "${" => {
                scope += 1;
                symbols_declared.push(Vec::new());
            }
            "$}" => {
                if scope == 0 {
                    return Err(Error::ClosedUnopenedScopeError);
                }

                scope -= 1;
                let mut scoped_vars = symbols_declared.pop().unwrap();
                prev_variables.append(&mut scoped_vars);
            }
            "$c" => {
                if scope != 0 {
                    return Err(Error::ConstStatementScopeError);
                }

                let mut at_least_one_symbol = false;

                while let Some(const_token) = token_iter.next() {
                    match const_token {
                        "$." => {
                            if at_least_one_symbol {
                                break;
                            } else {
                                return Err(Error::EmptyConstStatementError);
                            }
                        }
                        "$(" => {
                            get_next_until(&mut token_iter, "$)");
                        }
                        const_symbol => {
                            if !is_valid_math_symbol(const_symbol) {
                                return Err(Error::InvalidSymbolError);
                            }

                            if symbols_declared[0].contains(&const_symbol)
                                || prev_variables.contains(&const_symbol)
                            {
                                return Err(Error::TwiceDeclaredConstError);
                            }

                            metamath_data.constants.push(Constant {
                                symbol: const_symbol.to_string(),
                            });
                            add_constant_database(conn, next_const_index, const_symbol).await?;

                            next_const_index += 1;
                            symbols_declared[0].push(const_symbol);
                            at_least_one_symbol = true;
                        }
                    }
                }
            }
            "$v" => {
                let mut at_least_one_symbol = false;

                while let Some(var_token) = token_iter.next() {
                    match var_token {
                        "$." => {
                            if at_least_one_symbol {
                                break;
                            } else {
                                return Err(Error::EmptyVarStatementError);
                            }
                        }
                        "$(" => {
                            get_next_until(&mut token_iter, "$)");
                        }
                        var_symbol => {
                            if !is_valid_math_symbol(var_symbol) {
                                return Err(Error::InvalidSymbolError);
                            }

                            for scope_symbols_declared in &symbols_declared {
                                if scope_symbols_declared.contains(&var_symbol) {
                                    return Err(Error::TwiceDeclaredVarError);
                                }
                            }

                            if !prev_variables.contains(&var_symbol) {
                                metamath_data.variables.push(Variable {
                                    symbol: var_symbol.to_string(),
                                });
                                add_variable_database(conn, next_var_index, var_symbol).await?;

                                next_var_index += 1;
                            }

                            symbols_declared[scope].push(var_symbol);
                            at_least_one_symbol = true;
                        }
                    }
                }
            }
            _ => {}
        }
        tokens_processed += 1;
        if tokens_processed % 100_000 == 0 {
            println!(
                "Tokens processed: {}, Symbols: {}, Previous Variables: {}",
                tokens_processed,
                symbols_declared[0].len(),
                prev_variables.len(),
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
