use serde::{Deserialize, Serialize};
use simpl_driver_trait::ColumnMeta;
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportMapping {
    /// CSV/JSON 字段名 -> 表列名
    pub column_map: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportPreview {
    pub headers: Vec<String>,
    pub sample_rows: Vec<Vec<String>>,
    pub suggested_mapping: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportRequest {
    pub file_path: String,
    pub schema: Option<String>,
    pub table: String,
    pub mapping: ImportMapping,
    pub format: ImportFileFormat,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ImportFileFormat {
    Csv,
    Json,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportResult {
    pub rows_imported: usize,
    pub insert_statements: Vec<String>,
}

/// 预览 CSV/JSON 文件并建议列映射。
pub fn preview_import(
    path: &Path,
    format: ImportFileFormat,
    table_columns: &[ColumnMeta],
) -> Result<ImportPreview, String> {
    match format {
        ImportFileFormat::Csv => preview_csv(path, table_columns),
        ImportFileFormat::Json => preview_json(path, table_columns),
    }
}

/// 根据映射生成批量 INSERT 语句（由调用方执行）。
pub fn build_import_sql(req: &ImportRequest) -> Result<ImportResult, String> {
    let path = Path::new(&req.file_path);
    let preview = preview_import(path, req.format, &[])?;
    let rows = load_all_rows(path, req.format)?;
    let table = qualified_table(&req.schema, &req.table);
    let mut statements = Vec::new();

    for row in rows {
        let mut cols = Vec::new();
        let mut vals = Vec::new();
        for (src, dst) in &req.mapping.column_map {
            if let Some(idx) = preview.headers.iter().position(|h| h == src) {
                if let Some(raw) = row.get(idx) {
                    cols.push(dst.clone());
                    vals.push(format!("'{}'", raw.replace('\'', "''")));
                }
            }
        }
        if cols.is_empty() {
            continue;
        }
        statements.push(format!(
            "INSERT INTO {table} ({}) VALUES ({});",
            cols.join(", "),
            vals.join(", ")
        ));
    }

    Ok(ImportResult {
        rows_imported: statements.len(),
        insert_statements: statements,
    })
}

fn preview_csv(path: &Path, table_columns: &[ColumnMeta]) -> Result<ImportPreview, String> {
    let mut rdr = csv::Reader::from_path(path).map_err(|e| e.to_string())?;
    let headers: Vec<String> = rdr
        .headers()
        .map_err(|e| e.to_string())?
        .iter()
        .map(|s| s.to_string())
        .collect();
    let mut sample_rows = Vec::new();
    for (i, result) in rdr.records().enumerate() {
        if i >= 5 {
            break;
        }
        let record = result.map_err(|e| e.to_string())?;
        sample_rows.push(record.iter().map(|s| s.to_string()).collect());
    }
    Ok(ImportPreview {
        suggested_mapping: suggest_mapping(&headers, table_columns),
        headers,
        sample_rows,
    })
}

fn preview_json(path: &Path, table_columns: &[ColumnMeta]) -> Result<ImportPreview, String> {
    let raw = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    let values: Vec<serde_json::Value> = serde_json::from_str(&raw).map_err(|e| e.to_string())?;
    let headers: Vec<String> = values
        .first()
        .and_then(|v| v.as_object())
        .map(|obj| obj.keys().cloned().collect())
        .unwrap_or_default();
    let mut sample_rows = Vec::new();
    for v in values.iter().take(5) {
        if let Some(obj) = v.as_object() {
            let row: Vec<String> = headers
                .iter()
                .map(|h| obj.get(h).map(|x| x.to_string()).unwrap_or_default())
                .collect();
            sample_rows.push(row);
        }
    }
    Ok(ImportPreview {
        suggested_mapping: suggest_mapping(&headers, table_columns),
        headers,
        sample_rows,
    })
}

fn load_all_rows(path: &Path, format: ImportFileFormat) -> Result<Vec<Vec<String>>, String> {
    match format {
        ImportFileFormat::Csv => {
            let mut rdr = csv::Reader::from_path(path).map_err(|e| e.to_string())?;
            let mut rows = Vec::new();
            for result in rdr.records() {
                rows.push(
                    result
                        .map_err(|e| e.to_string())?
                        .iter()
                        .map(|s| s.to_string())
                        .collect(),
                );
            }
            Ok(rows)
        }
        ImportFileFormat::Json => {
            let raw = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
            let values: Vec<serde_json::Value> =
                serde_json::from_str(&raw).map_err(|e| e.to_string())?;
            let headers = values
                .first()
                .and_then(|v| v.as_object())
                .map(|obj| obj.keys().cloned().collect::<Vec<_>>())
                .unwrap_or_default();
            Ok(values
                .into_iter()
                .filter_map(|v| {
                    v.as_object().map(|obj| {
                        headers
                            .iter()
                            .map(|h| obj.get(h).map(|x| x.to_string()).unwrap_or_default())
                            .collect()
                    })
                })
                .collect())
        }
    }
}

fn suggest_mapping(headers: &[String], table_columns: &[ColumnMeta]) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for header in headers {
        if let Some(col) = table_columns
            .iter()
            .find(|c| c.name.eq_ignore_ascii_case(header))
        {
            map.insert(header.clone(), col.name.clone());
        }
    }
    map
}

fn qualified_table(schema: &Option<String>, table: &str) -> String {
    if let Some(s) = schema {
        format!("\"{s}\".\"{table}\"")
    } else {
        format!("\"{table}\"")
    }
}
