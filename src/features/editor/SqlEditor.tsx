import { sql } from "@codemirror/lang-sql";
import { oneDark } from "@codemirror/theme-one-dark";
import { EditorState } from "@codemirror/state";
import { EditorView } from "@codemirror/view";
import { basicSetup } from "codemirror";
import { useEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { api } from "@/lib/api";
import type { ExecuteResult } from "@/lib/types";
import { DataGrid } from "@/features/grid/DataGrid";

const STORAGE_KEY = "simplsource.editor.draft";

interface Props {
  connectionId: string | null;
  connectionName: string;
  connected: boolean;
}

export function SqlEditor({ connectionId, connectionName, connected }: Props) {
  const { t } = useTranslation("common");
  const hostRef = useRef<HTMLDivElement>(null);
  const viewRef = useRef<EditorView | null>(null);
  const [results, setResults] = useState<ExecuteResult[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [running, setRunning] = useState(false);
  const [activeResult, setActiveResult] = useState(0);

  useEffect(() => {
    if (!hostRef.current || viewRef.current) return;
    const saved = localStorage.getItem(STORAGE_KEY) ?? "SELECT 1;";
    const state = EditorState.create({
      doc: saved,
      extensions: [
        basicSetup,
        sql(),
        oneDark,
        EditorView.updateListener.of((update) => {
          if (update.docChanged) {
            localStorage.setItem(STORAGE_KEY, update.state.doc.toString());
          }
        }),
      ],
    });
    viewRef.current = new EditorView({ state, parent: hostRef.current });
    return () => {
      viewRef.current?.destroy();
      viewRef.current = null;
    };
  }, []);

  const run = async () => {
    if (!connectionId || !connected || !viewRef.current) return;
    const sqlText = viewRef.current.state.doc.toString();
    setRunning(true);
    setError(null);
    try {
      const res = await api.executeSql(connectionId, sqlText, connectionName);
      setResults(res);
      setActiveResult(0);
    } catch (e) {
      setError(String(e));
      setResults([]);
    } finally {
      setRunning(false);
    }
  };

  if (!connectionId || !connected) {
    return <div className="empty-state">{t("noConnectionSelected")}</div>;
  }

  const current = results[activeResult];
  const gridPage = current
    ? {
        columns: current.columns,
        rows: current.rows,
        offset: 0,
        limit: current.rows.length,
        total_rows: current.rows_affected,
      }
    : null;

  return (
    <div className="split-vertical">
      <div className="editor-toolbar">
        <button className="primary" disabled={running} onClick={() => void run()}>
          ▶ {t("execute")} {running ? "…" : ""}
        </button>
        {results.length > 1 &&
          results.map((_, i) => (
            <button
              key={i}
              className={activeResult === i ? "tab active" : "tab"}
              onClick={() => setActiveResult(i)}
            >
              #{i + 1}
            </button>
          ))}
      </div>
      {error && <div className="error-banner">{error}</div>}
      <div className="editor-container" ref={hostRef} />
      {current?.message && <div className="results-meta">{current.message}</div>}
      {current && (
        <div className="results-meta">
          {current.rows_affected} {t("rows")} · {current.execution_time_ms} {t("ms")}
        </div>
      )}
      <DataGrid page={gridPage} title={undefined} />
    </div>
  );
}
