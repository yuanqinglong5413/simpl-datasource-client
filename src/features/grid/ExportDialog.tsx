import { useState } from "react";
import { useTranslation } from "react-i18next";
import { api } from "@/lib/api";
import type { ExportFormat, RowPage } from "@/lib/types";

interface Props {
  page: RowPage;
  tableName?: string;
  onClose: () => void;
}

const FORMATS: ExportFormat[] = ["csv", "json", "jsonl", "sql", "xlsx"];

/** 导出对话框：选择格式并保存到本地文件。 */
export function ExportDialog({ page, tableName, onClose }: Props) {
  const { t } = useTranslation("common");
  const [format, setFormat] = useState<ExportFormat>("csv");
  const [busy, setBusy] = useState(false);
  const [message, setMessage] = useState<string | null>(null);

  const defaultName = tableName ? `${tableName}.${format}` : `export.${format}`;

  const handleExport = async () => {
    setBusy(true);
    setMessage(null);
    try {
      const path = await api.pickSaveFile(defaultName);
      if (!path) {
        setBusy(false);
        return;
      }
      const result = await api.exportData(page, format, path);
      setMessage(t("exportSuccess", { count: result.rows_written, path: result.file_path }));
    } catch (e) {
      setMessage(String(e));
    } finally {
      setBusy(false);
    }
  };

  return (
    <div className="modal-backdrop" onClick={onClose}>
      <div className="modal" onClick={(e) => e.stopPropagation()}>
        <h2>{t("exportTitle")}</h2>
        {message && <div className="success-banner">{message}</div>}
        <label>{t("exportFormat")}</label>
        <select value={format} onChange={(e) => setFormat(e.target.value as ExportFormat)}>
          {FORMATS.map((f) => (
            <option key={f} value={f}>
              {f.toUpperCase()}
            </option>
          ))}
        </select>
        <div className="modal-actions">
          <button disabled={busy} onClick={onClose}>
            {t("cancel")}
          </button>
          <button className="primary" disabled={busy} onClick={() => void handleExport()}>
            {t("exportExecute")}
          </button>
        </div>
      </div>
    </div>
  );
}
