use crate::state::AppState;
use chrono::Utc;
use simpl_datasource_core::QueryHistoryEntry;
use simpl_driver_trait::{ExecuteResult, TablePageRequest};
use tauri::State;
use uuid::Uuid;

#[tauri::command]
pub async fn execute_sql(
    state: State<'_, AppState>,
    connection_id: Uuid,
    sql: String,
    connection_name: String,
) -> Result<Vec<ExecuteResult>, String> {
    let start = std::time::Instant::now();
    let results = state
        .query_engine
        .execute_sql(&connection_id, &sql)
        .await
        .map_err(|e| e.user_message)?;

    let row_count: u64 = results.iter().map(|r| r.rows_affected).sum();
    let entry = QueryHistoryEntry {
        id: Uuid::new_v4(),
        connection_id,
        connection_name,
        sql,
        executed_at: Utc::now(),
        duration_ms: start.elapsed().as_millis() as u64,
        row_count,
        favorite: false,
    };
    let _ = state.history.append(entry);

    Ok(results)
}

#[tauri::command]
pub async fn fetch_table_page(
    state: State<'_, AppState>,
    connection_id: Uuid,
    request: TablePageRequest,
) -> Result<simpl_driver_trait::RowPage, String> {
    state
        .query_engine
        .fetch_table_page(&connection_id, request)
        .await
        .map_err(|e| e.user_message)
}

#[tauri::command]
pub async fn list_query_history(
    state: State<'_, AppState>,
    connection_id: Option<Uuid>,
    limit: Option<usize>,
) -> Result<Vec<QueryHistoryEntry>, String> {
    Ok(state.history.list(connection_id, limit.unwrap_or(50)))
}
