use tauri::{AppHandle, Emitter, Listener, Manager, WebviewUrl, WebviewWindowBuilder, Window};

use crate::Error;

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
