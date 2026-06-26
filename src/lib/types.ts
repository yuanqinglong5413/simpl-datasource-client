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
