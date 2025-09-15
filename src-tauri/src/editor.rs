use std::{
    collections::HashMap,
    fs::{self, OpenOptions},
    io::{Read, Seek, SeekFrom, Write},
    path::{Path, PathBuf},
};

use tauri::async_runtime::Mutex;

use crate::{model::FolderData, AppState, Error};

pub mod external_window;
pub mod format;
pub mod on_edit;
pub mod parse_mmp;
pub mod renumber;
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

    let path = Path::new(folder_path);

    let folder = get_folder(path).await?;

    app_state.open_folder_data = Some(FolderData {
        path: PathBuf::from(path),
        file_handles: HashMap::new(),
    });

    Ok(folder)
}

#[tauri::command]
pub async fn close_folder(state: tauri::State<'_, Mutex<AppState>>) -> Result<(), ()> {
    let mut app_state = state.lock().await;

    app_state.open_folder_data = None;

    Ok(())
}

#[tauri::command]
pub async fn get_subfolder(
    state: tauri::State<'_, Mutex<AppState>>,
    relative_path: &str,
) -> Result<FolderRepresentation, Error> {
    let app_state = state.lock().await;
    let open_folder_data = app_state
        .open_folder_data
        .as_ref()
        .ok_or(Error::NoOpenFolderError)?;

    let mut path = open_folder_data.path.clone();
    path.push(relative_path);

    get_folder(Path::new(&path)).await
}

pub async fn get_folder(full_path: &Path) -> Result<FolderRepresentation, Error> {
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
pub async fn open_file(
    state: tauri::State<'_, Mutex<AppState>>,
    relative_path: &str,
) -> Result<String, Error> {
    let mut app_state = state.lock().await;
    let open_folder_data = app_state
        .open_folder_data
        .as_mut()
        .ok_or(Error::NoOpenFolderError)?;

    let mut path = open_folder_data.path.clone();
    path.push(relative_path);

    let mut file_handle = OpenOptions::new()
        .read(true)
        .write(true)
        .open(&path)
        .map_err(|_| Error::FileOpenError)?;

    let mut file_content = String::new();
    file_handle
        .read_to_string(&mut file_content)
        .map_err(|_| Error::FileReadError)?;

    open_folder_data
        .file_handles
        .insert(relative_path.to_string(), file_handle);

    Ok(file_content)
}

#[tauri::command]
pub async fn save_file(
    state: tauri::State<'_, Mutex<AppState>>,
    relative_path: &str,
    content: &str,
) -> Result<(), Error> {
    let mut app_state = state.lock().await;
    let open_folder_data = app_state
        .open_folder_data
        .as_mut()
        .ok_or(Error::NoOpenFolderError)?;

    if let Some(file_handle) = open_folder_data.file_handles.get_mut(relative_path) {
        file_handle
            .seek(SeekFrom::Start(0))
            .map_err(|_| Error::FileWriteError)?;
        file_handle.set_len(0).map_err(|_| Error::FileWriteError)?;
        file_handle
            .write_all(content.as_bytes())
            .map_err(|_| Error::FileWriteError)?;
        file_handle.flush().map_err(|_| Error::FileWriteError)?;
    }

    Ok(())
}

#[tauri::command]
pub async fn close_file(
    state: tauri::State<'_, Mutex<AppState>>,
    relative_path: &str,
) -> Result<(), Error> {
    let mut app_state = state.lock().await;
    let open_folder_data = app_state
        .open_folder_data
        .as_mut()
        .ok_or(Error::NoOpenFolderError)?;

    open_folder_data.file_handles.remove(relative_path);

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
