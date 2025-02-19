use tauri::async_runtime::Mutex;

use crate::{model::MetamathData, AppState, Error};

// #[tauri::command]
// pub async fn get_in_progress_theorem_local(
//     state: tauri::State<'_, Mutex<AppState>>,
//     name: &str,
// ) -> Result<InProgressTheorem, Error> {
//     let app_state = state.lock().await;
//     let metamath_data = app_state.metamath_data.as_ref().ok_or(Error::NoMmDbError)?;

//     for in_progress_theorem in &metamath_data.in_progress_theorems {
//         if in_progress_theorem.name == name {
//             return Ok(in_progress_theorem.clone());
//         }
//     }

//     Err(Error::NotFoundError)
// }

// #[tauri::command]
// pub async fn get_in_progress_theorem_names_local(
//     state: tauri::State<'_, Mutex<AppState>>,
// ) -> Result<Vec<String>, Error> {
//     let app_state = state.lock().await;
//     let metamath_data = app_state.metamath_data.as_ref().ok_or(Error::NoMmDbError)?;

//     let mut names: Vec<String> = Vec::new();
//     for in_progress_theorem in &metamath_data.in_progress_theorems {
//         names.push(in_progress_theorem.name.clone());
//     }
//     return Ok(names);
// }

// pub fn add_in_progress_theorem_local(metamath_data: &mut MetamathData, name: &str, text: &str) {
//     metamath_data.in_progress_theorems.push(InProgressTheorem {
//         name: name.to_string(),
//         text: text.to_string(),
//     })
// }

// pub fn set_in_progress_theorem_name_local(
//     metamath_data: &mut MetamathData,
//     old_name: &str,
//     new_name: &str,
// ) {
//     for in_progress_theorem in &mut metamath_data.in_progress_theorems {
//         if in_progress_theorem.name == old_name {
//             in_progress_theorem.name = new_name.to_string();
//             return;
//         }
//     }
// }

// pub fn set_in_progress_theorem_text_local(
//     metamath_data: &mut MetamathData,
//     name: &str,
//     text: &str,
// ) {
//     for in_progress_theorem in &mut metamath_data.in_progress_theorems {
//         if in_progress_theorem.name == name {
//             in_progress_theorem.text = text.to_string();
//             return;
//         }
//     }
// }

// pub fn delete_in_progress_theorem_local(metamath_data: &mut MetamathData, name: &str) {
//     for (index, in_progress_theorem) in (&metamath_data.in_progress_theorems).iter().enumerate() {
//         if in_progress_theorem.name == name {
//             metamath_data.in_progress_theorems.remove(index);
//             return;
//         }
//     }
// }
