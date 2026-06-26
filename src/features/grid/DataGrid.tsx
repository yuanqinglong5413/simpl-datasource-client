import { useTranslation } from "react-i18next";
import type { ColumnMeta, RowPage } from "@/lib/types";
import { formatCell } from "@/lib/types";

interface Props {
  page: RowPage | null;
  loading?: boolean;
  title?: string;
}

export function DataGrid({ page, loading, title }: Props) {
  const { t } = useTranslation("common");

  if (loading) {
    return <div className="empty-state">{t("loading")}</div>;
  }

  if (!page) {
    return <div className="empty-state">{t("selectTableHint")}</div>;
  }

  const columns: ColumnMeta[] =
    page.columns.length > 0
      ? page.columns
      : page.rows[0]?.map((_, i) => ({
          name: `col_${i + 1}`,
          data_type: "unknown",
          nullable: true,
          is_primary_key: false,
        })) ?? [];

  return (
    <div className="results-panel">
      {title && <div className="results-meta">{title}</div>}
      <div className="results-meta">
        {page.rows.length} {t("rows")} · offset {page.offset}
      </div>
      <div className="data-grid-wrap">
        <table className="data-grid">
          <thead>
            <tr>
              {columns.map((col) => (
                <th key={col.name}>{col.name}</th>
              ))}
            </tr>
          </thead>
          <tbody>
            {page.rows.map((row, ri) => (
              <tr key={ri}>
                {row.map((cell, ci) => (
                  <td key={ci}>{formatCell(cell)}</td>
                ))}
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}
