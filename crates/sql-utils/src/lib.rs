use sqlparser::dialect::{MySqlDialect, PostgreSqlDialect, SQLiteDialect};
use sqlparser::parser::Parser;

/// 将多语句 SQL 拆分为独立语句列表（忽略空语句）。
pub fn split_statements(sql: &str, dialect: SqlDialectKind) -> Result<Vec<String>, String> {
    let dialect_impl = dialect.dialect();
    let statements = Parser::parse_sql(&*dialect_impl, sql).map_err(|e| e.to_string())?;
    Ok(statements
        .into_iter()
        .map(|s| s.to_string())
        .filter(|s| !s.trim().is_empty())
        .collect())
}

#[derive(Debug, Clone, Copy)]
pub enum SqlDialectKind {
    Postgres,
    Mysql,
    Sqlite,
}

impl SqlDialectKind {
    fn dialect(&self) -> Box<dyn sqlparser::dialect::Dialect> {
        match self {
            Self::Postgres => Box::new(PostgreSqlDialect {}),
            Self::Mysql => Box::new(MySqlDialect {}),
            Self::Sqlite => Box::new(SQLiteDialect {}),
        }
    }
}

impl From<simpl_driver_trait::Dialect> for SqlDialectKind {
    fn from(value: simpl_driver_trait::Dialect) -> Self {
        match value {
            simpl_driver_trait::Dialect::Postgres => Self::Postgres,
            simpl_driver_trait::Dialect::Mysql => Self::Mysql,
            simpl_driver_trait::Dialect::Sqlite => Self::Sqlite,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_multiple_statements() {
        let parts = split_statements("SELECT 1; SELECT 2;", SqlDialectKind::Postgres).unwrap();
        assert_eq!(parts.len(), 2);
    }
}
