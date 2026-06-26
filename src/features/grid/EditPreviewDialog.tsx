import { useTranslation } from "react-i18next";
import type { DmlPreview } from "@/lib/types";

interface Props {
  preview: DmlPreview;
  onConfirm: () => void;
  onCancel: () => void;
  busy?: boolean;
}

/** 内联编辑 SQL 预览确认对话框。 */
export function EditPreviewDialog({ preview, onConfirm, onCancel, busy }: Props) {
  const { t } = useTranslation("common");

  return (
    <div className="modal-backdrop" onClick={onCancel}>
      <div className="modal modal-wide" onClick={(e) => e.stopPropagation()}>
        <h2>{t("editPreviewTitle")}</h2>
        <p className="text-muted">{t("editPreviewHint")}</p>
        <pre className="sql-preview">{preview.sql}</pre>
        <div className="modal-actions">
          <button disabled={busy} onClick={onCancel}>
            {t("cancel")}
          </button>
          <button className="primary" disabled={busy} onClick={onConfirm}>
            {t("queueChange")}
          </button>
        </div>
      </div>
    </div>
  );
}
