use std::fs;

use tauri::async_runtime::Mutex;

use crate::{
    model::{DatabaseElement, MetamathData, Statement::*},
    AppState, Error,
};

#[tauri::command]
pub async fn new_database(
    state: tauri::State<'_, Mutex<AppState>>,
    file_path: &str,
) -> Result<(), Error> {
    let mut app_state = state.lock().await;

    fs::write(file_path, "").or(Err(Error::FileWriteError))?;

    app_state.metamath_data = Some(MetamathData::default());
    app_state.metamath_data.as_mut().unwrap().database_path = file_path.to_string();

    Ok(())
}

#[tauri::command]
pub async fn save_database(state: tauri::State<'_, Mutex<AppState>>) -> Result<(), Error> {
    let app_state = state.lock().await;
    let mm_data = app_state.metamath_data.as_ref().ok_or(Error::NoMmDbError)?;

    let database_string = calc_database_string(mm_data);

    fs::write(&mm_data.database_path, database_string).or(Err(Error::FileWriteError))?;

    Ok(())
}

#[tauri::command]
pub async fn export_database(
    state: tauri::State<'_, Mutex<AppState>>,
    file_path: &str,
) -> Result<(), Error> {
    let app_state = state.lock().await;
    let mm_data = app_state.metamath_data.as_ref().ok_or(Error::NoMmDbError)?;

    let database_string = calc_database_string(mm_data);

    fs::write(file_path, database_string).or(Err(Error::FileWriteError))?;

    Ok(())
}

fn calc_database_string(metamath_data: &MetamathData) -> String {
    let mut res = String::new();

    for element in metamath_data.database_header.iter() {
        match element {
            DatabaseElement::Header(header, depth) => {
                res.push_str("$(\n");
                res.push_str(header_line(depth));
                res.push_str("\n  ");
                res.push_str(&header.title);
                res.push('\n');
                res.push_str(header_line(depth));
                res.push('\n');
                res.push_str("$)\n\n");
            }
            DatabaseElement::Statement(statement) => match statement {
                CommentStatement(comment) => {
                    res.push_str("$(");
                    write_text_wrapped(&mut res, &comment.text, "  ");
                    write_text_wrapped(&mut res, "$)", "  ");
                    res.push_str("\n\n");
                }
                ConstantStatement(constants) => {
                    res.push_str("$c ");
                    for constant in constants {
                        res.push_str(&constant.symbol);
                        res.push(' ');
                    }
                    res.push_str("$.\n\n");
                }
                VariableStatement(variables) => {
                    res.push_str("$v ");
                    for variable in variables {
                        res.push_str(&variable.symbol);
                        res.push(' ');
                    }
                    res.push_str("$.\n\n");
                }
                FloatingHypohesisStatement(floating_hypothesis) => {
                    res.push_str(&floating_hypothesis.label);
                    res.push_str(" $f ");
                    res.push_str(&floating_hypothesis.typecode);
                    res.push(' ');
                    res.push_str(&floating_hypothesis.variable);
                    res.push_str(" $.\n\n");
                }
                TheoremStatement(theorem) => {
                    res.push_str("${\n");
                    for dist_vars in &theorem.disjoints {
                        res.push_str("  $d ");
                        res.push_str(dist_vars);
                        res.push_str(" $.\n")
                    }
                    for hyp in &theorem.hypotheses {
                        res.push_str("  ");
                        res.push_str(&hyp.label);
                        res.push_str(" $e ");
                        res.push_str(&hyp.hypothesis);
                        res.push_str(" $.\n");
                    }
                    res.push_str("  $(");
                    write_text_wrapped(&mut res, &theorem.description, "    ");
                    write_text_wrapped(&mut res, "$)", "    ");
                    res.push('\n');

                    res.push_str("  ");
                    res.push_str(&theorem.label);
                    match &theorem.proof {
                        None => {
                            write_text_wrapped(&mut res, "$a", "    ");
                            write_text_wrapped(&mut res, &theorem.assertion, "    ");
                            write_text_wrapped(&mut res, "$.", "    ");
                            res.push('\n');
                        }
                        Some(proof) => {
                            write_text_wrapped(&mut res, "$p", "    ");
                            write_text_wrapped(&mut res, &theorem.assertion, "    ");
                            write_text_wrapped(&mut res, "$=", "    ");
                            write_text_wrapped(&mut res, proof, "    ");
                            write_text_wrapped(&mut res, "$.", "    ");
                            res.push('\n');
                        }
                    }
                    res.push_str("$}\n\n");
                }
            },
        }
    }

    res
}

fn write_text_wrapped(target: &mut String, text: &str, line_prefix: &str) {
    let line_legth = 80;
    let mut curr_line_length = last_line_length(&target) as usize;

    for token in text.split_ascii_whitespace() {
        if curr_line_length + 1 + token.len() < line_legth {
            target.push(' ');
            target.push_str(token);
            curr_line_length += 1 + token.len();
        } else {
            target.push('\n');
            target.push_str(line_prefix);
            target.push_str(token);
            curr_line_length = line_prefix.len() + token.len();
        }
    }
}

fn last_line_length(text: &str) -> u32 {
    let mut len = 0;
    let mut index = text.len() - 1;

    while index > 0 && text.as_bytes()[index] != b'\n' {
        len += 1;
        index -= 1;
    }

    len
}

fn header_line(depth: u32) -> &'static str {
    match depth {
        1 => "###############################################################################",
        2 => "#*#*#*#*#*#*#*#*#*#*#*#*#*#*#*#*#*#*#*#*#*#*#*#*#*#*#*#*#*#*#*#*#*#*#*#*#*#*#*#",
        3 => "=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=",
        4 => "-.-.-.-.-.-.-.-.-.-.-.-.-.-.-.-.-.-.-.-.-.-.-.-.-.-.-.-.-.-.-.-.-.-.-.-.-.-.-.-",
        _ => "",
    }
}
