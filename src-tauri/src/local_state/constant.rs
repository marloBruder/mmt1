use tauri::async_runtime::Mutex;

use crate::{
    metamath::Error,
    model::{Constant, MetamathData},
    AppState,
};

#[tauri::command]
pub async fn get_constants_local(
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<Vec<Constant>, Error> {
    let app_state = state.lock().await;
    let db_state = app_state.db_state.as_ref().ok_or(Error::NoDatabaseError)?;

    Ok(db_state.metamath_data.constants.clone())
}

pub fn set_constants_local(metamath_data: &mut MetamathData, symbols: &Vec<&str>) {
    metamath_data.constants = Vec::new();
    for symbol in symbols {
        metamath_data.constants.push(Constant {
            symbol: symbol.to_string(),
        })
    }
}
