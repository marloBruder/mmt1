use tauri::async_runtime::Mutex;

use crate::{
    model::{FloatingHypohesis, MetamathData},
    AppState, Error,
};

#[tauri::command]
pub async fn get_floating_hypotheses_local(
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<Vec<FloatingHypohesis>, Error> {
    let app_state = state.lock().await;
    let db_state = app_state.db_state.as_ref().ok_or(Error::NoDatabaseError)?;

    Ok(db_state.metamath_data.floating_hypotheses.clone())
}

pub fn set_floating_hypotheses_local(
    metamath_data: &mut MetamathData,
    floating_hypotheses: &Vec<FloatingHypohesis>,
) {
    metamath_data.floating_hypotheses = floating_hypotheses.clone();
}

// pub fn set_floating_hypotheses(
//     &mut self,
//     labels: &Vec<&str>,
//     typecodes: &Vec<&str>,
//     variables: &Vec<&str>,
// ) {
//     self.floating_hypotheses = Vec::new();
//     for index in 0..labels.len() {
//         self.floating_hypotheses.push(FloatingHypohesis {
//             label: labels[index].to_string(),
//             typecode: typecodes[index].to_string(),
//             variable: variables[index].to_string(),
//         })
//     }
// }
