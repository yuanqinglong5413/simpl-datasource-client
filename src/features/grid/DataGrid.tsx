import { useCallback, useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { api, buildPrimaryKeyRow } from "@/lib/api";
import type { CellChangeRequest, CellValue, ColumnMeta, DmlPreview, RowPage, TableMeta } from "@/lib/types";
import { formatCell, parseCellInput } from "@/lib/types";
import { EditPreviewDialog } from "./EditPreviewDialog";
import { ExportDialog } from "./ExportDialog";
import { TransactionBar } from "./TransactionBar";
import type { TransactionStatus } from "@/lib/types";

interface Props {
  page: RowPage | null;
  loading?: boolean;
  title?: string;
  connectionId?: string | null;
  table?: TableMeta | null;
  readOnly?: boolean;
  onRefresh?: () => void;
  onImport?: () => void;
}

interface EditState {
  rowIndex: number;
  colIndex: number;
  draft: string;
}

export function DataGrid({
  page,
  loading,
  title,
  connectionId,
  table,
  readOnly,
  onRefresh,
  onImport,
}: Props) {
  const { t } = useTranslation("common");
  const [editing, setEditing] = useState<EditState | null>(null);
  const [preview, setPreview] = useState<DmlPreview | null>(null);
  const [pendingChange, setPendingChange] = useState<CellChangeRequest | null>(null);
  const [txStatus, setTxStatus] = useState<TransactionStatus>({ active: false, pending_count: 0 });
  const [busy, setBusy] = useState(false);
  const [showExport, setShowExport] = useState(false);

  const editable =
    !!connectionId &&
    !!table &&
    !readOnly &&
    table.primary_key.length > 0;

  const refreshTxStatus = useCallback(async () => {
    if (!connectionId) return;
    try {
      setTxStatus(await api.getTransactionStatus(connectionId));
    } catch {
      /* ignore */
    }
  }, [connectionId]);

  useEffect(() => {
    void refreshTxStatus();
  }, [refreshTxStatus, page]);

  const columns: ColumnMeta[] =
    page && page.columns.length > 0
      ? page.columns
      : page?.rows[0]?.map((_, i) => ({
          name: `col_${i + 1}`,
          data_type: "unknown",
          nullable: true,
          is_primary_key: false,
        })) ?? [];

  const startEdit = (rowIndex: number, colIndex: number, cell: CellValue) => {
    if (!editable) return;
    const col = columns[colIndex];
    if (col?.is_primary_key) return;
    setEditing({ rowIndex, colIndex, draft: formatCell(cell) });
  };

  const submitEdit = async () => {
    if (!editing || !page || !connectionId || !table) return;
    const { rowIndex, colIndex, draft } = editing;
    const col = columns[colIndex];
    const oldCell = page.rows[rowIndex][colIndex];
    const newCell = parseCellInput(draft, oldCell);
    if (formatCell(oldCell) === formatCell(newCell)) {
      setEditing(null);
      return;
    }

    const change: CellChangeRequest = {
      schema: table.schema,
      table: table.name,
      primary_key: buildPrimaryKeyRow(columns, table.primary_key, page.rows[rowIndex]),
      column: col.name,
      old_value: oldCell,
      new_value: newCell,
    };

    setBusy(true);
    try {
      const result = await api.previewCellChange(connectionId, change);
      setPendingChange(change);
      setPreview(result);
      setEditing(null);
    } catch (e) {
      alert(String(e));
    } finally {
      setBusy(false);
    }
  };

  const confirmQueue = async () => {
    if (!pendingChange || !connectionId) return;
    setBusy(true);
    try {
      await api.queueCellChange(connectionId, pendingChange);
      setPreview(null);
      setPendingChange(null);
      await refreshTxStatus();
    } catch (e) {
      alert(String(e));
    } finally {
      setBusy(false);
    }
  };

  const handleCommit = async () => {
    if (!connectionId) return;
    setBusy(true);
    try {
      await api.commitTransaction(connectionId);
      await refreshTxStatus();
      onRefresh?.();
    } catch (e) {
      alert(String(e));
    } finally {
      setBusy(false);
    }
  };

  const handleRollback = async () => {
    if (!connectionId) return;
    setBusy(true);
    try {
      await api.rollbackTransaction(connectionId);
      await refreshTxStatus();
    } catch (e) {
      alert(String(e));
    } finally {
      setBusy(false);
    }
  };

  if (loading) {
    return <div className="empty-state">{t("loading")}</div>;
  }

  if (!page) {
    return <div className="empty-state">{t("selectTableHint")}</div>;
  }

  return (
    <div className="results-panel">
      <div className="grid-toolbar">
        {title && <div className="results-meta">{title}</div>}
        <div className="grid-toolbar-actions">
          <span className="results-meta">
            {page.rows.length} {t("rows")} · offset {page.offset}
          </span>
          {onImport && (
            <button disabled={busy} onClick={onImport}>
              {t("import")}
            </button>
          )}
          <button disabled={busy} onClick={() => setShowExport(true)}>
            {t("export")}
          </button>
        </div>
      </div>

      <TransactionBar
        status={txStatus}
        busy={busy}
        onCommit={() => void handleCommit()}
        onRollback={() => void handleRollback()}
      />

      {!editable && table && table.primary_key.length === 0 && !readOnly && (
        <div className="info-banner">{t("noPrimaryKeyHint")}</div>
      )}

      <div className="data-grid-wrap">
        <table className="data-grid">
          <thead>
            <tr>
              {columns.map((col) => (
                <th key={col.name}>
                  {col.name}
                  {col.is_primary_key && " 🔑"}
                </th>
              ))}
            </tr>
          </thead>
          <tbody>
            {page.rows.map((row, ri) => (
              <tr key={ri}>
                {row.map((cell, ci) => {
                  const isEditing = editing?.rowIndex === ri && editing?.colIndex === ci;
                  return (
                    <td
                      key={ci}
                      className={editable && !columns[ci]?.is_primary_key ? "editable-cell" : undefined}
                      onDoubleClick={() => startEdit(ri, ci, cell)}
                    >
                      {isEditing ? (
                        <input
                          className="cell-editor"
                          autoFocus
                          value={editing.draft}
                          onChange={(e) =>
                            setEditing({ rowIndex: ri, colIndex: ci, draft: e.target.value })
                          }
                          onBlur={() => void submitEdit()}
                          onKeyDown={(e) => {
                            if (e.key === "Enter") void submitEdit();
                            if (e.key === "Escape") setEditing(null);
                          }}
                        />
                      ) : (
                        formatCell(cell)
                      )}
                    </td>
                  );
                })}
              </tr>
            ))}
          </tbody>
        </table>
      </div>

      {preview && (
        <EditPreviewDialog
          preview={preview}
          busy={busy}
          onConfirm={() => void confirmQueue()}
          onCancel={() => {
            setPreview(null);
            setPendingChange(null);
          }}
        />
      )}

      {showExport && (
        <ExportDialog
          page={page}
          tableName={table?.name}
          onClose={() => setShowExport(false)}
        />
      )}
    </div>
  );
}
