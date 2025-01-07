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
    // let file = File::open(mm_file_path).or(Err(Error::FileNotFoundError))?;

    // let lines_iter = BufReader::new(file).lines();

    let file_content = read_to_string(mm_file_path).unwrap();

    if !file_content.is_ascii() {
        return Err(Error::InvalidCharactersError);
    }

    let mut last_comment_str_vec: Vec<&str> = Vec::new();

    let mut next_const_index = 0;
    let mut next_var_index = 0;

    let mut token_iter = file_content.split_whitespace();

    while let Some(token) = token_iter.next() {
        match token {
            "$(" => last_comment_str_vec = get_next_as_str_vec_until(&mut token_iter, "$)"),
            "$c" => {
                while let Some(const_token) = token_iter.next() {
                    match const_token {
                        "$." => break,
                        const_symbol => {
                            metamath_data.constants.push(Constant {
                                symbol: const_symbol.to_string(),
                            });
                            add_constant_database(conn, next_const_index, const_symbol).await?;
                            next_const_index += 1;
                        }
                    }
                }
            }
            "$v" => {
                while let Some(var_token) = token_iter.next() {
                    match var_token {
                        "$." => break,
                        var_symbol => {
                            metamath_data.variables.push(Variable {
                                symbol: var_symbol.to_string(),
                            });
                            add_variable_database(conn, next_var_index, var_symbol).await?;
                            next_var_index += 1;
                        }
                    }
                }
            }
            _ => {}
        }
    }

    Ok(())
}

fn get_next_as_str_vec_until<'a>(
    iter: &'a mut std::str::SplitWhitespace,
    until: &str,
) -> Vec<&'a str> {
    let mut result: Vec<&str> = Vec::new();
    while let Some(token) = iter.next() {
        if token == until {
            break;
        } else {
            result.push(token);
        }
    }
    result
}
