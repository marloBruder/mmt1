use tauri::async_runtime::Mutex;

use crate::{
    model::{Comment, HeaderPath, Statement::*},
    AppState, Error,
};

#[tauri::command]
pub async fn get_comment_local(
    state: tauri::State<'_, Mutex<AppState>>,
    header_path: HeaderPath,
    comment_num: u32,
) -> Result<Comment, Error> {
    let app_state = state.lock().await;
    let mm_data = app_state.metamath_data.as_ref().ok_or(Error::NoMmDbError)?;

    let header = header_path
        .resolve(&mm_data.database_header)
        .ok_or(Error::NotFoundError)?;

    let comment = header
        .content
        .iter()
        .filter_map(|c| {
            if let CommentStatement(comment) = c {
                Some(comment)
            } else {
                None
            }
        })
        .nth(comment_num as usize)
        .ok_or(Error::NotFoundError)?;

    Ok(comment.clone())
}
