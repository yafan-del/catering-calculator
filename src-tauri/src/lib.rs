mod snap;

use snap::{SnapConfig, SnapManager, SnapStatus};
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager, State, WindowEvent,
};

// ──────────────────────────── Tauri Commands ────────────────────────────

#[tauri::command]
async fn start_snap(
    app: tauri::AppHandle,
    config: SnapConfig,
    state: State<'_, SnapManager>,
) -> Result<(), String> {
    state.start(app, config);
    Ok(())
}

#[tauri::command]
async fn stop_snap(state: State<'_, SnapManager>) -> Result<(), String> {
    state.stop();
    Ok(())
}

#[tauri::command]
async fn get_snap_status(state: State<'_, SnapManager>) -> Result<SnapStatus, String> {
    Ok(state.status())
}

// ──────────────────────────── App 入口 ────────────────────────────

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_process::init())
        .manage(SnapManager::new())
        .invoke_handler(tauri::generate_handler![
            start_snap,
            stop_snap,
            get_snap_status,
        ])
        .setup(|app| {
            let show_item = MenuItem::with_id(app, "show", "显示窗口", true, None::<&str>)?;
            let quit_item = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show_item, &quit_item])?;

            TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .tooltip("餐饮计算器")
                .menu(&menu)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "show" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                })
                .build(app)?;

            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
