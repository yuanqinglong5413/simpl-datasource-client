use serde::{Deserialize, Serialize};
use simpl_driver_trait::{CellValue, ColumnMeta, RowPage};
use std::fs::File;
use std::path::Path;

/// 导出格式。
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ExportFormat {
    Csv,
    Json,
    Jsonl,
    Sql,
    Xlsx,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportRequest {
    pub format: ExportFormat,
    pub file_path: String,
    pub table_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportResult {
    pub rows_written: usize,
    pub file_path: String,
}

/// 将分页数据导出到文件。
pub fn export_row_page(
    page: &RowPage,
    format: ExportFormat,
    path: &Path,
) -> Result<ExportResult, String> {
    match format {
        ExportFormat::Csv => export_csv(page, path),
        ExportFormat::Json => export_json(page, path, false),
        ExportFormat::Jsonl => export_json(page, path, true),
        ExportFormat::Sql => export_sql_inserts(page, path),
        ExportFormat::Xlsx => export_xlsx(page, path),
    }
}

fn export_csv(page: &RowPage, path: &Path) -> Result<ExportResult, String> {
    let file = File::create(path).map_err(|e| e.to_string())?;
    let mut wtr = csv::Writer::from_writer(file);
    let headers: Vec<String> = page.columns.iter().map(|c| c.name.clone()).collect();
    wtr.write_record(&headers).map_err(|e| e.to_string())?;
    for row in &page.rows {
        let record: Vec<String> = row.iter().map(cell_to_export_string).collect();
        wtr.write_record(&record).map_err(|e| e.to_string())?;
    }
    wtr.flush().map_err(|e| e.to_string())?;
    Ok(ExportResult {
        rows_written: page.rows.len(),
        file_path: path.display().to_string(),
    })
}

fn export_json(page: &RowPage, path: &Path, jsonl: bool) -> Result<ExportResult, String> {
    let mut rows_json = Vec::new();
    for row in &page.rows {
        rows_json.push(row_to_object(&page.columns, row));
    }
    let content = if jsonl {
        rows_json
            .iter()
            .map(|r| serde_json::to_string(r).unwrap_or_default())
            .collect::<Vec<_>>()
            .join("\n")
    } else {
        serde_json::to_string_pretty(&rows_json).map_err(|e| e.to_string())?
    };
    std::fs::write(path, content).map_err(|e| e.to_string())?;
    Ok(ExportResult {
        rows_written: page.rows.len(),
        file_path: path.display().to_string(),
    })
}

fn export_sql_inserts(page: &RowPage, path: &Path) -> Result<ExportResult, String> {
    let table = "exported_data";
    let cols: Vec<String> = page.columns.iter().map(|c| c.name.clone()).collect();
    let col_list = cols.join(", ");
    let mut lines = Vec::new();
    for row in &page.rows {
        let values: Vec<String> = row.iter().map(sql_value).collect();
        lines.push(format!(
            "INSERT INTO {table} ({col_list}) VALUES ({});",
            values.join(", ")
        ));
    }
    std::fs::write(path, lines.join("\n")).map_err(|e| e.to_string())?;
    Ok(ExportResult {
        rows_written: page.rows.len(),
        file_path: path.display().to_string(),
    })
}

fn export_xlsx(page: &RowPage, path: &Path) -> Result<ExportResult, String> {
    use rust_xlsxwriter::{Format, Workbook};
    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();
    let header_format = Format::new().set_bold();
    for (col, meta) in page.columns.iter().enumerate() {
        worksheet
            .write_string_with_format(0, col as u16, &meta.name, &header_format)
            .map_err(|e| e.to_string())?;
    }
    for (row_idx, row) in page.rows.iter().enumerate() {
        for (col_idx, cell) in row.iter().enumerate() {
            write_cell(worksheet, (row_idx + 1) as u32, col_idx as u16, cell)?;
        }
    }
    workbook.save(path).map_err(|e| e.to_string())?;
    Ok(ExportResult {
        rows_written: page.rows.len(),
        file_path: path.display().to_string(),
    })
}

fn write_cell(
    worksheet: &mut rust_xlsxwriter::Worksheet,
    row: u32,
    col: u16,
    cell: &CellValue,
) -> Result<(), String> {
    match cell {
        CellValue::Null => {
            worksheet
                .write_blank(row, col, &rust_xlsxwriter::Format::new())
                .map_err(|e| e.to_string())?;
            Ok(())
        }
        CellValue::Bool(b) => {
            worksheet
                .write_boolean(row, col, *b)
                .map_err(|e| e.to_string())?;
            Ok(())
        }
        CellValue::Int64(n) => {
            worksheet
                .write_number(row, col, *n as f64)
                .map_err(|e| e.to_string())?;
            Ok(())
        }
        CellValue::Float64(n) => {
            worksheet
                .write_number(row, col, *n)
                .map_err(|e| e.to_string())?;
            Ok(())
        }
        CellValue::String(s) | CellValue::DateTime(s) | CellValue::Bytes(s) => {
            worksheet
                .write_string(row, col, s)
                .map_err(|e| e.to_string())?;
            Ok(())
        }
        CellValue::Json(v) => {
            worksheet
                .write_string(row, col, v.to_string())
                .map_err(|e| e.to_string())?;
            Ok(())
        }
    }
}

fn row_to_object(
    columns: &[ColumnMeta],
    row: &[CellValue],
) -> serde_json::Map<String, serde_json::Value> {
    let mut map = serde_json::Map::new();
    for (idx, col) in columns.iter().enumerate() {
        let val = row.get(idx).cloned().unwrap_or(CellValue::Null);
        map.insert(col.name.clone(), cell_to_json(&val));
    }
    map
}

fn cell_to_json(cell: &CellValue) -> serde_json::Value {
    match cell {
        CellValue::Null => serde_json::Value::Null,
        CellValue::Bool(b) => serde_json::Value::Bool(*b),
        CellValue::Int64(n) => serde_json::Value::from(*n),
        CellValue::Float64(n) => serde_json::Value::from(*n),
        CellValue::String(s) | CellValue::DateTime(s) | CellValue::Bytes(s) => {
            serde_json::Value::String(s.clone())
        }
        CellValue::Json(v) => v.clone(),
    }
}

fn cell_to_export_string(cell: &CellValue) -> String {
    match cell {
        CellValue::Null => String::new(),
        CellValue::Bool(b) => b.to_string(),
        CellValue::Int64(n) => n.to_string(),
        CellValue::Float64(n) => n.to_string(),
        CellValue::String(s) | CellValue::DateTime(s) | CellValue::Bytes(s) => s.clone(),
        CellValue::Json(v) => v.to_string(),
    }
}

fn sql_value(cell: &CellValue) -> String {
    match cell {
        CellValue::Null => "NULL".into(),
        CellValue::Bool(b) => {
            if *b {
                "TRUE".into()
            } else {
                "FALSE".into()
            }
        }
        CellValue::Int64(n) => n.to_string(),
        CellValue::Float64(n) => n.to_string(),
        CellValue::String(s) | CellValue::DateTime(s) | CellValue::Bytes(s) => {
            format!("'{}'", s.replace('\'', "''"))
        }
        CellValue::Json(v) => format!("'{}'", v.to_string().replace('\'', "''")),
    }
}
