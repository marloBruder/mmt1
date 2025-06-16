use tauri::async_runtime::Mutex;

use crate::{
    model::{Constant, ConstantsPageData, DatabaseElement, Statement},
    AppState, Error,
};

#[tauri::command]
pub async fn get_constants_local(
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<Vec<Constant>, Error> {
    let app_state = state.lock().await;
    let metamath_data = app_state.metamath_data.as_ref().ok_or(Error::NoMmDbError)?;

    Ok(metamath_data
        .database_header
        .constant_iter()
        .map(|c| c.clone())
        .collect())
}

#[tauri::command]
pub async fn get_constant_statement_local(
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

// pub fn set_constants_local(metamath_data: &mut MetamathData, symbols: &Vec<&str>) {
//     metamath_data.constants = Vec::new();
//     for symbol in symbols {
//         metamath_data.constants.push(Constant {
//             symbol: symbol.to_string(),
//         })
//     }
// }
