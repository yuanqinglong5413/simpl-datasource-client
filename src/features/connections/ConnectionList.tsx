import { useCallback, useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { api } from "@/lib/api";
import type { ConnectionConfig, ConnectionRecord } from "@/lib/types";
import { defaultConnection } from "@/lib/types";
import { ConnectionForm } from "./ConnectionForm";

interface Props {
  selectedId: string | null;
  onSelect: (record: ConnectionRecord) => void;
  onConnected: (record: ConnectionRecord) => void;
}

export function ConnectionList({ selectedId, onSelect, onConnected }: Props) {
  const { t } = useTranslation("connections");
  const [records, setRecords] = useState<ConnectionRecord[]>([]);
  const [loading, setLoading] = useState(true);
  const [formOpen, setFormOpen] = useState(false);
  const [editing, setEditing] = useState<ConnectionConfig | null>(null);

  const reload = useCallback(async () => {
    setLoading(true);
    try {
      const list = await api.listConnections();
      setRecords(list);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    void reload();
  }, [reload]);

  const handleConnect = async (record: ConnectionRecord) => {
    try {
      if (record.connected) {
        await api.disconnectDatabase(record.config.id);
      } else {
        await api.connectDatabase(record.config.id);
        onConnected(record);
      }
      await reload();
    } catch (e) {
      alert(String(e));
    }
  };

  return (
    <div style={{ display: "flex", flexDirection: "column", height: "100%" }}>
      <div style={{ padding: "10px 12px", borderBottom: "1px solid var(--border)" }}>
        <button className="primary" style={{ width: "100%" }} onClick={() => {
          setEditing(defaultConnection());
          setFormOpen(true);
        }}>
          + {t("newConnection")}
        </button>
      </div>
      <div style={{ flex: 1, overflow: "auto" }}>
        {loading && <div className="empty-state">{t("loading", { ns: "common" })}</div>}
        {!loading && records.length === 0 && (
          <div className="empty-state" style={{ padding: 24, fontSize: 13 }}>
            {t("emptyConnections", { ns: "common" })}
          </div>
        )}
        {records.map((record) => (
          <div
            key={record.config.id}
            className={`connection-item ${selectedId === record.config.id ? "selected" : ""}`}
            onClick={() => onSelect(record)}
          >
            <div className="name">{record.config.name}</div>
            <span className="badge">{record.config.kind}</span>
            {record.connected && <span className="badge connected">●</span>}
            <button
              onClick={(e) => {
                e.stopPropagation();
                void handleConnect(record);
              }}
            >
              {record.connected ? t("disconnect", { ns: "common" }) : t("connect", { ns: "common" })}
            </button>
            <button
              onClick={(e) => {
                e.stopPropagation();
                setEditing(record.config);
                setFormOpen(true);
              }}
            >
              ✎
            </button>
          </div>
        ))}
      </div>
      {formOpen && editing && (
        <ConnectionForm
          initial={editing}
          onClose={() => setFormOpen(false)}
          onSaved={async () => {
            setFormOpen(false);
            await reload();
          }}
        />
      )}
    </div>
  );
}
