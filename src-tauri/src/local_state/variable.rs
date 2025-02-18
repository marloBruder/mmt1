use tauri::async_runtime::Mutex;

use crate::{
    model::{MetamathData, Variable},
    AppState, Error,
};

#[tauri::command]
pub async fn get_variables_local(
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<Vec<Variable>, Error> {
    let app_state = state.lock().await;
    let metamath_data = app_state.metamath_data.as_ref().ok_or(Error::NoMmDbError)?;

    Ok(metamath_data.variables.clone())
}

pub fn set_variables_local(metamath_data: &mut MetamathData, symbols: &Vec<&str>) {
    metamath_data.variables = Vec::new();
    for symbol in symbols {
        metamath_data.variables.push(Variable {
            symbol: symbol.to_string(),
        })
    }
}
