use crate::state::AppState;
use simpl_datasource_core::{SaveConnectionRequest, TestConnectionRequest};
use simpl_driver_trait::ConnectionConfig;
use tauri::State;
use uuid::Uuid;

#[tauri::command]
pub async fn list_connections(
    state: State<'_, AppState>,
) -> Result<Vec<simpl_datasource_core::ConnectionRecord>, String> {
    state
        .connections
        .list_connections()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn save_connection(
    state: State<'_, AppState>,
    request: SaveConnectionRequest,
) -> Result<ConnectionConfig, String> {
    let _guard = state.io_lock.lock().await;
    state
        .connections
        .save_connection(request)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_connection(state: State<'_, AppState>, id: Uuid) -> Result<(), String> {
    let _guard = state.io_lock.lock().await;
    state
        .connections
        .delete_connection(&id)
        .map_err(|e| e.to_string())?;
    state.connections.disconnect(&id).await;
    state.schema_cache.invalidate(&id).await;
    Ok(())
}

#[tauri::command]
pub async fn test_connection(
    state: State<'_, AppState>,
    request: TestConnectionRequest,
) -> Result<(), String> {
    state
        .connections
        .test_connection(request)
        .await
        .map_err(|e| e.user_message)
}

#[tauri::command]
pub async fn connect_database(
    state: State<'_, AppState>,
    id: Uuid,
) -> Result<ConnectionConfig, String> {
    // 仅建立驱动连接；Schema 由前端 SchemaTree 异步拉取，避免连接按钮长时间无响应
    state
        .connections
        .connect(&id)
        .await
        .map_err(|e| e.user_message)
}

#[tauri::command]
pub async fn disconnect_database(state: State<'_, AppState>, id: Uuid) -> Result<(), String> {
    state.connections.disconnect(&id).await;
    state.schema_cache.invalidate(&id).await;
    Ok(())
}
