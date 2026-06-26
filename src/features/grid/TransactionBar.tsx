import { useTranslation } from "react-i18next";
import type { TransactionStatus } from "@/lib/types";

interface Props {
  status: TransactionStatus;
  busy?: boolean;
  onCommit: () => void;
  onRollback: () => void;
}

/** Safe Mode 事务工具栏：展示待提交变更数，提供提交/回滚。 */
export function TransactionBar({ status, busy, onCommit, onRollback }: Props) {
  const { t } = useTranslation("common");

  if (status.pending_count === 0) {
    return null;
  }

  return (
    <div className="transaction-bar">
      <span>
        {t("pendingChanges", { count: status.pending_count })}
      </span>
      <div className="transaction-actions">
        <button disabled={busy} onClick={onRollback}>
          {t("rollback")}
        </button>
        <button className="primary" disabled={busy} onClick={onCommit}>
          {t("commit")}
        </button>
      </div>
    </div>
  );
}
