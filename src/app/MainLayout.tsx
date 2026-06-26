import { useCallback, useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { ConnectionList } from "@/features/connections/ConnectionList";
import { SchemaTree } from "@/features/schema/SchemaTree";
import { SqlEditor } from "@/features/editor/SqlEditor";
import { DataGrid } from "@/features/grid/DataGrid";
import { api } from "@/lib/api";
import type { AppInfo, ConnectionRecord, RowPage, TableMeta } from "@/lib/types";

type MainTab = "editor" | "data";

export function MainLayout() {
  const { t } = useTranslation("common");
  const [appInfo, setAppInfo] = useState<AppInfo | null>(null);
  const [pingOk, setPingOk] = useState(false);
  const [selected, setSelected] = useState<ConnectionRecord | null>(null);
  const [tab, setTab] = useState<MainTab>("editor");
  const [tablePage, setTablePage] = useState<RowPage | null>(null);
  const [tableLoading, setTableLoading] = useState(false);
  const [selectedTable, setSelectedTable] = useState<TableMeta | null>(null);

  useEffect(() => {
    void (async () => {
      try {
        await api.ping();
        setPingOk(true);
        setAppInfo(await api.getAppInfo());
      } catch {
        setPingOk(false);
      }
    })();
  }, []);

  const loadTable = useCallback(
    async (table: TableMeta) => {
      if (!selected?.connected) return;
      setTableLoading(true);
      setSelectedTable(table);
      setTab("data");
      try {
        const page = await api.fetchTablePage(selected.config.id, {
          schema: table.schema,
          table: table.name,
          offset: 0,
          limit: 200,
        });
        setTablePage(page);
      } catch (e) {
        alert(String(e));
        setTablePage(null);
      } finally {
        setTableLoading(false);
      }
    },
    [selected],
  );

  return (
    <div className="app-shell">
      <header className="app-header">
        <div style={{ display: "flex", alignItems: "baseline" }}>
          <h1>{appInfo?.name ?? t("appName")}</h1>
          <span className="subtitle">{appInfo?.name_en ?? t("appSubtitle")}</span>
        </div>
        <div style={{ fontSize: 12, color: "var(--text-muted)" }}>
          <span className={`status-dot ${pingOk ? "ok" : "err"}`} />
          {pingOk ? t("pingOk") : t("pingFail")}
          {appInfo && ` · v${appInfo.version}`}
        </div>
      </header>
      <div className="app-body">
        <aside className="sidebar">
          <ConnectionList
            selectedId={selected?.config.id ?? null}
            onSelect={setSelected}
            onConnected={(r) => setSelected({ ...r, connected: true })}
          />
        </aside>
        <aside className="sidebar" style={{ width: 260 }}>
          <div style={{ padding: "8px 12px", borderBottom: "1px solid var(--border)", fontWeight: 600 }}>
            {t("schema")}
          </div>
          <SchemaTree
            connectionId={selected?.config.id ?? null}
            connected={!!selected?.connected}
            onSelectTable={(table) => void loadTable(table)}
          />
        </aside>
        <main className="main-panel">
          <div className="tabs">
            <button
              className={`tab ${tab === "editor" ? "active" : ""}`}
              onClick={() => setTab("editor")}
            >
              {t("editor")}
            </button>
            <button
              className={`tab ${tab === "data" ? "active" : ""}`}
              onClick={() => setTab("data")}
            >
              {t("data")}
              {selectedTable ? ` · ${selectedTable.name}` : ""}
            </button>
          </div>
          {tab === "editor" ? (
            <SqlEditor
              connectionId={selected?.config.id ?? null}
              connectionName={selected?.config.name ?? ""}
              connected={!!selected?.connected}
            />
          ) : (
            <DataGrid
              page={tablePage}
              loading={tableLoading}
              title={selectedTable ? `${selectedTable.schema}.${selectedTable.name}` : undefined}
            />
          )}
        </main>
      </div>
    </div>
  );
}
