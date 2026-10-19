import { useEffect, useState } from "react";
import type { Attachment } from "@confluity/shared";
import { api } from "../lib/api";
import { formatTimestamp } from "../lib/preferences";
import { useApp } from "../context/AppContext";

export function AttachmentsPanel({
  entityType,
  entityId,
}: {
  entityType: string;
  entityId: string;
}) {
  const { can, preferences } = useApp();
  const [files, setFiles] = useState<Attachment[]>([]);
  const [error, setError] = useState<string | null>(null);
  const allowed = can("panel.attachments") || can("support.field.attachments");

  async function reload() {
    try {
      setFiles(await api.listAttachments(entityType, entityId));
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    }
  }

  useEffect(() => {
    if (!allowed) return;
    void reload();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [entityType, entityId, allowed]);

  if (!allowed) return null;

  async function onPick(file: File | null) {
    if (!file) return;
    const buf = new Uint8Array(await file.arrayBuffer());
    try {
      await api.addAttachment({
        entity_type: entityType,
        entity_id: entityId,
        filename: file.name,
        mime: file.type || null,
        bytes: Array.from(buf),
      });
      await reload();
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    }
  }

  return (
    <div className="panel">
      <div className="panel-head">
        <h2>Attachments</h2>
        <label className="btn">
          Add file
          <input
            type="file"
            style={{ display: "none" }}
            onChange={(e) => void onPick(e.target.files?.[0] ?? null)}
          />
        </label>
      </div>
      {error && <p style={{ color: "var(--danger)", padding: "0 18px" }}>{error}</p>}
      <div className="list">
        {files.map((f) => (
          <div key={f.id} className="list-item">
            <div>
              <div className="title">{f.filename}</div>
              <div className="meta">
                {Math.max(1, Math.round(f.size_bytes / 1024))} KB
                {f.created_at ? ` · ${formatTimestamp(f.created_at, preferences.timeFormat)}` : ""}
              </div>
            </div>
          </div>
        ))}
        {files.length === 0 && <div className="empty">No files yet</div>}
      </div>
    </div>
  );
}
