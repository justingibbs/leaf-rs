//! LEAF Application - Tauri desktop application for LEAF

pub mod commands;
pub mod events;
pub mod execution;
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
        .plugin(tauri_plugin_dialog::init())
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
            // Stack commands
            commands::stacks::list_stacks,
            commands::stacks::get_stack,
            commands::stacks::create_stack,
            commands::stacks::update_stack,
            commands::stacks::delete_stack,
            commands::stacks::enable_stack,
            commands::stacks::disable_stack,
            // Card commands
            commands::cards::list_cards,
            commands::cards::get_card,
            commands::cards::create_card,
            commands::cards::update_card,
            commands::cards::delete_card,
            commands::cards::enable_card,
            commands::cards::disable_card,
            commands::cards::trigger_card,
            // Execution commands
            commands::executions::list_executions,
            commands::executions::get_execution,
            commands::executions::list_executions_for_event,
            // Session commands
            commands::sessions::list_sessions,
            commands::sessions::list_active_sessions,
            commands::sessions::create_session,
            commands::sessions::get_session,
            commands::sessions::update_session_title,
            commands::sessions::archive_session,
            commands::sessions::unarchive_session,
            commands::sessions::delete_session,
            // Chat commands
            commands::chat::get_messages,
            commands::chat::send_message,
            commands::chat::send_message_sync,
            // MCP commands
            commands::mcp::list_mcp_servers,
            commands::mcp::add_mcp_server,
            commands::mcp::remove_mcp_server,
            commands::mcp::enable_mcp_server,
            commands::mcp::disable_mcp_server,
            commands::mcp::list_mcp_tools,
            commands::mcp::test_mcp_server,
            // Artifact commands
            commands::artifacts::list_artifacts,
            commands::artifacts::get_artifact,
            commands::artifacts::scan_artifacts,
            // Settings commands
            commands::settings::get_app_config,
            commands::settings::update_app_config,
            commands::settings::get_project_settings,
            commands::settings::update_project_settings,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
