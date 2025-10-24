use std::fs;

use tauri::async_runtime::Mutex;

use crate::{
    metamath::mmp_parser::LocateAfterRef,
    model::{ConstantsPageData, DatabaseElement, Statement},
    util, AppState, Error,
};

#[tauri::command]
pub async fn get_constant_statement(
    state: tauri::State<'_, Mutex<AppState>>,
    any_constant: &str,
) -> Result<ConstantsPageData, Error> {
    let app_state = state.lock().await;
    let metamath_data = app_state.metamath_data.as_ref().ok_or(Error::NoMmDbError)?;

    Ok(ConstantsPageData {
        constants: metamath_data
            .database_header
            .iter()
            .find_map(|db_element| match db_element {
                DatabaseElement::Statement(s) => match s {
                    Statement::ConstantStatement(consts) => {
                        if consts.iter().any(|c| c.symbol == any_constant) {
                            Some(consts.iter().map(|c| c.clone()).collect())
                        } else {
                            None
                        }
                    }
                    _ => None,
                },
                _ => None,
            })
            .ok_or(Error::NotFoundError)?,
    })
}

#[tauri::command]
pub async fn get_constant_mmp_format(
    state: tauri::State<'_, Mutex<AppState>>,
    any_constant: &str,
) -> Result<String, Error> {
    let app_state = state.lock().await;
    let mm_data = app_state.metamath_data.as_ref().ok_or(Error::NoMmDbError)?;

    util::locate_after_to_mmp_file_format_of_statement_it_refers_to(
        LocateAfterRef::LocateAfterConst(any_constant),
        mm_data,
    )
}

#[tauri::command]
pub async fn write_constant_mmp_format_to_file(
    state: tauri::State<'_, Mutex<AppState>>,
    any_constant: &str,
    file_path: &str,
) -> Result<(), Error> {
    let app_state = state.lock().await;
    let mm_data = app_state.metamath_data.as_ref().ok_or(Error::NoMmDbError)?;

    let mmp_format = util::locate_after_to_mmp_file_format_of_statement_it_refers_to(
        LocateAfterRef::LocateAfterConst(any_constant),
        mm_data,
    )?;

    fs::write(file_path, mmp_format).map_err(|_| Error::FileWriteError)?;

    Ok(())
}
