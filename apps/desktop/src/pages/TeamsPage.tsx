import { useEffect, useMemo, useRef, useState, type FormEvent } from "react";
import { Link, useSearchParams } from "react-router-dom";
import type { DmMessage, DmThread, Member, OrgCategory, Position, Team, TeamChannel, TeamMessage } from "@confluity/shared";
import { api } from "../lib/api";
import { avatarTone, memberPlacement, personInitials } from "../components/PersonChip";
import { MessageAuthor } from "../components/MemberHoverCard";
import { useApp } from "../context/AppContext";

type Selection =
  | { kind: "channel"; teamId: string; channelId: string }
  | { kind: "dm"; id: string }
  | null;

function RailChevron({ open }: { open: boolean }) {
  return (
    <span className={`teams-rail-chevron${open ? " open" : ""}`} aria-hidden>
      <svg viewBox="0 0 12 12" width="12" height="12">
        <path
          d="M4.2 2.4 7.8 6 4.2 9.6"
          fill="none"
          stroke="currentColor"
          strokeWidth="1.5"
          strokeLinecap="round"
          strokeLinejoin="round"
        />
      </svg>
    </span>
  );
}

export function TeamsPage() {
  const { settings, setFocus, can } = useApp();
  const [searchParams, setSearchParams] = useSearchParams();
  const [teams, setTeams] = useState<Team[]>([]);
  const [members, setMembers] = useState<Member[]>([]);
  const [categories, setCategories] = useState<OrgCategory[]>([]);
  const [positions, setPositions] = useState<Position[]>([]);
  const [dms, setDms] = useState<DmThread[]>([]);
  const [selection, setSelection] = useState<Selection>(null);
  const [teamMessages, setTeamMessages] = useState<TeamMessage[]>([]);
  const [dmMessages, setDmMessages] = useState<DmMessage[]>([]);
  const [draft, setDraft] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [showEmoji, setShowEmoji] = useState(false);
  const [showSendMenu, setShowSendMenu] = useState(false);
  const [formatHints, setFormatHints] = useState(false);
  const [openSections, setOpenSections] = useState({
    all: true,
    dms: true,
    coworkers: true,
  });
  const [expandedTeams, setExpandedTeams] = useState<Record<string, boolean>>({});
  const [canCreateChats, setCanCreateChats] = useState(false);
  const [creating, setCreating] = useState(false);
  const [newChatName, setNewChatName] = useState("");
  const [newChatTeamId, setNewChatTeamId] = useState("all");
  const [newChatMembers, setNewChatMembers] = useState<string[]>([]);
  const bottomRef = useRef<HTMLDivElement | null>(null);
  const composerRef = useRef<HTMLTextAreaElement | null>(null);
  const fileInputRef = useRef<HTMLInputElement | null>(null);

  function toggleSection(key: keyof typeof openSections) {
    setOpenSections((curr) => ({ ...curr, [key]: !curr[key] }));
  }

  function insertInDraft(text: string, wrapSelection?: { before: string; after: string }) {
    const el = composerRef.current;
    if (!el) {
      setDraft((d) => d + text);
      return;
    }
    const start = el.selectionStart ?? draft.length;
    const end = el.selectionEnd ?? draft.length;
    const selected = draft.slice(start, end);
    const next =
      wrapSelection != null
        ? draft.slice(0, start) +
          wrapSelection.before +
          (selected || "text") +
          wrapSelection.after +
          draft.slice(end)
        : draft.slice(0, start) + text + draft.slice(end);
    setDraft(next);
    requestAnimationFrame(() => {
      el.focus();
      const cursor =
        wrapSelection != null
          ? start + wrapSelection.before.length + (selected || "text").length + wrapSelection.after.length
          : start + text.length;
      el.setSelectionRange(cursor, cursor);
    });
  }

  const meId = settings?.current_member_id ?? members[0]?.id ?? null;
  const me = members.find((m) => m.id === meId) ?? null;
  const mePosition = positions.find((p) => p.id === me?.position_id) ?? null;
  const meCategory = categories.find((c) => c.id === mePosition?.category_id) ?? null;

  const myDeptTeams = useMemo(
    () => teams.filter((t) => (t.kind || "department") !== "chat"),
    [teams],
  );
  const allTeams = useMemo(
    () =>
      [...teams].sort((a, b) => {
        const ac = (a.kind || "department") === "chat" ? 1 : 0;
        const bc = (b.kind || "department") === "chat" ? 1 : 0;
        if (ac !== bc) return ac - bc;
        return a.name.localeCompare(b.name);
      }),
    [teams],
  );

  function positionDepth(id: string | null | undefined): number {
    if (!id) return 99;
    let depth = 0;
    let cur = positions.find((p) => p.id === id);
    const seen = new Set<string>();
    while (cur?.parent_id && !seen.has(cur.id)) {
      seen.add(cur.id);
      depth += 1;
      cur = positions.find((p) => p.id === cur!.parent_id);
    }
    return depth;
  }

  const deptPeople = useMemo(() => {
    if (!me) return [];
    if (!meCategory) {
      return members.filter((m) => m.id !== me.id && !m.position_id);
    }
    return members
      .filter((m) => {
        if (m.id === me.id) return false;
        const pos = positions.find((p) => p.id === m.position_id);
        return pos?.category_id === meCategory.id;
      })
      .sort((a, b) => {
        const da = positionDepth(a.position_id);
        const db = positionDepth(b.position_id);
        if (da !== db) return da - db;
        return a.name.localeCompare(b.name);
      });
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [members, me, meCategory, positions]);

  async function reloadLists() {
    const [t, m, d, cats, pos] = await Promise.all([
      api.listTeams(),
      api.listMembers(),
      api.listDmThreads().catch(() => [] as DmThread[]),
      api.listOrgCategories().catch(() => [] as OrgCategory[]),
      api.listPositions().catch(() => [] as Position[]),
    ]);
    setTeams(t);
    setMembers(m);
    setDms(d);
    setCategories(cats);
    setPositions(pos);
    try {
      setCanCreateChats(await api.canCreateTeamChats());
    } catch {
      setCanCreateChats(false);
    }
  }

  useEffect(() => {
    setFocus({ entityType: null, entityId: null, label: "Teams" });
    void reloadLists().catch((e) => setError(e instanceof Error ? e.message : String(e)));
  }, [setFocus]);

  // Deep-link: ?dm=memberId or ?team=teamId
  useEffect(() => {
    const dmMember = searchParams.get("dm");
    const teamId = searchParams.get("team");
    if (dmMember && can("teams.message")) {
      void (async () => {
        try {
          const thread = await api.openDm(dmMember);
          await reloadLists();
          setSelection({ kind: "dm", id: thread.id });
          setSearchParams({}, { replace: true });
        } catch (e) {
          setError(e instanceof Error ? e.message : String(e));
        }
      })();
    } else if (teamId) {
      const team = teams.find((t) => t.id === teamId);
      if (!team) return;
      const general =
        (team.channels ?? []).find((c) => c.name.toLowerCase() === "general") ??
        team.channels?.[0];
      if (general) setSelection({ kind: "channel", teamId, channelId: general.id });
      setSearchParams({}, { replace: true });
    }
  }, [searchParams, can, setSearchParams, teams]);

  useEffect(() => {
    if (!selection) {
      setTeamMessages([]);
      setDmMessages([]);
      return;
    }
    void (async () => {
      try {
        if (selection.kind === "channel") {
          setTeamMessages(await api.listTeamMessages(selection.teamId, selection.channelId));
          setDmMessages([]);
        } else {
          setDmMessages(await api.listDmMessages(selection.id));
          setTeamMessages([]);
        }
        setError(null);
      } catch (e) {
        setError(e instanceof Error ? e.message : String(e));
      }
    })();
  }, [selection]);

  useEffect(() => {
    bottomRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [teamMessages, dmMessages]);

  const activeTeam =
    selection?.kind === "channel" ? teams.find((t) => t.id === selection.teamId) ?? null : null;
  const activeChannel =
    selection?.kind === "channel"
      ? (activeTeam?.channels ?? []).find((c) => c.id === selection.channelId) ?? null
      : null;
  const activeDm = selection?.kind === "dm" ? dms.find((d) => d.id === selection.id) : null;

  function channelsOf(team: Team): TeamChannel[] {
    return team.channels ?? [];
  }

  function selectChannel(team: Team, channel?: TeamChannel | null) {
    const ch = channel ?? channelsOf(team).find((c) => c.name.toLowerCase() === "general") ?? channelsOf(team)[0];
    if (!ch) return;
    setExpandedTeams((curr) => ({ ...curr, [team.id]: true }));
    setSelection({ kind: "channel", teamId: team.id, channelId: ch.id });
  }

  async function createChat() {
    if (!newChatName.trim() || busy || !canCreateChats) return;
    setBusy(true);
    setError(null);
    try {
      if (newChatTeamId === "all") {
        const members = new Set(newChatMembers);
        if (meId) members.add(meId);
        const team = await api.createGroupChat({
          name: newChatName.trim(),
          member_ids: [...members],
        });
        await reloadLists();
        selectChannel(team);
      } else {
        const channel = await api.createTeamChannel({
          team_id: newChatTeamId,
          name: newChatName.trim(),
        });
        await reloadLists();
        setSelection({ kind: "channel", teamId: newChatTeamId, channelId: channel.id });
        setExpandedTeams((curr) => ({ ...curr, [newChatTeamId]: true }));
      }
      setNewChatName("");
      setCreating(false);
      setNewChatMembers([]);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  }

  async function send(e: FormEvent) {
    e.preventDefault();
    if (!selection || !draft.trim() || busy || !can("teams.message")) return;
    setBusy(true);
    setError(null);
    try {
      if (selection.kind === "channel") {
        const msg = await api.sendTeamMessage(selection.teamId, draft.trim(), selection.channelId);
        setTeamMessages((curr) => [...curr, msg]);
      } else {
        const msg = await api.sendDmMessage(selection.id, draft.trim());
        setDmMessages((curr) => [...curr, msg]);
        await reloadLists();
      }
      setDraft("");
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setBusy(false);
    }
  }

  async function startDm(memberId: string) {
    if (!can("teams.message")) return;
    setBusy(true);
    setError(null);
    try {
      const thread = await api.openDm(memberId);
      await reloadLists();
      setSelection({ kind: "dm", id: thread.id });
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  }

  const feed =
    selection?.kind === "channel"
      ? teamMessages.map((m) => ({
          id: m.id,
          author: m.author_name,
          authorId: m.author_id,
          body: m.body,
          at: m.created_at,
        }))
      : dmMessages.map((m) => ({
          id: m.id,
          author: m.author_name,
          authorId: m.author_id,
          body: m.body,
          at: m.created_at,
        }));

  return (
    <div className="page page-wide teams-page">
      <div className="page-header">
        <div>
          <h1>Teams</h1>
          <p>Category teams from your position, plus any chats you were invited to.</p>
        </div>
        {canCreateChats && (
          <button className="btn btn-primary" type="button" onClick={() => setCreating((v) => !v)}>
            {creating ? "Cancel" : "New chat"}
          </button>
        )}
      </div>

      {error && <p style={{ color: "var(--danger)" }}>{error}</p>}

      {creating && canCreateChats && (
        <div className="panel teams-create-chat">
          <div className="nav-label">New chat</div>
          <div className="form-grid" style={{ gap: 10 }}>
            <div className="form-row">
              <label>Name</label>
              <input
                value={newChatName}
                onChange={(e) => setNewChatName(e.target.value)}
                placeholder="e.g. shift-handoff"
                onKeyDown={(e) => {
                  if (e.key === "Enter") {
                    e.preventDefault();
                    void createChat();
                  }
                }}
              />
            </div>
            <div className="form-row">
              <label>Where</label>
              <select value={newChatTeamId} onChange={(e) => setNewChatTeamId(e.target.value)}>
                <option value="all">All (invite-only chat)</option>
                {myDeptTeams.map((t) => (
                  <option key={t.id} value={t.id}>
                    {t.name} team
                  </option>
                ))}
              </select>
            </div>
            {newChatTeamId === "all" && (
              <div className="form-row">
                <label>Members</label>
                <div className="teams-member-row" style={{ padding: 0 }}>
                  {members
                    .filter((m) => m.id !== meId)
                    .map((m) => {
                      const on = newChatMembers.includes(m.id);
                      return (
                        <button
                          key={m.id}
                          type="button"
                          className={`teams-member-chip${on ? " on" : ""}`}
                          onClick={() =>
                            setNewChatMembers((curr) =>
                              on ? curr.filter((id) => id !== m.id) : [...curr, m.id],
                            )
                          }
                        >
                          {m.name}
                        </button>
                      );
                    })}
                </div>
              </div>
            )}
            <button
              className="btn btn-primary"
              type="button"
              disabled={busy || !newChatName.trim()}
              onClick={() => void createChat()}
            >
              Create chat
            </button>
          </div>
        </div>
      )}

      <div className="teams-shell panel">
        <aside className="teams-rail">
          <div className="teams-rail-section">
            <button
              type="button"
              className="teams-rail-heading"
              aria-expanded={openSections.all}
              onClick={() => toggleSection("all")}
            >
              <RailChevron open={openSections.all} />
              All
            </button>
            {openSections.all && (
              <>
                {allTeams.map((t) => {
                  const open = expandedTeams[t.id] ?? true;
                  const chans = channelsOf(t);
                  return (
                    <div key={t.id} className="teams-rail-group">
                      <button
                        type="button"
                        className="teams-rail-item teams-rail-team"
                        onClick={() => {
                          setExpandedTeams((curr) => ({ ...curr, [t.id]: !open }));
                          if (!open || !chans.length) selectChannel(t);
                        }}
                      >
                        <RailChevron open={open} />
                        <span className="teams-team-mark" style={{ background: t.color }} />
                        <span className="teams-rail-label">{t.name}</span>
                      </button>
                      {open &&
                        chans.map((ch) => (
                          <button
                            key={ch.id}
                            type="button"
                            className={`teams-rail-item nested${
                              selection?.kind === "channel" && selection.channelId === ch.id ? " active" : ""
                            }`}
                            onClick={() => selectChannel(t, ch)}
                          >
                            <span className="teams-hash" style={{ color: t.color }}>
                              #
                            </span>
                            <span className="teams-rail-label">{ch.name}</span>
                          </button>
                        ))}
                    </div>
                  );
                })}
                {allTeams.length === 0 && (
                  <div className="teams-rail-note">No category teams yet — assign a position in Admin</div>
                )}
              </>
            )}
          </div>

          <div className="teams-rail-section">
            <button
              type="button"
              className="teams-rail-heading"
              aria-expanded={openSections.dms}
              onClick={() => toggleSection("dms")}
            >
              <RailChevron open={openSections.dms} />
              Direct messages
            </button>
            {openSections.dms && (
              <>
                {dms.map((d) => (
                  <button
                    key={d.id}
                    type="button"
                    className={`teams-rail-item${selection?.kind === "dm" && selection.id === d.id ? " active" : ""}`}
                    onClick={() => setSelection({ kind: "dm", id: d.id })}
                  >
                    <span className="teams-person-avatar sm" style={avatarTone(d.other_member_name)} aria-hidden>
                      {personInitials(d.other_member_name)}
                    </span>
                    <span className="teams-rail-label">{d.other_member_name}</span>
                  </button>
                ))}
                {dms.length === 0 && <div className="teams-rail-note">No conversations yet</div>}
              </>
            )}
          </div>

          <div className="teams-rail-section">
            <button
              type="button"
              className="teams-rail-heading"
              aria-expanded={openSections.coworkers}
              onClick={() => toggleSection("coworkers")}
            >
              <RailChevron open={openSections.coworkers} />
              {meCategory ? `${meCategory.name}` : "No department set"}
            </button>
            {openSections.coworkers && (
              <>
                {deptPeople.map((m) => {
                  const pos = positions.find((p) => p.id === m.position_id);
                  const cat = categories.find((c) => c.id === pos?.category_id);
                  return (
                  <button
                    key={m.id}
                    type="button"
                    className="teams-rail-item"
                    disabled={!can("teams.message") || busy}
                    onClick={() => void startDm(m.id)}
                    title={`Message ${m.name}`}
                  >
                    <span className="teams-person-avatar" style={avatarTone(m.name)} aria-hidden>
                      {personInitials(m.name)}
                    </span>
                    <span className="teams-person-copy">
                      <span>{m.name}</span>
                      <span className="meta">
                        {memberPlacement(m, cat?.name, pos?.name) || "No position"}
                      </span>
                    </span>
                  </button>
                  );
                })}
                {deptPeople.length === 0 && (
                  <div className="teams-rail-note">
                    {meCategory
                      ? "No one else in your department"
                      : "Assign a position in Admin → Members"}
                  </div>
                )}
              </>
            )}
          </div>
        </aside>

        <section className="teams-main">
          {!selection && (
            <div className="teams-empty">
              <div className="teams-empty-mark" aria-hidden>
                <svg viewBox="0 0 24 24" width="26" height="26" fill="none" stroke="currentColor" strokeWidth="1.6">
                  <path d="M5 6.5A2.5 2.5 0 0 1 7.5 4h9A2.5 2.5 0 0 1 19 6.5v7A2.5 2.5 0 0 1 16.5 16H9l-4 3.5V6.5Z" strokeLinejoin="round" />
                </svg>
              </div>
              <h2>Choose a conversation</h2>
              <p>Pick a team channel, or message someone in your department.</p>
            </div>
          )}

          {activeTeam && (
            <div className="teams-channel-head">
              <div className="teams-channel-title">
                <span className="teams-hash-lg" style={{ color: activeTeam.color }}>
                  #
                </span>
                <div>
                  <h2>{activeChannel?.name || activeTeam.name}</h2>
                  <p className="meta">
                    {activeChannel?.description || activeTeam.description || activeTeam.department || "Team chat"}
                    {" · "}
                    {activeTeam.name}
                    {" · "}
                    {activeTeam.member_ids.length} members
                  </p>
                </div>
              </div>
              <div className="teams-facepile" aria-label="Channel members">
                {activeTeam.member_ids.slice(0, 6).map((id) => {
                  const m = members.find((x) => x.id === id);
                  if (!m) return null;
                  const pos = positions.find((p) => p.id === m.position_id);
                  return (
                    <button
                      key={id}
                      type="button"
                      className="teams-face"
                      title={pos ? `${m.name} · ${pos.name}` : m.name}
                      disabled={!can("teams.message") || m.id === meId}
                      onClick={() => void startDm(m.id)}
                    >
                      <span style={avatarTone(m.name)} className="teams-person-avatar sm">
                        {personInitials(m.name)}
                      </span>
                    </button>
                  );
                })}
                {activeTeam.member_ids.length > 6 && (
                  <span className="teams-face-more">+{activeTeam.member_ids.length - 6}</span>
                )}
              </div>
            </div>
          )}

          {activeDm && (
            <div className="teams-channel-head">
              <div className="teams-channel-title">
                <span className="teams-person-avatar" style={avatarTone(activeDm.other_member_name)} aria-hidden>
                  {personInitials(activeDm.other_member_name)}
                </span>
                <div>
                  <h2>{activeDm.other_member_name}</h2>
                  <p className="meta">Direct message</p>
                </div>
              </div>
              <Link className="btn" to="/profile">
                Your profile
              </Link>
            </div>
          )}

          {selection && (
            <>
              <div className="teams-feed">
                {feed.length === 0 && (
                  <div className="teams-thread-start">
                    {activeDm ? (
                      <span className="teams-person-avatar" style={avatarTone(activeDm.other_member_name)}>
                        {personInitials(activeDm.other_member_name)}
                      </span>
                    ) : (
                      <span className="teams-hash-lg" style={{ color: activeTeam?.color }}>
                        #
                      </span>
                    )}
                    <h3>
                      {activeDm
                        ? `This is the start of your conversation with ${activeDm.other_member_name}`
                        : `This is the start of #${activeChannel?.name || activeTeam?.name || "channel"}`}
                    </h3>
                    <p>
                      {activeChannel?.description ||
                        activeTeam?.description ||
                        "Say hello to get things moving."}
                    </p>
                  </div>
                )}
                {feed.map((m, index) => {
                  const author = members.find((x) => x.id === m.authorId) ?? null;
                  const pos = positions.find((p) => p.id === author?.position_id);
                  const cat = categories.find((c) => c.id === pos?.category_id);
                  const prev = index > 0 ? feed[index - 1] : null;
                  const gap = prev ? new Date(m.at).getTime() - new Date(prev.at).getTime() : Number.POSITIVE_INFINITY;
                  const continued = !!prev && prev.authorId === m.authorId && gap >= 0 && gap < 5 * 60 * 1000;
                  return (
                  <MessageAuthor
                    key={m.id}
                    member={author}
                    name={m.author}
                    memberId={m.authorId}
                    organizationName={settings?.organization_name}
                    placement={memberPlacement(author, cat?.name, pos?.name)}
                    timestamp={m.at}
                    body={m.body}
                    mine={m.authorId === meId}
                    continued={continued}
                    canMessage={can("teams.message") && m.authorId !== meId}
                    onMessage={(id) => void startDm(id)}
                  />
                  );
                })}
                <div ref={bottomRef} />
              </div>
              <form
                className="teams-composer"
                onSubmit={(e) => {
                  setShowEmoji(false);
                  setShowSendMenu(false);
                  void send(e);
                }}
              >
                <input
                  ref={fileInputRef}
                  type="file"
                  hidden
                  multiple
                  onChange={(e) => {
                    const names = Array.from(e.target.files ?? []).map((f) => f.name);
                    if (names.length) insertInDraft(names.map((n) => `[file: ${n}]`).join(" ") + " ");
                    e.target.value = "";
                  }}
                />
                <div className="teams-composer-box">
                  {formatHints && (
                    <div className="teams-composer-hints">
                      <button type="button" onClick={() => insertInDraft("", { before: "**", after: "**" })}>
                        Bold
                      </button>
                      <button type="button" onClick={() => insertInDraft("", { before: "_", after: "_" })}>
                        Italic
                      </button>
                      <button type="button" onClick={() => insertInDraft("", { before: "`", after: "`" })}>
                        Code
                      </button>
                      <button type="button" onClick={() => insertInDraft("- ")}>
                        List
                      </button>
                    </div>
                  )}
                  <textarea
                    ref={composerRef}
                    rows={2}
                    value={draft}
                    onChange={(e) => setDraft(e.target.value)}
                    onKeyDown={(e) => {
                      if (e.key === "Enter" && !e.shiftKey) {
                        e.preventDefault();
                        if (draft.trim() && can("teams.message") && !busy) {
                          e.currentTarget.form?.requestSubmit();
                        }
                      }
                    }}
                    placeholder={
                      can("teams.message")
                        ? activeTeam
                          ? `Message #${activeChannel?.name || activeTeam.name}`
                          : "Write a direct message"
                        : "Messaging disabled for your role"
                    }
                    disabled={!can("teams.message") || busy}
                  />
                  <div className="teams-composer-toolbar">
                    <div className="teams-composer-tools">
                      <button
                        type="button"
                        className="teams-composer-icon teams-composer-plus"
                        title="Upload a file"
                        disabled={!can("teams.message") || busy}
                        onClick={() => fileInputRef.current?.click()}
                      >
                        <svg viewBox="0 0 24 24" width="18" height="18" fill="none" stroke="currentColor" strokeWidth="2">
                          <path d="M12 5v14M5 12h14" />
                        </svg>
                      </button>
                      <span className="teams-composer-sep" aria-hidden />
                      <button
                        type="button"
                        className={`teams-composer-icon${formatHints ? " active" : ""}`}
                        title="Formatting"
                        disabled={!can("teams.message") || busy}
                        onClick={() => setFormatHints((v) => !v)}
                      >
                        Aa
                      </button>
                      <div className="teams-composer-pop">
                        <button
                          type="button"
                          className={`teams-composer-icon${showEmoji ? " active" : ""}`}
                          title="Emoji"
                          disabled={!can("teams.message") || busy}
                          onClick={() => {
                            setShowSendMenu(false);
                            setShowEmoji((v) => !v);
                          }}
                        >
                          <svg viewBox="0 0 24 24" width="18" height="18" fill="none" stroke="currentColor" strokeWidth="1.75">
                            <circle cx="12" cy="12" r="9" />
                            <path d="M8.5 10h.01M15.5 10h.01M8.5 15c1.2 1.2 2.6 1.8 3.5 1.8s2.3-.6 3.5-1.8" />
                          </svg>
                        </button>
                        {showEmoji && (
                          <div className="teams-emoji-panel" role="listbox">
                            {["😀", "🙂", "😂", "🙌", "👍", "🔥", "🎉", "✅", "❗", "💡"].map((em) => (
                              <button
                                key={em}
                                type="button"
                                onClick={() => {
                                  insertInDraft(em);
                                  setShowEmoji(false);
                                }}
                              >
                                {em}
                              </button>
                            ))}
                          </div>
                        )}
                      </div>
                      <button
                        type="button"
                        className="teams-composer-icon"
                        title="Mention someone"
                        disabled={!can("teams.message") || busy}
                        onClick={() => insertInDraft("@")}
                      >
                        @
                      </button>
                      <span className="teams-composer-sep" aria-hidden />
                      <button
                        type="button"
                        className="teams-composer-icon"
                        title="Start a video clip (coming soon)"
                        disabled={!can("teams.message") || busy}
                        onClick={() => setError("Video messages will arrive with Confluity Server.")}
                      >
                        <svg viewBox="0 0 24 24" width="18" height="18" fill="none" stroke="currentColor" strokeWidth="1.75">
                          <rect x="3" y="7" width="12" height="10" rx="2" />
                          <path d="M15 10.5 21 7v10l-6-3.5" />
                        </svg>
                      </button>
                      <button
                        type="button"
                        className="teams-composer-icon"
                        title="Record voice (coming soon)"
                        disabled={!can("teams.message") || busy}
                        onClick={() => setError("Voice messages will arrive with Confluity Server.")}
                      >
                        <svg viewBox="0 0 24 24" width="18" height="18" fill="none" stroke="currentColor" strokeWidth="1.75">
                          <rect x="9" y="3" width="6" height="11" rx="3" />
                          <path d="M5 11a7 7 0 0 0 14 0M12 18v3" />
                        </svg>
                      </button>
                      <span className="teams-composer-sep" aria-hidden />
                      <button
                        type="button"
                        className="teams-composer-icon"
                        title="Slash commands"
                        disabled={!can("teams.message") || busy}
                        onClick={() => insertInDraft("/")}
                      >
                        <svg viewBox="0 0 24 24" width="18" height="18" fill="none" stroke="currentColor" strokeWidth="1.75">
                          <rect x="4" y="4" width="16" height="16" rx="3" />
                          <path d="M9 16 15 8" />
                        </svg>
                      </button>
                    </div>
                    <div className="teams-composer-send">
                      <button
                        className="teams-composer-send-main"
                        type="submit"
                        disabled={!can("teams.message") || busy || !draft.trim()}
                        title="Send message"
                      >
                        <svg viewBox="0 0 24 24" width="18" height="18" fill="currentColor">
                          <path d="M3.4 20.6 21 12 3.4 3.4l2.3 7.3L14 12l-8.3 1.3z" />
                        </svg>
                      </button>
                      <div className="teams-composer-pop">
                        <button
                          type="button"
                          className="teams-composer-send-more"
                          title="More send options"
                          disabled={!can("teams.message") || busy}
                          onClick={() => {
                            setShowEmoji(false);
                            setShowSendMenu((v) => !v);
                          }}
                        >
                          ▾
                        </button>
                        {showSendMenu && (
                          <div className="teams-send-menu" role="menu">
                            <button
                              type="button"
                              role="menuitem"
                              onClick={() => {
                                setShowSendMenu(false);
                                composerRef.current?.form?.requestSubmit();
                              }}
                            >
                              Send now
                            </button>
                            <button
                              type="button"
                              role="menuitem"
                              onClick={() => {
                                setShowSendMenu(false);
                                setError("Scheduled send will arrive with Confluity Server.");
                              }}
                            >
                              Schedule for later
                            </button>
                          </div>
                        )}
                      </div>
                    </div>
                  </div>
                </div>
              </form>
            </>
          )}
        </section>
      </div>
    </div>
  );
}
