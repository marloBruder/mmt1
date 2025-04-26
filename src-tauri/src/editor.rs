use std::fs;

use tauri::async_runtime::Mutex;

use crate::{AppState, Error};

pub mod parse_mmp;
pub mod unify;

pub struct FolderRepresentation {
    file_names: Vec<String>,
    subfolder_names: Vec<String>,
}

#[tauri::command]
pub async fn open_folder(
    state: tauri::State<'_, Mutex<AppState>>,
    folder_path: &str,
) -> Result<FolderRepresentation, Error> {
    let mut app_state = state.lock().await;

    let folder = get_folder(folder_path).await?;

    app_state.open_folder = Some(folder_path.to_string());

    Ok(folder)
}

#[tauri::command]
pub async fn close_folder(state: tauri::State<'_, Mutex<AppState>>) -> Result<(), ()> {
    let mut app_state = state.lock().await;

    app_state.open_folder = None;

    Ok(())
}

#[tauri::command]
pub async fn get_subfolder(
    state: tauri::State<'_, Mutex<AppState>>,
    relative_path: &str,
) -> Result<FolderRepresentation, Error> {
    let app_state = state.lock().await;
    let mut open_folder = app_state
        .open_folder
        .as_ref()
        .ok_or(Error::NoOpenFolderError)?
        .clone();

    open_folder.push('/');
    open_folder.push_str(relative_path);

    get_folder(&open_folder).await
}

pub async fn get_folder(full_path: &str) -> Result<FolderRepresentation, Error> {
    let mut file_names = Vec::new();
    let mut subfolder_names = Vec::new();

    for entry in fs::read_dir(full_path).or(Err(Error::FolderReadError))? {
        let entry = entry.or(Err(Error::FolderReadError))?;

        if entry.path().is_file() {
            file_names.push(
                entry
                    .file_name()
                    .into_string()
                    .or(Err(Error::FolderReadError))?,
            );
        } else {
            subfolder_names.push(
                entry
                    .file_name()
                    .into_string()
                    .or(Err(Error::FolderReadError))?,
            );
        }
    }

    Ok(FolderRepresentation {
        file_names,
        subfolder_names,
    })
}

#[tauri::command]
pub async fn read_file(
    state: tauri::State<'_, Mutex<AppState>>,
    relative_path: &str,
) -> Result<String, Error> {
    let app_state = state.lock().await;

    let mut file_path = app_state
        .open_folder
        .as_ref()
        .ok_or(Error::NoOpenFolderError)?
        .clone();
    file_path.push('/');
    file_path.push_str(relative_path);

    Ok(fs::read_to_string(file_path).or(Err(Error::FileReadError))?)
}

#[tauri::command]
pub async fn save_file(
    state: tauri::State<'_, Mutex<AppState>>,
    relative_path: &str,
    content: &str,
) -> Result<(), Error> {
    let app_state = state.lock().await;

    let mut file_path = app_state
        .open_folder
        .as_ref()
        .ok_or(Error::NoOpenFolderError)?
        .clone();
    file_path.push('/');
    file_path.push_str(relative_path);

    fs::write(file_path, content).or(Err(Error::FileWriteError))?;

    Ok(())
}

impl serde::Serialize for FolderRepresentation {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        use serde::ser::SerializeStruct;

        let mut state = serializer.serialize_struct("Folder", 2)?;
        state.serialize_field("fileNames", &self.file_names)?;
        state.serialize_field("subfolderNames", &self.subfolder_names)?;
        state.end()
    }
}

// #[tauri::command]
// pub async fn add_in_progress_theorem(
//     state: tauri::State<'_, Mutex<AppState>>,
//     name: &str,
//     text: &str,
// ) -> Result<(), Error> {
//     let mut app_state = state.lock().await;
//     let db_state = app_state.db_state.as_mut().ok_or(Error::NoDatabaseError)?;

//     if !MetamathData::valid_label(name) {
//         return Err(Error::InvalidLabelError);
//     }

//     if db_state.metamath_data.label_exists(name) {
//         return Err(Error::LabelAlreadyExistsError);
//     }

//     add_in_progress_theorem_database(&mut db_state.db_conn, name, text).await?;

//     add_in_progress_theorem_local(&mut db_state.metamath_data, name, text);

//     Ok(())
// }

// #[tauri::command]
// pub async fn set_in_progress_theorem_name(
//     state: tauri::State<'_, Mutex<AppState>>,
//     old_name: &str,
//     new_name: &str,
// ) -> Result<(), Error> {
//     let mut app_state = state.lock().await;
//     let db_state = app_state.db_state.as_mut().ok_or(Error::NoDatabaseError)?;

//     if old_name != new_name {
//         if !MetamathData::valid_label(new_name) {
//             return Err(Error::InvalidLabelError);
//         }

//         if db_state.metamath_data.label_exists(new_name) {
//             return Err(Error::LabelAlreadyExistsError);
//         }

//         set_in_progress_theorem_name_database(&mut db_state.db_conn, old_name, new_name).await?;

//         set_in_progress_theorem_name_local(&mut db_state.metamath_data, old_name, new_name);
//     }

//     Ok(())
// }

// #[tauri::command]
// pub async fn set_in_progress_theorem(
//     state: tauri::State<'_, Mutex<AppState>>,
//     name: &str,
//     text: &str,
// ) -> Result<(), Error> {
//     let mut app_state = state.lock().await;
//     let db_state = app_state.db_state.as_mut().ok_or(Error::NoDatabaseError)?;

//     set_in_progress_theorem_text_database(&mut db_state.db_conn, name, text).await?;

//     set_in_progress_theorem_text_local(&mut db_state.metamath_data, name, text);

//     Ok(())
// }

// #[tauri::command]
// pub async fn delete_in_progress_theorem(
//     state: tauri::State<'_, Mutex<AppState>>,
//     name: &str,
// ) -> Result<(), Error> {
//     let mut app_state = state.lock().await;
//     let db_state = app_state.db_state.as_mut().ok_or(Error::NoDatabaseError)?;

//     delete_in_progress_theorem_database(&mut db_state.db_conn, name).await?;

//     delete_in_progress_theorem_local(&mut db_state.metamath_data, name);

//     Ok(())
// }
