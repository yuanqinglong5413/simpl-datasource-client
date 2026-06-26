use crate::state::AppState;
use simpl_driver_trait::SchemaMeta;
use tauri::State;
use uuid::Uuid;

#[tauri::command]
pub async fn get_schema(
    state: State<'_, AppState>,
    connection_id: Uuid,
    force_refresh: Option<bool>,
) -> Result<SchemaMeta, String> {
    state
        .query_engine
        .get_schema(&connection_id, force_refresh.unwrap_or(false))
        .await
        .map_err(|e| e.user_message)
}
