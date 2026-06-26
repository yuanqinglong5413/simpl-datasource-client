import { invoke } from "@tauri-apps/api/core";
import type {
  AppInfo,
  ConnectionConfig,
  ConnectionRecord,
  ExecuteResult,
  RowPage,
  SchemaMeta,
} from "./types";

export const api = {
  ping: () => invoke<{ message: string }>("ping"),
  getAppInfo: () => invoke<AppInfo>("get_app_info"),
  listConnections: () => invoke<ConnectionRecord[]>("list_connections"),
  saveConnection: (config: ConnectionConfig, password?: string) =>
    invoke<ConnectionConfig>("save_connection", {
      request: { config, password: password ?? null },
    }),
  deleteConnection: (id: string) => invoke<void>("delete_connection", { id }),
  testConnection: (config: ConnectionConfig, password?: string) =>
    invoke<void>("test_connection", {
      request: { config, password: password ?? null },
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
};
