use simpl_driver_trait::{CellValue, ColumnMeta};
use sqlx::{Column, Row};

pub fn row_to_cells(row: &sqlx::postgres::PgRow) -> Vec<CellValue> {
    (0..row.len()).map(|i| pg_cell_at(row, i)).collect()
}

fn pg_cell_at(row: &sqlx::postgres::PgRow, index: usize) -> CellValue {
    if row.try_get::<Option<i32>, _>(index).ok().flatten().is_none()
        && row.try_get::<Option<String>, _>(index).ok().flatten().is_none()
        && row.try_get::<Option<bool>, _>(index).ok().flatten().is_none()
        && row.try_get::<Option<i64>, _>(index).ok().flatten().is_none()
        && row.try_get::<Option<f64>, _>(index).ok().flatten().is_none()
        && row.try_get::<Option<serde_json::Value>, _>(index)
            .ok()
            .flatten()
            .is_none()
    {
        if let Ok(raw) = row.try_get::<Option<String>, _>(index) {
            if raw.is_none() {
                return CellValue::Null;
            }
        }
    }

    if let Ok(v) = row.try_get::<Option<bool>, _>(index) {
        if let Some(b) = v {
            return CellValue::Bool(b);
        }
    }
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
    if let Ok(v) = row.try_get::<Option<serde_json::Value>, _>(index) {
        if let Some(json) = v {
            return CellValue::Json(json);
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

pub fn columns_from_row(row: &sqlx::postgres::PgRow) -> Vec<ColumnMeta> {
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
