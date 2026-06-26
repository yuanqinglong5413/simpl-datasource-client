use crate::state::AppState;
use simpl_datasource_core::{ExportFormat, ImportFileFormat, ImportRequest};
use simpl_driver_trait::RowPage;
use std::path::PathBuf;
use tauri::State;
use uuid::Uuid;

#[tauri::command]
pub async fn export_data(
    state: State<'_, AppState>,
    page: RowPage,
    format: ExportFormat,
    file_path: String,
) -> Result<simpl_datasource_core::ExportResult, String> {
    state
        .connections
        .export_page(&page, format, PathBuf::from(file_path).as_path())
}

#[tauri::command]
pub async fn import_preview(
    state: State<'_, AppState>,
    connection_id: Uuid,
    file_path: String,
    format: ImportFileFormat,
    table: String,
    schema: Option<String>,
) -> Result<simpl_datasource_core::ImportPreview, String> {
    state
        .connections
        .import_preview(
            &connection_id,
            PathBuf::from(file_path).as_path(),
            format,
            &table,
            schema,
        )
        .await
        .map_err(|e| e.user_message)
}

#[tauri::command]
pub async fn import_execute(
    state: State<'_, AppState>,
    connection_id: Uuid,
    request: ImportRequest,
) -> Result<simpl_datasource_core::ImportResult, String> {
    state
        .connections
        .execute_import(&connection_id, request)
        .await
        .map_err(|e| e.user_message)
}

#[tauri::command]
pub async fn pick_save_file(default_name: Option<String>) -> Result<Option<String>, String> {
    let mut builder = rfd::FileDialog::new();
    if let Some(name) = default_name {
        builder = builder.set_file_name(name);
    }
    Ok(builder.save_file().map(|p| p.display().to_string()))
}

#[tauri::command]
pub async fn pick_open_file() -> Result<Option<String>, String> {
    Ok(rfd::FileDialog::new()
        .pick_file()
        .map(|p| p.display().to_string()))
}
