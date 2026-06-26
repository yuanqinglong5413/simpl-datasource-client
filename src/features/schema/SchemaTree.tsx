import { useCallback, useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { api } from "@/lib/api";
import type { SchemaMeta, TableMeta } from "@/lib/types";

interface Props {
  connectionId: string | null;
  connected: boolean;
  onSelectTable: (table: TableMeta) => void;
}

export function SchemaTree({ connectionId, connected, onSelectTable }: Props) {
  const { t } = useTranslation("common");
  const [schema, setSchema] = useState<SchemaMeta | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [expanded, setExpanded] = useState<Record<string, boolean>>({});

  const load = useCallback(
    async (force = false) => {
      if (!connectionId || !connected) {
        setSchema(null);
        return;
      }
      setLoading(true);
      setError(null);
      try {
        const meta = await api.getSchema(connectionId, force);
        setSchema(meta);
      } catch (e) {
        setError(String(e));
      } finally {
        setLoading(false);
      }
    },
    [connectionId, connected],
  );

  useEffect(() => {
    void load();
  }, [load]);

  if (!connectionId || !connected) {
    return <div className="empty-state">{t("noConnectionSelected")}</div>;
  }

  return (
    <div className="schema-tree">
      <div style={{ display: "flex", gap: 8, marginBottom: 8 }}>
        <button onClick={() => void load(true)} disabled={loading}>
          {t("refresh")}
        </button>
      </div>
      {error && <div className="error-banner">{error}</div>}
      {loading && !schema && <div>{t("loading")}</div>}
      {schema?.databases.map((db) => (
        <div key={db.name}>
          <div
            className="schema-node"
            onClick={() =>
              setExpanded((e) => ({ ...e, [db.name]: !e[db.name] }))
            }
          >
            📁 {db.name}
          </div>
          {expanded[db.name] !== false &&
            db.schemas.map((sch) => (
              <div key={`${db.name}.${sch.name}`}>
                <div
                  className="schema-node"
                  style={{ paddingLeft: 16 }}
                  onClick={() =>
                    setExpanded((e) => ({
                      ...e,
                      [`${db.name}.${sch.name}`]: !e[`${db.name}.${sch.name}`],
                    }))
                  }
                >
                  📂 {sch.name}
                </div>
                {(expanded[`${db.name}.${sch.name}`] ?? true) &&
                  sch.tables.map((table) => (
                    <div
                      key={`${sch.name}.${table.name}`}
                      className="schema-node table"
                      onClick={() => onSelectTable(table)}
                      title={`${table.columns.length} columns`}
                    >
                      📋 {table.name}
                    </div>
                  ))}
              </div>
            ))}
        </div>
      ))}
    </div>
  );
}
