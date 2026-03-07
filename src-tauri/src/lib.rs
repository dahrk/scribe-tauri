mod audio;
mod asr;
mod commands;
mod db;
mod recorder;
mod settings;
mod summarization;

use parking_lot::Mutex;
use rusqlite::Connection;
use settings::Settings;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::{
    image::Image,
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager,
};
use tokio::sync::mpsc;

pub struct AppState {
    pub db: Arc<Mutex<Connection>>,
    pub cmd_tx: Arc<Mutex<mpsc::Sender<recorder::RecorderCommand>>>,
    pub settings: Arc<Mutex<Settings>>,
    pub settings_path: Arc<Mutex<Option<PathBuf>>>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::init();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            // ── Paths ────────────────────────────────────────────────────────
            let data_dir = app
                .path()
                .app_data_dir()
                .expect("app data dir unavailable");
            std::fs::create_dir_all(&data_dir)?;

            let db_path = data_dir.join("recordings.db");
            let settings_path = data_dir.join("settings.json");

            // ── Settings ─────────────────────────────────────────────────────
            let loaded_settings = Settings::load(&settings_path).unwrap_or_default();

            // ── Database ─────────────────────────────────────────────────────
            let conn = db::open(&db_path).expect("failed to open SQLite DB");
            let db = Arc::new(Mutex::new(conn));

            // ── Recorder channel ──────────────────────────────────────────────
            let (cmd_tx, cmd_rx) = mpsc::channel::<recorder::RecorderCommand>(32);
            let cmd_tx = Arc::new(Mutex::new(cmd_tx));

            // ── App state ────────────────────────────────────────────────────
            let settings = Arc::new(Mutex::new(loaded_settings.clone()));
            app.manage(AppState {
                db: db.clone(),
                cmd_tx: cmd_tx.clone(),
                settings: settings.clone(),
                settings_path: Arc::new(Mutex::new(Some(settings_path))),
            });

            // ── Recorder background task ──────────────────────────────────────
            let app_handle = app.handle().clone();
            let db_clone = db.clone();
            tokio::spawn(async move {
                recorder::run_recorder(app_handle, db_clone, loaded_settings, cmd_rx).await;
            });

            // ── System tray ──────────────────────────────────────────────────
            setup_tray(app)?;

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::list_recordings,
            commands::get_recording,
            commands::update_recording,
            commands::delete_recording,
            commands::start_recording,
            commands::stop_recording,
            commands::get_settings,
            commands::save_settings,
            commands::retry_summary,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn setup_tray(app: &tauri::App) -> tauri::Result<()> {
    let open_item = MenuItem::with_id(app, "open", "Open Scribe", true, None::<&str>)?;
    let toggle_item =
        MenuItem::with_id(app, "toggle_auto", "Auto-record: On", true, None::<&str>)?;
    let start_item =
        MenuItem::with_id(app, "start_rec", "Start Recording", true, None::<&str>)?;
    let stop_item =
        MenuItem::with_id(app, "stop_rec", "Stop Recording", false, None::<&str>)?;
    let sep = PredefinedMenuItem::separator(app)?;
    let quit_item = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;

    let menu = Menu::with_items(
        app,
        &[
            &open_item,
            &sep,
            &toggle_item,
            &start_item,
            &stop_item,
            &sep,
            &quit_item,
        ],
    )?;

    TrayIconBuilder::new()
        .icon(app.default_window_icon().cloned().unwrap_or_else(|| {
            Image::from_bytes(include_bytes!("../icons/32x32.png")).expect("tray icon")
        }))
        .menu(&menu)
        .on_menu_event(|app, event| {
            let state = app.state::<AppState>();
            match event.id().as_ref() {
                "open" => {
                    if let Some(w) = app.get_webview_window("main") {
                        let _ = w.show();
                        let _ = w.set_focus();
                    }
                }
                "toggle_auto" => {
                    let mut s = state.settings.lock();
                    s.auto_record = !s.auto_record;
                    let _ = app.emit("settings-changed", s.clone());
                }
                "start_rec" => {
                    let tx = state.cmd_tx.lock();
                    let tx_clone = tx.clone();
                    tokio::spawn(async move {
                        let _ = tx_clone.send(recorder::RecorderCommand::StartManual).await;
                    });
                }
                "stop_rec" => {
                    let tx = state.cmd_tx.lock();
                    let tx_clone = tx.clone();
                    tokio::spawn(async move {
                        let _ = tx_clone.send(recorder::RecorderCommand::StopManual).await;
                    });
                }
                "quit" => {
                    app.exit(0);
                }
                _ => {}
            }
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let app = tray.app_handle();
                if let Some(w) = app.get_webview_window("main") {
                    let _ = w.show();
                    let _ = w.set_focus();
                }
            }
        })
        .build(app)?;

    Ok(())
}
