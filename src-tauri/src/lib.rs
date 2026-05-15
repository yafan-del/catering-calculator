mod paste;
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

#[tauri::command]
async fn activate_and_paste(keywords: Vec<String>) -> Result<(), String> {
    if keywords.is_empty() {
        return Err("未提供目标窗口关键词".to_string());
    }
    paste::activate_and_paste(&keywords)
}

// ──────────────────────────── App 入口 ────────────────────────────

fn show_main_window(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
    }
}

#[cfg(target_os = "macos")]
fn handle_run_event(app: &tauri::AppHandle, event: tauri::RunEvent) {
    if let tauri::RunEvent::Reopen { .. } = event {
        show_main_window(app);
    }
}

#[cfg(not(target_os = "macos"))]
fn handle_run_event(_app: &tauri::AppHandle, _event: tauri::RunEvent) {}

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
            activate_and_paste,
        ])
        .setup(|app| {
            let show_item = MenuItem::with_id(app, "show", "显示窗口", true, None::<&str>)?;
            let quit_item = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show_item, &quit_item])?;
            let app_handle = app.handle().clone();

            TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .tooltip("餐饮计算器")
                .menu(&menu)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "show" => {
                        show_main_window(app);
                    }
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {}
                })
                .on_tray_icon_event(move |_tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        show_main_window(&app_handle);
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
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app, event| {
            handle_run_event(app, event);
        });
}
