import type { ReactNode } from "react";
import { BackLink } from "../../components/BackLink";

export function newConnectionId(prefix: string) {
  return `${prefix}_${crypto.randomUUID().slice(0, 8)}`;
}

export function PlusIcon() {
  return (
    <svg width="16" height="16" viewBox="0 0 24 24" fill="none" aria-hidden>
      <path d="M12 5v14M5 12h14" stroke="currentColor" strokeWidth="2" strokeLinecap="round" />
    </svg>
  );
}

export function ListPageHeader({
  title,
  onAdd,
  addLabel,
  backTo,
  backLabel,
}: {
  title: string;
  onAdd: () => void;
  addLabel: string;
  backTo: string;
  backLabel: string;
}) {
  return (
    <>
      <BackLink to={backTo}>{backLabel}</BackLink>
      <div className="panel-head">
        <h2 style={{ margin: 0 }}>{title}</h2>
        <button type="button" className="btn btn-icon" title={addLabel} aria-label={addLabel} onClick={onAdd}>
          <PlusIcon />
        </button>
      </div>
    </>
  );
}

export function ConnectionRow({
  title,
  subtitle,
  meta,
  actions,
}: {
  title: string;
  subtitle: string;
  meta: string;
  actions: ReactNode;
}) {
  return (
    <li className="panel" style={{ marginBottom: 10, padding: "12px 14px", listStyle: "none" }}>
      <div style={{ display: "flex", justifyContent: "space-between", gap: 12, alignItems: "flex-start" }}>
        <div style={{ minWidth: 0 }}>
          <strong>{title}</strong>
          <div className="meta" style={{ wordBreak: "break-all" }}>
            {subtitle}
          </div>
          <div className="meta">{meta}</div>
        </div>
        <div style={{ display: "flex", gap: 8, flexWrap: "wrap", flexShrink: 0 }}>{actions}</div>
      </div>
    </li>
  );
}
