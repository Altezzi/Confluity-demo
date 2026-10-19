import { useState } from "react";
import { api, recordPath } from "../lib/api";
import { useApp } from "../context/AppContext";

export function RecordActions({
  entityType,
  entityId,
  label,
}: {
  entityType: string;
  entityId: string;
  label: string;
}) {
  const { notifyRecordChange } = useApp();
  const [copied, setCopied] = useState(false);
  const [notified, setNotified] = useState(false);

  async function copyLink() {
    const hash = `#${recordPath(entityType, entityId)}`;
    await navigator.clipboard.writeText(`${window.location.origin}${window.location.pathname}${hash}`);
    setCopied(true);
    setTimeout(() => setCopied(false), 1600);
  }

  async function notify() {
    await api.notifyWorkspace(`Look at ${label}`, `Workspace record ${entityType}`, entityType, entityId);
    notifyRecordChange();
    setNotified(true);
    setTimeout(() => setNotified(false), 1600);
  }

  return (
    <div style={{ display: "flex", gap: 8 }}>
      <button className="btn" type="button" onClick={() => void copyLink()}>
        {copied ? "Copied" : "Copy link"}
      </button>
      <button className="btn" type="button" onClick={() => void notify()}>
        {notified ? "Notified" : "Notify workspace"}
      </button>
    </div>
  );
}
