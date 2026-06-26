use serde::{Deserialize, Serialize};
use simpl_driver_trait::{CellValue, Dialect};
use std::collections::HashMap;

/// 单元格变更请求（内联编辑）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CellChangeRequest {
    pub schema: Option<String>,
    pub table: String,
    pub primary_key: HashMap<String, CellValue>,
    pub column: String,
    pub old_value: CellValue,
    pub new_value: CellValue,
}

/// DML 预览结果。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DmlPreview {
    pub sql: String,
    pub operation: String,
}

/// 根据方言生成 UPDATE 语句（基于主键定位行）。
pub fn generate_update(dialect: Dialect, change: &CellChangeRequest) -> Result<DmlPreview, String> {
    if change.primary_key.is_empty() {
        return Err("该表没有主键，无法安全内联编辑".into());
    }
    if matches!(change.new_value, CellValue::Null) && matches!(change.old_value, CellValue::Null) {
        return Err("值未变化".into());
    }

    let table_ref = qualified_table(dialect, change.schema.as_deref(), &change.table);
    let set_clause = format!(
        "{} = {}",
        quote_ident(dialect, &change.column),
        sql_literal(&change.new_value)
    );
    let where_clause = pk_where(dialect, &change.primary_key);

    Ok(DmlPreview {
        sql: format!("UPDATE {table_ref} SET {set_clause} WHERE {where_clause};"),
        operation: "UPDATE".into(),
    })
}

fn qualified_table(dialect: Dialect, schema: Option<&str>, table: &str) -> String {
    match dialect {
        Dialect::Postgres => {
            let sch = schema.unwrap_or("public");
            format!("\"{sch}\".\"{table}\"")
        }
        Dialect::Mysql => format!("`{table}`"),
        Dialect::Sqlite => format!("\"{table}\""),
    }
}

fn quote_ident(dialect: Dialect, name: &str) -> String {
    match dialect {
        Dialect::Postgres => format!("\"{name}\""),
        Dialect::Mysql => format!("`{name}`"),
        Dialect::Sqlite => format!("\"{name}\""),
    }
}

fn pk_where(dialect: Dialect, pk: &HashMap<String, CellValue>) -> String {
    pk.iter()
        .map(|(col, val)| format!("{} = {}", quote_ident(dialect, col), sql_literal(val)))
        .collect::<Vec<_>>()
        .join(" AND ")
}

fn sql_literal(value: &CellValue) -> String {
    match value {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_update_sql() {
        let mut pk = HashMap::new();
        pk.insert("id".into(), CellValue::Int64(1));
        let preview = generate_update(
            Dialect::Postgres,
            &CellChangeRequest {
                schema: Some("public".into()),
                table: "users".into(),
                primary_key: pk,
                column: "name".into(),
                old_value: CellValue::String("a".into()),
                new_value: CellValue::String("b".into()),
            },
        )
        .unwrap();
        assert!(preview.sql.contains("UPDATE"));
        assert!(preview.sql.contains("WHERE"));
    }
}
