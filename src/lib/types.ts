/** IPC 共享类型（与 Rust serde 结构对齐） */

export type DatabaseKind = "postgres" | "mysql" | "sqlite";

export interface ConnectionConfig {
  id: string;
  name: string;
  kind: DatabaseKind;
  host: string;
  port: number;
  database: string;
  username: string;
  ssl_mode: "prefer" | "require" | "disable" | "verify_ca";
  ssh: {
    enabled: boolean;
    host: string;
    port: number;
    username: string;
    private_key_path?: string | null;
  };
  read_only: boolean;
  environment?: string | null;
  color?: string | null;
  created_at: string;
  updated_at: string;
}

export interface ConnectionRecord {
  config: ConnectionConfig;
  connected: boolean;
}

export interface ColumnMeta {
  name: string;
  data_type: string;
  nullable: boolean;
  is_primary_key: boolean;
}

export interface TableMeta {
  schema: string;
  name: string;
  columns: ColumnMeta[];
  primary_key: string[];
  row_count?: number | null;
}

export interface SchemaMeta {
  databases: Array<{
    name: string;
    schemas: Array<{
      name: string;
      tables: TableMeta[];
      views: TableMeta[];
    }>;
  }>;
}

export type CellValue =
  | { type: "Null" }
  | { type: "Bool"; value: boolean }
  | { type: "Int64"; value: number }
  | { type: "Float64"; value: number }
  | { type: "String"; value: string }
  | { type: "Bytes"; value: string }
  | { type: "Json"; value: unknown }
  | { type: "DateTime"; value: string };

export interface ExecuteResult {
  columns: ColumnMeta[];
  rows: CellValue[][];
  rows_affected: number;
  execution_time_ms: number;
  message?: string | null;
}

export interface RowPage {
  columns: ColumnMeta[];
  rows: CellValue[][];
  total_rows?: number | null;
  offset: number;
  limit: number;
}

export interface AppInfo {
  name: string;
  name_en: string;
  version: string;
  uses_keyring: boolean;
  data_dir: string;
}

export interface DmlPreview {
  sql: string;
  operation: string;
}

export interface TransactionStatus {
  active: boolean;
  pending_count: number;
}

export type ExportFormat = "csv" | "json" | "jsonl" | "sql" | "xlsx";

export interface ExportResult {
  rows_written: number;
  file_path: string;
}

export type ImportFileFormat = "csv" | "json";

export interface ImportPreview {
  headers: string[];
  sample_rows: string[][];
  suggested_mapping: Record<string, string>;
}

export interface ImportRequest {
  file_path: string;
  schema?: string | null;
  table: string;
  mapping: { column_map: Record<string, string> };
  format: ImportFileFormat;
}

export interface ImportResult {
  rows_imported: number;
  insert_statements: string[];
}

export interface CellChangeRequest {
  schema?: string | null;
  table: string;
  primary_key: Record<string, CellValue>;
  column: string;
  old_value: CellValue;
  new_value: CellValue;
}

export function parseCellInput(text: string, sample: CellValue): CellValue {
  const trimmed = text.trim();
  if (trimmed.toUpperCase() === "NULL" || trimmed === "") {
    return { type: "Null" };
  }
  switch (sample.type) {
    case "Bool":
      return { type: "Bool", value: trimmed === "true" || trimmed === "1" };
    case "Int64": {
      const n = Number(trimmed);
      return Number.isFinite(n) ? { type: "Int64", value: n } : { type: "String", value: trimmed };
    }
    case "Float64": {
      const n = Number(trimmed);
      return Number.isFinite(n) ? { type: "Float64", value: n } : { type: "String", value: trimmed };
    }
    default:
      return { type: "String", value: trimmed };
  }
}

export function formatCell(cell: CellValue): string {
  switch (cell.type) {
    case "Null":
      return "NULL";
    case "Bool":
      return cell.value ? "true" : "false";
    case "Int64":
    case "Float64":
      return String(cell.value);
    case "String":
    case "DateTime":
    case "Bytes":
      return cell.value;
    case "Json":
      return JSON.stringify(cell.value);
    default:
      return "";
  }
}

export function defaultConnection(kind: DatabaseKind = "postgres"): ConnectionConfig {
  const now = new Date().toISOString();
  return {
    id: crypto.randomUUID(),
    name: "",
    kind,
    host: "127.0.0.1",
    port: kind === "postgres" ? 5432 : kind === "mysql" ? 3306 : 0,
    database: kind === "sqlite" ? "./database.sqlite" : "postgres",
    username: kind === "sqlite" ? "" : "postgres",
    ssl_mode: "prefer",
    ssh: { enabled: false, host: "", port: 22, username: "" },
    read_only: false,
    environment: "dev",
    color: null,
    created_at: now,
    updated_at: now,
  };
}
