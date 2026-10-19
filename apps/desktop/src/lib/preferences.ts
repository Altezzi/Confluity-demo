import type { Notification } from "@confluity/shared";
import clickSound from "../assets/sounds/click.mp3";
import pingSound from "../assets/sounds/ping.mp3";
import stepSound from "../assets/sounds/step.mp3";
import riseSound from "../assets/sounds/rise.mp3";
import ringSound from "../assets/sounds/ring.mp3";
import glassSound from "../assets/sounds/glass.mp3";

export type ThemePreference = "dark" | "light" | "system";
export type DensityPreference = "comfortable" | "compact";
export type TimeFormatPreference = "12" | "24";
export type NotificationSound = "off" | "click" | "ping" | "step" | "rise" | "ring" | "glass";

const NOTIFICATION_SOUNDS: NotificationSound[] = ["off", "click", "ping", "step", "rise", "ring", "glass"];

const SOUND_FILES: Record<Exclude<NotificationSound, "off">, string> = {
  click: clickSound,
  ping: pingSound,
  step: stepSound,
  rise: riseSound,
  ring: ringSound,
  glass: glassSound,
};
export type HomePreference = "dashboard" | "support" | "crm" | "projects" | "inbox" | "last";

export type UserPreferences = {
  theme: ThemePreference;
  density: DensityPreference;
  reduceMotion: boolean;
  timeFormat: TimeFormatPreference;
  desktopNotifications: boolean;
  notificationSound: NotificationSound;
  notifyAssignments: boolean;
  notifyMentions: boolean;
  notifyActivity: boolean;
  openAssistantOnLaunch: boolean;
  home: HomePreference;
};

const DEVICE_KEY = "confluity.preferences.v1";
const LAST_PATH_KEY = "confluity.last-path";

function systemTimeFormat(): TimeFormatPreference {
  try {
    return Intl.DateTimeFormat(undefined, { hour: "numeric" }).resolvedOptions().hour12 === false
      ? "24"
      : "12";
  } catch {
    return "12";
  }
}

export const DEFAULT_PREFERENCES: UserPreferences = {
  theme: "dark",
  density: "comfortable",
  reduceMotion: false,
  timeFormat: systemTimeFormat(),
  desktopNotifications: false,
  notificationSound: "rise",
  notifyAssignments: true,
  notifyMentions: true,
  notifyActivity: false,
  openAssistantOnLaunch: true,
  home: "dashboard",
};

function memberKey(memberId: string): string {
  return `${DEVICE_KEY}.${memberId}`;
}

function parsePreferences(raw: string | null): UserPreferences | null {
  if (!raw) return null;
  try {
    const parsed = JSON.parse(raw) as Partial<UserPreferences>;
    const prefs = { ...DEFAULT_PREFERENCES, ...parsed };
    if (!NOTIFICATION_SOUNDS.includes(prefs.notificationSound)) {
      prefs.notificationSound = DEFAULT_PREFERENCES.notificationSound;
    }
    return prefs;
  } catch {
    return null;
  }
}

export function loadPreferences(memberId?: string | null): UserPreferences {
  const member = memberId ? parsePreferences(localStorage.getItem(memberKey(memberId))) : null;
  return member ?? parsePreferences(localStorage.getItem(DEVICE_KEY)) ?? { ...DEFAULT_PREFERENCES };
}

export function savePreferences(prefs: UserPreferences, memberId?: string | null): void {
  const json = JSON.stringify(prefs);
  localStorage.setItem(DEVICE_KEY, json);
  if (memberId) localStorage.setItem(memberKey(memberId), json);
}

function resolvedTheme(theme: ThemePreference): "dark" | "light" {
  if (theme === "system") {
    return window.matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light";
  }
  return theme;
}

let themeMedia: MediaQueryList | null = null;
let onThemeMedia: (() => void) | null = null;

export function applyPreferences(prefs: UserPreferences): void {
  const root = document.documentElement;
  const theme = resolvedTheme(prefs.theme);
  root.dataset.theme = theme;
  root.dataset.density = prefs.density;
  root.dataset.reduceMotion = prefs.reduceMotion ? "true" : "false";
  root.style.colorScheme = theme;

  if (themeMedia && onThemeMedia) {
    themeMedia.removeEventListener("change", onThemeMedia);
    themeMedia = null;
    onThemeMedia = null;
  }
  if (prefs.theme === "system") {
    themeMedia = window.matchMedia("(prefers-color-scheme: dark)");
    onThemeMedia = () => {
      const next = themeMedia?.matches ? "dark" : "light";
      root.dataset.theme = next;
      root.style.colorScheme = next;
    };
    themeMedia.addEventListener("change", onThemeMedia);
  }
}

export function formatTimestamp(
  iso: string | null | undefined,
  timeFormat: TimeFormatPreference,
): string {
  if (!iso) return "";
  const date = new Date(iso);
  if (Number.isNaN(date.getTime())) return "";
  return date.toLocaleString(undefined, { hour12: timeFormat === "12" });
}

export function rememberPath(pathname: string): void {
  if (!pathname || pathname === "/login" || pathname === "/setup" || pathname === "/set-password") {
    return;
  }
  localStorage.setItem(LAST_PATH_KEY, pathname);
}

export function homePath(prefs: UserPreferences = loadPreferences()): string {
  switch (prefs.home) {
    case "support":
      return "/support";
    case "crm":
      return "/crm";
    case "projects":
      return "/projects";
    case "inbox":
      return "/inbox";
    case "last":
      return localStorage.getItem(LAST_PATH_KEY) || "/";
    default:
      return "/";
  }
}

export type AlertKind = "assignment" | "mention" | "activity";

export function alertKind(note: Notification): AlertKind {
  const title = note.title.toLowerCase();
  if (title.includes("assigned")) return "assignment";
  if (title.includes("mentioned")) return "mention";
  return "activity";
}

export function shouldAlert(prefs: UserPreferences, note: Notification): boolean {
  const kind = alertKind(note);
  if (kind === "assignment") return prefs.notifyAssignments;
  if (kind === "mention") return prefs.notifyMentions;
  return prefs.notifyActivity;
}

let preview: HTMLAudioElement | null = null;

export function playNotificationSound(sound: NotificationSound): void {
  const src = sound === "off" ? null : SOUND_FILES[sound];
  if (!src) return;
  if (preview) {
    preview.pause();
    preview.currentTime = 0;
  }
  const audio = new Audio(src);
  audio.volume = 0.65;
  preview = audio;
  void audio.play().catch(() => {});
}

export function alertForNotifications(prefs: UserPreferences, notes: Notification[]): void {
  const matching = notes.filter((note) => shouldAlert(prefs, note));
  if (matching.length === 0) return;
  playNotificationSound(prefs.notificationSound);
  if (
    !prefs.desktopNotifications ||
    typeof Notification === "undefined" ||
    Notification.permission !== "granted"
  ) {
    return;
  }
  const first = matching[0];
  const extra = matching.length - 1;
  const title = extra > 0 ? `${first.title} (+${extra} more)` : first.title;
  new Notification(title, { body: first.body ?? undefined });
}
