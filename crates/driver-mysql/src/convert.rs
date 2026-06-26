use simpl_driver_trait::{CellValue, ColumnMeta};
use sqlx::{Column, Row};

pub fn row_to_cells(row: &sqlx::mysql::MySqlRow) -> Vec<CellValue> {
    (0..row.len()).map(|i| mysql_cell_at(row, i)).collect()
}

fn mysql_cell_at(row: &sqlx::mysql::MySqlRow, index: usize) -> CellValue {
    if let Ok(Some(b)) = row.try_get::<Option<bool>, _>(index) {
        return CellValue::Bool(b);
    }
    if let Ok(Some(n)) = row.try_get::<Option<i64>, _>(index) {
        return CellValue::Int64(n);
    }
    if let Ok(Some(n)) = row.try_get::<Option<f64>, _>(index) {
        return CellValue::Float64(n);
    }
    if let Ok(Some(json)) = row.try_get::<Option<serde_json::Value>, _>(index) {
        return CellValue::Json(json);
    }
    if let Ok(v) = row.try_get::<Option<String>, _>(index) {
        return match v {
            Some(s) => CellValue::String(s),
            None => CellValue::Null,
        };
    }
    CellValue::Null
}

pub fn columns_from_row(row: &sqlx::mysql::MySqlRow) -> Vec<ColumnMeta> {
    row.columns()
        .iter()
        .map(|col| ColumnMeta {
            name: col.name().to_string(),
            data_type: col.type_info().to_string(),
            nullable: true,
            is_primary_key: false,
        })
        .collect()
}
