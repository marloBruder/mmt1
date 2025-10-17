use tauri::{
    async_runtime::Mutex, AppHandle, Emitter, Listener, Manager, WebviewUrl, WebviewWindowBuilder,
    Window,
};

use crate::{AppState, Error};

#[tauri::command]
pub async fn open_external_window(app: AppHandle) -> Result<(), Error> {
    WebviewWindowBuilder::new(
        &app,
        "external-window",
        WebviewUrl::App("externalWindow".into()),
    )
    .decorations(false)
    .closable(true)
    .build()
    .or(Err(Error::OpenExternalWindowError))?;

    Ok(())
}

#[tauri::command]
pub fn close_external_window(app: AppHandle) {
    app.webview_windows()
        .get("external-window")
        .map(|window| window.close());
}

#[tauri::command]
pub fn set_up_external_window_close_listener(window: Window, app: AppHandle) {
    window.listen("tauri://close-requested", move |_| {
        // Do nothing if emit fails
        app.emit("external-window-close", ()).ok();
    });
}

#[tauri::command]
pub async fn load_external_window_relevant_info(
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<u32, Error> {
    let app_state = state.lock().await;
    let metamath_data = app_state.metamath_data.as_ref().ok_or(Error::NoMmDbError)?;

    Ok(metamath_data.optimized_data.theorem_amount)
}
