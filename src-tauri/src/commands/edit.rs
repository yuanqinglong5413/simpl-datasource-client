use crate::state::AppState;
use simpl_datasource_core::CellChangeRequest;
use tauri::State;
use uuid::Uuid;

#[tauri::command]
pub async fn preview_cell_change(
    state: State<'_, AppState>,
    connection_id: Uuid,
    change: CellChangeRequest,
) -> Result<simpl_datasource_core::DmlPreview, String> {
    state
        .connections
        .preview_cell_change(&connection_id, change)
        .await
        .map_err(|e| e.user_message)
}

#[tauri::command]
pub async fn queue_cell_change(
    state: State<'_, AppState>,
    connection_id: Uuid,
    change: CellChangeRequest,
) -> Result<simpl_datasource_core::DmlPreview, String> {
    state
        .connections
        .queue_cell_change(&connection_id, change)
        .await
        .map_err(|e| e.user_message)
}

#[tauri::command]
pub async fn commit_transaction(
    state: State<'_, AppState>,
    connection_id: Uuid,
) -> Result<u64, String> {
    state
        .connections
        .commit_transaction(&connection_id)
        .await
        .map_err(|e| e.user_message)
}

#[tauri::command]
pub async fn rollback_transaction(
    state: State<'_, AppState>,
    connection_id: Uuid,
) -> Result<(), String> {
    state
        .connections
        .rollback_transaction(&connection_id)
        .await
        .map_err(|e| e.user_message)
}

#[tauri::command]
pub async fn get_transaction_status(
    state: State<'_, AppState>,
    connection_id: Uuid,
) -> Result<simpl_datasource_core::TransactionStatus, String> {
    Ok(state.connections.transaction_status(&connection_id).await)
}
