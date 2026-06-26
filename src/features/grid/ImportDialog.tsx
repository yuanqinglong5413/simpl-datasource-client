import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { api } from "@/lib/api";
import type { ImportFileFormat, ImportPreview, TableMeta } from "@/lib/types";

interface Props {
  connectionId: string;
  table: TableMeta;
  filePath: string;
  format: ImportFileFormat;
  onClose: () => void;
  onDone: () => void;
}

/** 导入向导：预览文件、映射列、执行 INSERT。 */
export function ImportDialog({
  connectionId,
  table,
  filePath,
  format,
  onClose,
  onDone,
}: Props) {
  const { t } = useTranslation("common");
  const [preview, setPreview] = useState<ImportPreview | null>(null);
  const [mapping, setMapping] = useState<Record<string, string>>({});
  const [busy, setBusy] = useState(false);
  const [message, setMessage] = useState<string | null>(null);

  useEffect(() => {
    void (async () => {
      setBusy(true);
      try {
        const data = await api.importPreview(
          connectionId,
          filePath,
          format,
          table.name,
          table.schema,
        );
        setPreview(data);
        setMapping(data.suggested_mapping);
      } catch (e) {
        setMessage(String(e));
      } finally {
        setBusy(false);
      }
    })();
  }, [connectionId, filePath, format, table]);

  const handleImport = async () => {
    setBusy(true);
    setMessage(null);
    try {
      const result = await api.importExecute(connectionId, {
        file_path: filePath,
        schema: table.schema,
        table: table.name,
        format,
        mapping: { column_map: mapping },
      });
      setMessage(t("importSuccess", { count: result.rows_imported }));
      onDone();
    } catch (e) {
      setMessage(String(e));
    } finally {
      setBusy(false);
    }
  };

  return (
    <div className="modal-backdrop" onClick={onClose}>
      <div className="modal modal-wide" onClick={(e) => e.stopPropagation()}>
        <h2>{t("importTitle")}</h2>
        <p className="text-muted">{filePath}</p>
        {message && <div className="error-banner">{message}</div>}
        {busy && !preview && <div>{t("loading")}</div>}
        {preview && (
          <>
            <div className="mapping-grid">
              {preview.headers.map((header) => (
                <div key={header} className="mapping-row">
                  <span>{header}</span>
                  <span>→</span>
                  <select
                    value={mapping[header] ?? ""}
                    onChange={(e) =>
                      setMapping((m) => ({ ...m, [header]: e.target.value }))
                    }
                  >
                    <option value="">{t("skipColumn")}</option>
                    {table.columns.map((col) => (
                      <option key={col.name} value={col.name}>
                        {col.name}
                      </option>
                    ))}
                  </select>
                </div>
              ))}
            </div>
            {preview.sample_rows.length > 0 && (
              <div className="data-grid-wrap" style={{ maxHeight: 160, marginTop: 12 }}>
                <table className="data-grid">
                  <thead>
                    <tr>
                      {preview.headers.map((h) => (
                        <th key={h}>{h}</th>
                      ))}
                    </tr>
                  </thead>
                  <tbody>
                    {preview.sample_rows.map((row, i) => (
                      <tr key={i}>
                        {row.map((cell, j) => (
                          <td key={j}>{cell}</td>
                        ))}
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            )}
          </>
        )}
        <div className="modal-actions">
          <button disabled={busy} onClick={onClose}>
            {t("cancel")}
          </button>
          <button
            className="primary"
            disabled={busy || !preview}
            onClick={() => void handleImport()}
          >
            {t("importExecute")}
          </button>
        </div>
      </div>
    </div>
  );
}
