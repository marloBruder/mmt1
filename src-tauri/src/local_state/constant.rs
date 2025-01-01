use tauri::async_runtime::Mutex;

use crate::{
    model::{Constant, MetamathData},
    AppState,
};

#[tauri::command]
pub async fn get_constants_local(
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<Vec<Constant>, ()> {
    let app_state = state.lock().await;

    if let Some(ref mm_data) = app_state.metamath_data {
        return Ok(mm_data.constants.clone());
    }

    Err(())
}

pub fn set_constants_local(metamath_data: &mut MetamathData, symbols: &Vec<&str>) {
    metamath_data.constants = Vec::new();
    for symbol in symbols {
        metamath_data.constants.push(Constant {
            symbol: symbol.to_string(),
        })
    }
}
