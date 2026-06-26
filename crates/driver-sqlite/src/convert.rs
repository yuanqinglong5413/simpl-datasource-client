use simpl_driver_trait::{CellValue, ColumnMeta};
use sqlx::{Column, Row};

pub fn row_to_cells(row: &sqlx::sqlite::SqliteRow) -> Vec<CellValue> {
    (0..row.len()).map(|i| sqlite_cell_at(row, i)).collect()
}

fn sqlite_cell_at(row: &sqlx::sqlite::SqliteRow, index: usize) -> CellValue {
    if let Ok(v) = row.try_get::<Option<i64>, _>(index) {
        if let Some(n) = v {
            return CellValue::Int64(n);
        }
    }
    if let Ok(v) = row.try_get::<Option<f64>, _>(index) {
        if let Some(n) = v {
            return CellValue::Float64(n);
        }
    }
    if let Ok(v) = row.try_get::<Option<String>, _>(index) {
        return match v {
            Some(s) => CellValue::String(s),
            None => CellValue::Null,
        };
    }
    CellValue::Null
}

pub fn columns_from_row(row: &sqlx::sqlite::SqliteRow) -> Vec<ColumnMeta> {
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
