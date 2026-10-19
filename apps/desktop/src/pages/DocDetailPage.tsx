import { useEffect, useState } from "react";
import { Link, useParams } from "react-router-dom";
import type { Document } from "@confluity/shared";
import { api } from "../lib/api";
import { AttachmentsPanel } from "../components/AttachmentsPanel";
import { LinkedRecordsPanel } from "../components/LinkedRecords";
import { useApp } from "../context/AppContext";

export function DocDetailPage() {
  const { id = "" } = useParams();
  const { setFocus, can } = useApp();
  const [doc, setDoc] = useState<Document | null>(null);

  useEffect(() => {
    void (async () => {
      const d = await api.getDocument(id);
      setDoc(d);
      if (d) setFocus({ entityType: "document", entityId: d.id, label: d.title });
    })();
  }, [id, setFocus]);

  if (!doc) return <div className="empty">Document not found</div>;

  const isInternal = (doc.visibility ?? "internal") === "internal";
  if (isInternal && !can("docs.field.internal")) {
    return <div className="empty">This internal document is hidden by your role.</div>;
  }

  return (
    <div className="page">
      <div className="page-header">
        <div>
          <div className="eyebrow">Document</div>
          <h1>{doc.title}</h1>
          <p>{doc.category || "Uncategorized"}</p>
        </div>
        <Link className="btn" to="/docs">
          Back
        </Link>
      </div>

      <div className="grid-2">
        <div>
          <div className="panel detail-block">
            <h2>Content</h2>
            <pre>{doc.body}</pre>
          </div>
          <AttachmentsPanel entityType="document" entityId={doc.id} />
        </div>
        <LinkedRecordsPanel entityType="document" entityId={doc.id} />
      </div>
    </div>
  );
}
