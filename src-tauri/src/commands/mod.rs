mod connection;
mod query;
mod schema;

pub use connection::*;
pub use query::*;
pub use schema::*;

use crate::state::AppState;
use serde::Serialize;
use tauri::State;

#[derive(Serialize)]
pub struct PingResponse {
    pub message: String,
}

#[derive(Serialize)]
pub struct AppInfo {
    pub name: String,
    pub name_en: String,
    pub version: String,
    pub uses_keyring: bool,
    pub data_dir: String,
}

#[tauri::command]
pub fn ping() -> PingResponse {
    PingResponse {
        message: "pong".into(),
    }
}

#[tauri::command]
pub fn get_app_info(state: State<'_, AppState>) -> AppInfo {
    AppInfo {
        name: "简源".into(),
        name_en: "SimplSource".into(),
        version: env!("CARGO_PKG_VERSION").into(),
        uses_keyring: state.connections.uses_keyring(),
        data_dir: state.connections.data_dir().display().to_string(),
    }
}
