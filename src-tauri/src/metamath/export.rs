use std::fs;

use tauri::async_runtime::Mutex;

use crate::{
    model::{DatabaseElement, MetamathData, Statement::*},
    AppState, Error,
};

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
                res.push('\n');
                res.push_str(&header.title);
                res.push('\n');
                res.push_str(header_line(depth));
                res.push('\n');
                res.push_str("$)\n\n");
            }
            DatabaseElement::Statement(statement) => match statement {
                ConstantStatement(constant) => {
                    res.push_str("$c ");
                    res.push_str(&constant.symbol);
                    res.push_str(" $.\n\n");
                }
                VariableStatement(variable) => {
                    res.push_str("$v ");
                    res.push_str(&variable.symbol);
                    res.push_str(" $.\n\n");
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
                    res.push_str("  $( ");
                    res.push_str(&theorem.description);
                    res.push_str("\n  $)\n");

                    res.push_str("  ");
                    res.push_str(&theorem.label);
                    match &theorem.proof {
                        None => {
                            res.push_str(" $a ");
                            res.push_str(&theorem.assertion);
                            res.push_str("\n $.\n");
                        }
                        Some(proof) => {
                            res.push_str(" $p ");
                            res.push_str(&theorem.assertion);
                            res.push_str("\n $= ");
                            res.push_str(proof);
                            res.push_str("\n $.\n");
                        }
                    }
                    res.push_str("$}\n\n");
                }
            },
        }
    }

    res
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
