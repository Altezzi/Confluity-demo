import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useRef,
  useState,
  type ReactNode,
} from "react";
import type { AppSettings, RecordType, WorkspaceModule } from "@confluity/shared";
import { hasPermission, isAlwaysOnModule } from "@confluity/shared";
import { api } from "../lib/api";
import {
  applyPreferences,
  loadPreferences,
  savePreferences,
  type UserPreferences,
} from "../lib/preferences";

interface FocusContext {
  entityType: RecordType | null;
  entityId: string | null;
  label: string | null;
}

interface AppContextValue {
  settings: AppSettings | null;
  ready: boolean;
  error: string | null;
  aiOpen: boolean;
  setAiOpen: (open: boolean) => void;
  searchOpen: boolean;
  setSearchOpen: (open: boolean) => void;
  focus: FocusContext;
  setFocus: (focus: FocusContext) => void;
  recordPulse: number;
  notifyRecordChange: () => void;
  refreshSettings: () => Promise<void>;
  myPermissions: string[];
  refreshPermissions: () => Promise<void>;
  can: (key: string) => boolean;
  workspaceModules: WorkspaceModule[];
  refreshModules: () => Promise<void>;
  moduleEnabled: (moduleId: string) => boolean;
  moduleMode: (moduleId: string) => string;
  signedIn: boolean;
  mustSetPassword: boolean;
  signIn: (next?: { mustSetPassword?: boolean }) => void;
  signOut: () => void;
  preferences: UserPreferences;
  setPreference: <K extends keyof UserPreferences>(key: K, value: UserPreferences[K]) => void;
}

const AppContext = createContext<AppContextValue | null>(null);

export function AppProvider({ children }: { children: ReactNode }) {
  const [settings, setSettings] = useState<AppSettings | null>(null);
  const [ready, setReady] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [preferences, setPreferences] = useState(() => loadPreferences());
  const [aiOpen, setAiOpen] = useState(() => loadPreferences().openAssistantOnLaunch);
  const [searchOpen, setSearchOpen] = useState(false);
  const [focus, setFocus] = useState<FocusContext>({
    entityType: null,
    entityId: null,
    label: null,
  });
  const [recordPulse, setRecordPulse] = useState(0);
  const [myPermissions, setMyPermissions] = useState<string[]>([]);
  const [workspaceModules, setWorkspaceModules] = useState<WorkspaceModule[]>([]);
  const [signedIn, setSignedIn] = useState(true);
  const [mustSetPassword, setMustSetPassword] = useState(false);
  const notifyRecordChange = useCallback(() => {
    setRecordPulse((n) => n + 1);
  }, []);

  const refreshPermissions = useCallback(async () => {
    try {
      const perms = await api.getMyPermissions();
      setMyPermissions(perms);
    } catch {
      setMyPermissions([]);
    }
  }, []);

  const refreshModules = useCallback(async () => {
    try {
      setWorkspaceModules(await api.listWorkspaceModules());
    } catch {
      setWorkspaceModules([]);
    }
  }, []);

  const refreshSettings = useCallback(async () => {
    const s = await api.getSettings();
    setSettings(s);
    await Promise.all([refreshPermissions(), refreshModules()]);
  }, [refreshPermissions, refreshModules]);

  useEffect(() => {
    (async () => {
      try {
        await api.ensureSeeded();
        await refreshSettings();
        setReady(true);
      } catch (e) {
        setError(e instanceof Error ? e.message : String(e));
      }
    })();
  }, [refreshSettings]);

  const appliedMember = useRef<string | null>(null);

  useEffect(() => {
    const memberId = settings?.current_member_id;
    if (!memberId || appliedMember.current === memberId) return;
    appliedMember.current = memberId;
    const next = loadPreferences(memberId);
    setPreferences(next);
    applyPreferences(next);
    setAiOpen(next.openAssistantOnLaunch);
  }, [settings?.current_member_id]);

  const setPreference = useCallback(
    <K extends keyof UserPreferences>(key: K, value: UserPreferences[K]) => {
      setPreferences((prev) => {
        const next = { ...prev, [key]: value };
        savePreferences(next, settings?.current_member_id);
        applyPreferences(next);
        return next;
      });
    },
    [settings?.current_member_id],
  );

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === "k") {
        e.preventDefault();
        setSearchOpen(true);
      }
      if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === "j") {
        e.preventDefault();
        setAiOpen((v) => !v);
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, []);

  const can = useCallback(
    (key: string) => hasPermission(myPermissions, key),
    [myPermissions],
  );

  const moduleEnabled = useCallback(
    (moduleId: string) => {
      if (isAlwaysOnModule(moduleId)) return true;
      const row = workspaceModules.find((m) => m.module_id === moduleId);
      if (!row) return true;
      return row.enabled;
    },
    [workspaceModules],
  );

  const moduleMode = useCallback(
    (moduleId: string) => {
      const row = workspaceModules.find((m) => m.module_id === moduleId);
      return row?.mode ?? "native";
    },
    [workspaceModules],
  );

  const value = useMemo(
    () => ({
      settings,
      ready,
      error,
      aiOpen,
      setAiOpen,
      searchOpen,
      setSearchOpen,
      focus,
      setFocus,
      recordPulse,
      notifyRecordChange,
      refreshSettings,
      myPermissions,
      refreshPermissions,
      can,
      workspaceModules,
      refreshModules,
      moduleEnabled,
      moduleMode,
      signedIn,
      mustSetPassword,
      signIn: (next?: { mustSetPassword?: boolean }) => {
        setSignedIn(true);
        setMustSetPassword(Boolean(next?.mustSetPassword));
      },
      signOut: () => {
        setSignedIn(false);
        setMustSetPassword(false);
      },
      preferences,
      setPreference,
    }),
    [
      settings,
      ready,
      error,
      aiOpen,
      searchOpen,
      focus,
      recordPulse,
      notifyRecordChange,
      refreshSettings,
      myPermissions,
      refreshPermissions,
      can,
      workspaceModules,
      refreshModules,
      moduleEnabled,
      moduleMode,
      signedIn,
      mustSetPassword,
      preferences,
      setPreference,
    ],
  );

  return <AppContext.Provider value={value}>{children}</AppContext.Provider>;
}

export function useApp() {
  const ctx = useContext(AppContext);
  if (!ctx) throw new Error("useApp must be used within AppProvider");
  return ctx;
}
