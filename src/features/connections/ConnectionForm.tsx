import { useState } from "react";
import { useTranslation } from "react-i18next";
import { api } from "@/lib/api";
import type { ConnectionConfig, DatabaseKind } from "@/lib/types";

interface Props {
  initial: ConnectionConfig;
  onClose: () => void;
  onSaved: () => void;
}

export function ConnectionForm({ initial, onClose, onSaved }: Props) {
  const { t } = useTranslation("connections");
  const [config, setConfig] = useState(initial);
  const [password, setPassword] = useState("");
  const [busy, setBusy] = useState(false);
  const [message, setMessage] = useState<string | null>(null);

  const update = <K extends keyof ConnectionConfig>(key: K, value: ConnectionConfig[K]) => {
    setConfig((c) => ({ ...c, [key]: value }));
  };

  const handleTest = async () => {
    setBusy(true);
    setMessage(null);
    try {
      await api.testConnection(config, password || undefined);
      setMessage(t("testSuccess"));
    } catch (e) {
      setMessage(`${t("testFailed")}: ${e}`);
    } finally {
      setBusy(false);
    }
  };

  const handleSave = async () => {
    if (!config.name.trim()) {
      setMessage(t("connectionName") + " required");
      return;
    }
    setBusy(true);
    try {
      await api.saveConnection(config, password || undefined);
      onSaved();
    } catch (e) {
      setMessage(String(e));
    } finally {
      setBusy(false);
    }
  };

  const handleDelete = async () => {
    if (!confirm(t("deleteConfirm"))) return;
    setBusy(true);
    try {
      await api.deleteConnection(config.id);
      onSaved();
    } catch (e) {
      setMessage(String(e));
    } finally {
      setBusy(false);
    }
  };

  return (
    <div className="modal-backdrop" onClick={onClose}>
      <div className="modal" onClick={(e) => e.stopPropagation()}>
        <h2>{initial.name ? t("editConnection") : t("newConnection")}</h2>
        {message && <div className="error-banner">{message}</div>}
        <div className="form-grid">
          <label>{t("connectionName")}</label>
          <input value={config.name} onChange={(e) => update("name", e.target.value)} />

          <label>{t("databaseType")}</label>
          <select
            value={config.kind}
            onChange={(e) => update("kind", e.target.value as DatabaseKind)}
          >
            <option value="postgres">PostgreSQL</option>
            <option value="mysql">MySQL</option>
            <option value="sqlite">SQLite</option>
          </select>

          {config.kind !== "sqlite" && (
            <>
              <label>{t("host")}</label>
              <input value={config.host} onChange={(e) => update("host", e.target.value)} />
              <label>{t("port")}</label>
              <input
                type="number"
                value={config.port}
                onChange={(e) => update("port", Number(e.target.value))}
              />
              <label>{t("username")}</label>
              <input value={config.username} onChange={(e) => update("username", e.target.value)} />
            </>
          )}

          <label>{t("database")}</label>
          <input value={config.database} onChange={(e) => update("database", e.target.value)} />

          {config.kind !== "sqlite" && (
            <>
              <label>{t("password")}</label>
              <input
                type="password"
                value={password}
                onChange={(e) => setPassword(e.target.value)}
                placeholder="••••••"
              />
            </>
          )}

          <label>{t("readOnly")}</label>
          <input
            type="checkbox"
            checked={config.read_only}
            onChange={(e) => update("read_only", e.target.checked)}
          />
        </div>
        <div className="modal-actions">
          {initial.name && (
            <button className="danger" disabled={busy} onClick={() => void handleDelete()}>
              {t("delete")}
            </button>
          )}
          <button disabled={busy} onClick={onClose}>
            {t("cancel")}
          </button>
          <button disabled={busy} onClick={() => void handleTest()}>
            {t("test")}
          </button>
          <button className="primary" disabled={busy} onClick={() => void handleSave()}>
            {t("save")}
          </button>
        </div>
      </div>
    </div>
  );
}
