import { useEffect, useRef, useState } from "react";
import { useNavigate } from "react-router-dom";
import type { Member } from "@confluity/shared";
import { avatarTone, personInitials } from "./PersonChip";

function chatTime(iso: string): string {
  const date = new Date(iso);
  if (Number.isNaN(date.getTime())) return "";
  const now = new Date();
  const sameDay = date.toDateString() === now.toDateString();
  if (sameDay) return date.toLocaleTimeString(undefined, { hour: "numeric", minute: "2-digit" });
  return date.toLocaleString(undefined, {
    month: "short",
    day: "numeric",
    hour: "numeric",
    minute: "2-digit",
  });
}

function localEmail(name: string, org?: string | null): string {
  const slug = name
    .trim()
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, ".")
    .replace(/^\.+|\.+$/g, "");
  const domain = (org || "confluity.local")
    .trim()
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "")
    .slice(0, 24);
  return `${slug || "user"}@${domain || "confluity"}.local`;
}

export function MessageAuthor({
  member,
  name,
  memberId,
  organizationName,
  placement,
  timestamp,
  body,
  onMessage,
  canMessage,
  mine,
  continued,
}: {
  member?: Member | null;
  name: string;
  memberId?: string | null;
  organizationName?: string | null;
  placement?: string | null;
  timestamp: string;
  body: string;
  onMessage?: (memberId: string) => void;
  canMessage?: boolean;
  mine?: boolean;
  continued?: boolean;
}) {
  const navigate = useNavigate();
  const wrapRef = useRef<HTMLDivElement | null>(null);
  const [open, setOpen] = useState(false);
  const displayName = member?.name || name;
  const tone = avatarTone(displayName);
  const dept = placement?.trim() || member?.title?.trim() || "";
  const email = localEmail(displayName, organizationName);
  const profileId = member?.id || memberId || null;

  useEffect(() => {
    if (!open) return;
    function onPointerDown(e: MouseEvent) {
      if (!wrapRef.current?.contains(e.target as Node)) setOpen(false);
    }
    function onKey(e: KeyboardEvent) {
      if (e.key === "Escape") setOpen(false);
    }
    document.addEventListener("mousedown", onPointerDown);
    document.addEventListener("keydown", onKey);
    return () => {
      document.removeEventListener("mousedown", onPointerDown);
      document.removeEventListener("keydown", onKey);
    };
  }, [open]);

  function toggleCard() {
    if (!profileId) return;
    setOpen((v) => !v);
  }

  function openProfile() {
    if (!profileId) return;
    setOpen(false);
    navigate(`/profile/${profileId}`);
  }

  return (
    <div
      ref={wrapRef}
      className={`teams-msg member-hover-wrap${mine ? " mine" : ""}${continued ? " continued" : ""}${open ? " open" : ""}`}
    >
      <button
        type="button"
        className="member-hover-avatar-btn"
        onClick={toggleCard}
        disabled={!profileId}
        aria-expanded={open}
        title={profileId ? `View ${displayName}` : displayName}
      >
        <span className="teams-person-avatar sm" style={tone} aria-hidden>
          {personInitials(displayName)}
        </span>
      </button>
      <div className="teams-msg-body">
        <div className="teams-msg-meta">
          <button
            type="button"
            className="member-hover-name"
            onClick={toggleCard}
            disabled={!profileId}
            aria-expanded={open}
          >
            {displayName}
          </button>
          <span className="meta">{chatTime(timestamp)}</span>
        </div>
        <p>{body}</p>
      </div>

      {profileId && open && (
        <div className="member-hover-card" role="dialog" aria-label={`${displayName} profile preview`}>
          <div className="member-hover-banner" style={tone} />
          <div className="member-hover-body">
            <span className="member-hover-avatar" style={tone} aria-hidden>
              {personInitials(displayName)}
            </span>
            <div className="member-hover-copy">
              <div className="member-hover-card-name">{displayName}</div>
              <div className="meta">{dept || "Member"}</div>
              <div className="meta">{email}</div>
            </div>
            <div className="member-hover-actions">
              <button type="button" className="btn btn-primary" onClick={openProfile}>
                View profile
              </button>
              {canMessage && onMessage && member && (
                <button
                  type="button"
                  className="btn"
                  onClick={() => {
                    setOpen(false);
                    onMessage(member.id);
                  }}
                >
                  Message
                </button>
              )}
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
