//! LEAF Application - Tauri desktop application for LEAF

pub mod commands;
pub mod events;
pub mod state;

use state::AppState;
use tauri::Manager;
use tracing::info;
use tracing_subscriber::EnvFilter;

/// Initialize logging
fn init_logging() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();
}

/// Run the Tauri application
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    init_logging();
    info!("Starting LEAF application");

    tauri::Builder::default()
        .setup(|app| {
            let app_state = AppState::new()?;
            app.manage(app_state);
            info!("Application state initialized");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Project commands
            commands::projects::create_project,
            commands::projects::open_project,
            commands::projects::get_current_project,
            commands::projects::close_project,
            commands::projects::list_recent_projects,
            // Watcher commands
            commands::watcher::start_watcher,
            commands::watcher::stop_watcher,
            commands::watcher::is_watcher_running,
            commands::watcher::get_watch_paths,
            commands::watcher::add_watch_path,
            commands::watcher::remove_watch_path,
            // Event commands
            commands::events::list_events,
            commands::events::get_event,
            commands::events::list_pending_events,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
