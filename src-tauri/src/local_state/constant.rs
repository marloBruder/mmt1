use tauri::async_runtime::Mutex;

use crate::{
    model::{Constant, MetamathData},
    AppState, Error,
};

#[tauri::command]
pub async fn get_constants_local(
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<Vec<Constant>, Error> {
    let app_state = state.lock().await;
    let metamath_data = app_state.metamath_data.as_ref().ok_or(Error::NoMmDbError)?;

    Ok(metamath_data.constants.clone())
}

pub fn set_constants_local(metamath_data: &mut MetamathData, symbols: &Vec<&str>) {
    metamath_data.constants = Vec::new();
    for symbol in symbols {
        metamath_data.constants.push(Constant {
            symbol: symbol.to_string(),
        })
    }
}
