//! Thin native host for the revisioned Rust desktop bridge.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use ecad_bridge::{BridgeError, DesktopBridge, Request, Response};
use std::sync::Mutex;
use tauri::{Emitter, Manager};
use tauri_plugin_dialog::DialogExt;
struct Host(Mutex<DesktopBridge>);
#[tauri::command]
async fn bridge(request: Request, state: tauri::State<'_, Host>) -> Result<Response, BridgeError> {
    if matches!(&request, Request::Snapshot {}) {
        println!("ECAD desktop webview connected to the document engine");
    }
    state
        .0
        .lock()
        .map_err(|_| error("The document controller is unavailable"))?
        .handle(request)
}
fn error(message: impl ToString) -> BridgeError {
    BridgeError {
        code: "host".into(),
        message: message.to_string(),
    }
}
#[tauri::command]
async fn choose_file(app: tauri::AppHandle, save: bool) -> Result<Option<Response>, BridgeError> {
    if !save && app.state::<Host>().0.lock().map_err(error)?.is_dirty() {
        return Err(error("Save your changes before opening another project"));
    }
    let observed = app.state::<Host>().0.lock().map_err(error)?.revision_key();
    let picker = app.clone();
    let path = tauri::async_runtime::spawn_blocking(move || {
        let dialog = picker.dialog().file().add_filter("ECAD project", &["json"]);
        if save {
            dialog
                .set_file_name("project.ecad.json")
                .blocking_save_file()
        } else {
            dialog.blocking_pick_file()
        }
    })
    .await
    .map_err(error)?;
    let Some(path) = path else { return Ok(None) };
    let path = path.into_path().map_err(error)?;
    let state = app.state::<Host>();
    let mut bridge = state.0.lock().map_err(error)?;
    if bridge.revision_key() != observed {
        return Err(error(
            "The document changed while choosing a file. Try again",
        ));
    }
    if save {
        bridge.save_as(&path).map(Some)
    } else {
        bridge.open(&path).map(Some)
    }
}
fn main() {
    let controller = DesktopBridge::demo().unwrap_or_else(|e| {
        eprintln!("{e}");
        std::process::exit(1)
    });
    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(Host(Mutex::new(controller)))
        .setup(|app| {
            let path = app.path().app_config_dir()?.join("company-profiles.json");
            app.state::<Host>()
                .0
                .lock()
                .map_err(|_| std::io::Error::other("The document controller is unavailable"))?
                .configure_profile_store(path);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![bridge, choose_file])
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                let dirty = window
                    .state::<Host>()
                    .0
                    .lock()
                    .map(|b| b.is_dirty())
                    .unwrap_or(true);
                if dirty {
                    api.prevent_close();
                    let _ = window.emit(
                        "ecad:close-blocked",
                        "Save your changes before closing ECAD",
                    );
                }
            }
        });
    // Keep native text accelerators from bypassing the shared web/core document router.
    #[cfg(target_os = "macos")]
    let builder = builder.menu(|app| {
        use tauri::menu::{Menu, PredefinedMenuItem, Submenu};
        Menu::with_items(
            app,
            &[&Submenu::with_items(
                app,
                "ECAD",
                true,
                &[
                    &PredefinedMenuItem::hide(app, None)?,
                    &PredefinedMenuItem::quit(app, None)?,
                ],
            )?],
        )
    });
    let app = builder
        .build(tauri::generate_context!())
        .unwrap_or_else(|e| {
            eprintln!("{e}");
            std::process::exit(1)
        });
    app.run(|app, event| {
        if let tauri::RunEvent::ExitRequested { api, .. } = event
            && app
                .state::<Host>()
                .0
                .lock()
                .map(|bridge| bridge.is_dirty())
                .unwrap_or(true)
        {
            api.prevent_exit();
            let _ = app.emit(
                "ecad:close-blocked",
                "Save your changes before quitting ECAD",
            );
        }
    });
}
