use std::fs;

use tauri::async_runtime::Mutex;

use crate::{
    metamath::mmp_parser::LocateAfterRef,
    model::{DatabaseElement, Statement, Variable, VariablesPageData},
    util::{self, ForEachWhile},
    AppState, Error,
};

#[tauri::command]
pub async fn get_variable_statement(
    state: tauri::State<'_, Mutex<AppState>>,
    any_variable: &str,
) -> Result<VariablesPageData, Error> {
    let app_state = state.lock().await;
    let metamath_data = app_state.metamath_data.as_ref().ok_or(Error::NoMmDbError)?;

    let mut db_iter = metamath_data.database_header.iter();

    let mut variables: Vec<(Variable, String)> = db_iter
        .find_map(|db_element| match db_element {
            DatabaseElement::Statement(s) => match s {
                Statement::VariableStatement(vars) => {
                    if vars.iter().any(|v| v.symbol == any_variable) {
                        Some(vars.iter().map(|v| (v.clone(), String::new())).collect())
                    } else {
                        None
                    }
                }
                _ => None,
            },
            _ => None,
        })
        .ok_or(Error::NotFoundError)?;

    let mut typecodes_found = 0;

    db_iter.for_each_while(|db_element| {
        match db_element {
            DatabaseElement::Statement(s) => match s {
                Statement::FloatingHypohesisStatement(fh) => {
                    variables
                        .iter_mut()
                        .find(|(v, _)| v.symbol == fh.variable)
                        .map(|tuple| {
                            typecodes_found += 1;
                            tuple.1 = fh.typecode.clone();
                        });
                }
                _ => {}
            },
            _ => {}
        }

        // only continue if there are still typecodes to be found
        typecodes_found != variables.len()
    });

    Ok(VariablesPageData { variables })
}

#[tauri::command]
pub async fn get_variable_mmp_format(
    state: tauri::State<'_, Mutex<AppState>>,
    any_variable: &str,
) -> Result<String, Error> {
    let app_state = state.lock().await;
    let mm_data = app_state.metamath_data.as_ref().ok_or(Error::NoMmDbError)?;

    util::locate_after_to_mmp_file_format_of_statement_it_refers_to(
        LocateAfterRef::LocateAfterVar(any_variable),
        mm_data,
    )
}

#[tauri::command]
pub async fn write_variable_mmp_format_to_file(
    state: tauri::State<'_, Mutex<AppState>>,
    any_variable: &str,
    file_path: &str,
) -> Result<(), Error> {
    let app_state = state.lock().await;
    let mm_data = app_state.metamath_data.as_ref().ok_or(Error::NoMmDbError)?;

    let mmp_format = util::locate_after_to_mmp_file_format_of_statement_it_refers_to(
        LocateAfterRef::LocateAfterVar(any_variable),
        mm_data,
    )?;

    fs::write(file_path, mmp_format).map_err(|_| Error::FileWriteError)?;

    Ok(())
}
