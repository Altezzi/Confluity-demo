import { useState } from "react";
import { useNavigate } from "react-router-dom";
import {
  formatTimestamp,
  playNotificationSound,
  type HomePreference,
  type NotificationSound,
} from "../../lib/preferences";
import { useApp } from "../../context/AppContext";

type SectionId = "notifications" | "appearance" | "workspace";

const SECTIONS: { id: SectionId; label: string }[] = [
  { id: "workspace", label: "Workspace" },
  { id: "notifications", label: "Notifications" },
  { id: "appearance", label: "Appearance" },
];

const SOUNDS: { id: NotificationSound; label: string }[] = [
  { id: "click", label: "Click" },
  { id: "ping", label: "Ping" },
  { id: "step", label: "Step" },
  { id: "rise", label: "Rise" },
  { id: "ring", label: "Ring" },
  { id: "glass", label: "Glass" },
  { id: "off", label: "Off" },
];

const HOMES: { id: HomePreference; label: string }[] = [
  { id: "dashboard", label: "Dashboard" },
  { id: "support", label: "Support" },
  { id: "crm", label: "CRM" },
  { id: "projects", label: "Projects" },
  { id: "inbox", label: "Inbox" },
  { id: "last", label: "Last page I had open" },
];

export function PreferencesSettingsPage() {
  const { can, preferences, setPreference, signOut } = useApp();
  const navigate = useNavigate();
  const [section, setSection] = useState<SectionId>("workspace");
  const [desktopNote, setDesktopNote] = useState<string | null>(null);
  const [confirmLogout, setConfirmLogout] = useState(false);

  if (!can("settings.preferences") && !can("module.settings.view")) {
    return <div className="empty">You do not have permission to view Preferences.</div>;
  }

  async function setDesktopNotifications(on: boolean) {
    setDesktopNote(null);
    if (!on) {
      setPreference("desktopNotifications", false);
      return;
    }
    if (typeof Notification === "undefined") {
      setDesktopNote("This window cannot show desktop notifications.");
      setPreference("desktopNotifications", false);
      return;
    }
    if (Notification.permission === "denied") {
      setDesktopNote("Desktop notifications are blocked for Confluity. Allow them in Windows settings.");
      setPreference("desktopNotifications", false);
      return;
    }
    if (Notification.permission === "default") {
      const result = await Notification.requestPermission();
      if (result !== "granted") {
        setDesktopNote("Desktop notifications stay off until you allow them.");
        setPreference("desktopNotifications", false);
        return;
      }
    }
    setPreference("desktopNotifications", true);
  }

  return (
    <div className="prefs-layout">
      <div className="record-tabs" role="tablist" aria-label="Preference sections">
        {SECTIONS.map((item) => (
          <button
            key={item.id}
            type="button"
            role="tab"
            aria-selected={section === item.id}
            className={section === item.id ? "record-tab active" : "record-tab"}
            onClick={() => setSection(item.id)}
          >
            {item.label}
          </button>
        ))}
      </div>

      {section === "notifications" && (
        <section className="prefs-panel">
          <h2>Notifications</h2>
          <p>Choose what gets your attention, and whether it makes a sound.</p>
          <PrefSwitch
            title="Desktop notifications"
            detail="Show a system alert when something new arrives."
            checked={preferences.desktopNotifications}
            onChange={(on) => void setDesktopNotifications(on)}
          />
          {desktopNote && <p className="prefs-note">{desktopNote}</p>}
          <PrefSwitch
            title="Assignments"
            detail="When a ticket or piece of work is assigned to you."
            checked={preferences.notifyAssignments}
            onChange={(on) => setPreference("notifyAssignments", on)}
          />
          <PrefSwitch
            title="Mentions"
            detail="When someone @mentions you in a comment."
            checked={preferences.notifyMentions}
            onChange={(on) => setPreference("notifyMentions", on)}
          />
          <PrefSwitch
            title="Other activity"
            detail="Workspace updates that are not an assignment or a mention."
            checked={preferences.notifyActivity}
            onChange={(on) => setPreference("notifyActivity", on)}
          />
          <div className="pref-row">
            <div>
              <h3>Notification sound</h3>
              <p>Plays for the alerts you turned on above.</p>
            </div>
            <div className="pref-controls">
              <select
                aria-label="Notification sound"
                value={preferences.notificationSound}
                onChange={(e) => setPreference("notificationSound", e.target.value as NotificationSound)}
              >
                {SOUNDS.map((sound) => (
                  <option key={sound.id} value={sound.id}>
                    {sound.label}
                  </option>
                ))}
              </select>
              <button
                className="btn"
                type="button"
                disabled={preferences.notificationSound === "off"}
                onClick={() => playNotificationSound(preferences.notificationSound)}
              >
                Play
              </button>
            </div>
          </div>
        </section>
      )}

      {section === "appearance" && (
        <section className="prefs-panel">
          <h2>Appearance</h2>
          <p>Theme, density, and how times are written.</p>
          <div className="pref-row">
            <div>
              <h3>Theme</h3>
              <p>Dark is the default. System follows Windows.</p>
            </div>
            <Segment
              ariaLabel="Theme"
              value={preferences.theme}
              options={[
                { id: "dark", label: "Dark" },
                { id: "light", label: "Light" },
                { id: "system", label: "System" },
              ]}
              onChange={(value) => setPreference("theme", value)}
            />
          </div>
          <div className="pref-row">
            <div>
              <h3>Density</h3>
              <p>Compact tightens the sidebar and lists.</p>
            </div>
            <Segment
              ariaLabel="Density"
              value={preferences.density}
              options={[
                { id: "comfortable", label: "Comfortable" },
                { id: "compact", label: "Compact" },
              ]}
              onChange={(value) => setPreference("density", value)}
            />
          </div>
          <div className="pref-row">
            <div>
              <h3>Time format</h3>
              <p>Now: {formatTimestamp(new Date().toISOString(), preferences.timeFormat)}</p>
            </div>
            <Segment
              ariaLabel="Time format"
              value={preferences.timeFormat}
              options={[
                { id: "12", label: "12-hour" },
                { id: "24", label: "24-hour" },
              ]}
              onChange={(value) => setPreference("timeFormat", value)}
            />
          </div>
          <PrefSwitch
            title="Reduce motion"
            detail="Turn off open, close, and page transitions."
            checked={preferences.reduceMotion}
            onChange={(on) => setPreference("reduceMotion", on)}
          />
        </section>
      )}

      {section === "workspace" && (
        <section className="prefs-panel">
          <h2>Workspace</h2>
          <p>What you see when you come back, and whether the assistant starts open.</p>
          <div className="pref-row">
            <div>
              <h3>When you sign in, open</h3>
              <p>You can still open Dashboard from the sidebar.</p>
            </div>
            <select
              aria-label="Page to open when you sign in"
              value={preferences.home}
              onChange={(e) => setPreference("home", e.target.value as HomePreference)}
            >
              {HOMES.map((home) => (
                <option key={home.id} value={home.id}>
                  {home.label}
                </option>
              ))}
            </select>
          </div>
          <PrefSwitch
            title="Open the assistant on launch"
            detail="Hiding it from the top bar lasts until the next launch."
            checked={preferences.openAssistantOnLaunch}
            onChange={(on) => setPreference("openAssistantOnLaunch", on)}
          />
          <div className="pref-row">
            <div>
              <h3>Log out of workspace</h3>
              <p>Leave this sign-in on this machine. You can sign in again with the same account.</p>
            </div>
            <button className="btn" type="button" onClick={() => setConfirmLogout(true)}>
              Log out
            </button>
          </div>
        </section>
      )}
      {confirmLogout && (
        <div className="modal-backdrop" onClick={() => setConfirmLogout(false)}>
          <div
            className="modal"
            role="dialog"
            aria-labelledby="logout-title"
            onClick={(e) => e.stopPropagation()}
          >
            <div className="modal-head">
              <h2 id="logout-title">Log out of this workspace?</h2>
            </div>
            <p className="meta" style={{ padding: "0 18px" }}>
              You will need to sign in again to open it on this machine.
            </p>
            <div className="modal-actions" style={{ padding: "16px 18px 18px" }}>
              <button className="btn" type="button" onClick={() => setConfirmLogout(false)}>
                Cancel
              </button>
              <button
                className="btn btn-primary"
                type="button"
                onClick={() => {
                  signOut();
                  navigate("/login", { replace: true });
                }}
              >
                Log out
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}

function PrefSwitch({
  title,
  detail,
  checked,
  onChange,
}: {
  title: string;
  detail: string;
  checked: boolean;
  onChange: (on: boolean) => void;
}) {
  return (
    <div className="pref-row">
      <div>
        <h3>{title}</h3>
        <p>{detail}</p>
      </div>
      <button
        type="button"
        className="pref-switch"
        role="switch"
        aria-checked={checked}
        aria-label={title}
        onClick={() => onChange(!checked)}
      >
        <span />
      </button>
    </div>
  );
}

function Segment<T extends string>({
  ariaLabel,
  value,
  options,
  onChange,
}: {
  ariaLabel: string;
  value: T;
  options: { id: T; label: string }[];
  onChange: (value: T) => void;
}) {
  return (
    <div className="pref-segment" role="group" aria-label={ariaLabel}>
      {options.map((option) => (
        <button
          key={option.id}
          type="button"
          aria-pressed={value === option.id}
          className={value === option.id ? "active" : ""}
          onClick={() => onChange(option.id)}
        >
          {option.label}
        </button>
      ))}
    </div>
  );
}
