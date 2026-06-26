import { invoke } from "@tauri-apps/api/core";
import type {
  AppInfo,
  CellChangeRequest,
  CellValue,
  ConnectionConfig,
  ConnectionRecord,
  DmlPreview,
  ExecuteResult,
  ExportFormat,
  ExportResult,
  ImportFileFormat,
  ImportPreview,
  ImportRequest,
  ImportResult,
  RowPage,
  SchemaMeta,
  TransactionStatus,
} from "./types";

export const api = {
  ping: () => invoke<{ message: string }>("ping"),
  getAppInfo: () => invoke<AppInfo>("get_app_info"),
  listConnections: () => invoke<ConnectionRecord[]>("list_connections"),
  saveConnection: (
    config: ConnectionConfig,
    password?: string,
    sshPassword?: string,
  ) =>
    invoke<ConnectionConfig>("save_connection", {
      request: {
        config,
        password: password ?? null,
        ssh_password: sshPassword ?? null,
      },
    }),
  deleteConnection: (id: string) => invoke<void>("delete_connection", { id }),
  testConnection: (
    config: ConnectionConfig,
    password?: string,
    sshPassword?: string,
  ) =>
    invoke<void>("test_connection", {
      request: {
        config,
        password: password ?? null,
        ssh_password: sshPassword ?? null,
      },
    }),
  connectDatabase: (id: string) =>
    invoke<ConnectionConfig>("connect_database", { id }),
  disconnectDatabase: (id: string) =>
    invoke<void>("disconnect_database", { id }),
  getSchema: (connectionId: string, forceRefresh = false) =>
    invoke<SchemaMeta>("get_schema", {
      connectionId,
      forceRefresh,
    }),
  executeSql: (connectionId: string, sql: string, connectionName: string) =>
    invoke<ExecuteResult[]>("execute_sql", {
      connectionId,
      sql,
      connectionName,
    }),
  fetchTablePage: (
    connectionId: string,
    request: {
      schema?: string | null;
      table: string;
      offset: number;
      limit: number;
      order_by?: string | null;
      order_desc?: boolean;
      filter?: string | null;
    },
  ) =>
    invoke<RowPage>("fetch_table_page", {
      connectionId,
      request,
    }),
  listQueryHistory: (connectionId?: string, limit = 50) =>
    invoke<unknown[]>("list_query_history", { connectionId, limit }),

  previewCellChange: (connectionId: string, change: CellChangeRequest) =>
    invoke<DmlPreview>("preview_cell_change", { connectionId, change }),

  queueCellChange: (connectionId: string, change: CellChangeRequest) =>
    invoke<DmlPreview>("queue_cell_change", { connectionId, change }),

  commitTransaction: (connectionId: string) =>
    invoke<number>("commit_transaction", { connectionId }),

  rollbackTransaction: (connectionId: string) =>
    invoke<void>("rollback_transaction", { connectionId }),

  getTransactionStatus: (connectionId: string) =>
    invoke<TransactionStatus>("get_transaction_status", { connectionId }),

  exportData: (page: RowPage, format: ExportFormat, filePath: string) =>
    invoke<ExportResult>("export_data", { page, format, filePath }),

  importPreview: (
    connectionId: string,
    filePath: string,
    format: ImportFileFormat,
    table: string,
    schema?: string,
  ) =>
    invoke<ImportPreview>("import_preview", {
      connectionId,
      filePath,
      format,
      table,
      schema: schema ?? null,
    }),

  importExecute: (connectionId: string, request: ImportRequest) =>
    invoke<ImportResult>("import_execute", { connectionId, request }),

  pickSaveFile: (defaultName?: string) =>
    invoke<string | null>("pick_save_file", { defaultName: defaultName ?? null }),

  pickOpenFile: () => invoke<string | null>("pick_open_file"),
};

/** 从行数据构建主键 map（用于内联编辑 WHERE 子句）。 */
export function buildPrimaryKeyRow(
  columns: { name: string }[],
  primaryKey: string[],
  row: CellValue[],
): Record<string, CellValue> {
  const pk: Record<string, CellValue> = {};
  for (const key of primaryKey) {
    const idx = columns.findIndex((c) => c.name === key);
    if (idx >= 0 && row[idx]) {
      pk[key] = row[idx];
    }
  }
  return pk;
}
