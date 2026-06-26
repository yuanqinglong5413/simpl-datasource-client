mod commands;
mod state;

use state::AppState;
use tauri::Manager;
use tracing_subscriber::EnvFilter;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            let data_dir = simpl_datasource_core::default_data_dir();
            let state = AppState::new(data_dir);
            app.manage(state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::ping,
            commands::get_app_info,
            commands::list_connections,
            commands::save_connection,
            commands::delete_connection,
            commands::test_connection,
            commands::connect_database,
            commands::disconnect_database,
            commands::get_schema,
            commands::execute_sql,
            commands::fetch_table_page,
            commands::list_query_history,
        ])
        .run(tauri::generate_context!())
        .expect("failed to run 简源 application");
}
