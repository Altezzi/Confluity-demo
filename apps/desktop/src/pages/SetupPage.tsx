import { useCallback, useEffect, useState, type FocusEvent } from "react";
import { useNavigate } from "react-router-dom";
import type { OrgCategory, Position, Role, SetupStatus } from "@confluity/shared";
import { api } from "../lib/api";
import { homePath } from "../lib/preferences";
import { useApp } from "../context/AppContext";
import { TitleBar } from "../components/TitleBar";

const STEPS = [
  { id: "welcome", label: "Start", hint: "A clean workspace" },
  { id: "workspace", label: "Company", hint: "Name your org" },
  { id: "server", label: "Server", hint: "Where data lives" },
  { id: "admin", label: "Admin", hint: "Your first login" },
  { id: "positions", label: "Positions", hint: "Job titles" },
  { id: "done", label: "Ready", hint: "Open the dashboard" },
] as const;

function exampleField(example: string) {
  return {
    placeholder: example,
    onFocus: (event: FocusEvent<HTMLInputElement>) => {
      event.currentTarget.placeholder = "";
    },
    onBlur: (event: FocusEvent<HTMLInputElement>) => {
      if (!event.currentTarget.value) event.currentTarget.placeholder = example;
    },
  };
}

export function SetupPage() {
  const navigate = useNavigate();
  const { refreshSettings, signIn, signedIn } = useApp();
  const [step, setStep] = useState(0);
  const [status, setStatus] = useState<SetupStatus | null>(null);
  const [categories, setCategories] = useState<OrgCategory[]>([]);
  const [positions, setPositions] = useState<Position[]>([]);
  const [defaultRoles, setDefaultRoles] = useState<Role[]>([]);
  const [categoryId, setCategoryId] = useState("");
  const [newCategory, setNewCategory] = useState("");
  const [positionName, setPositionName] = useState("");
  const [positionRoleId, setPositionRoleId] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [clearedDemo, setClearedDemo] = useState(false);

  const [orgName, setOrgName] = useState("");
  const [workspaceName, setWorkspaceName] = useState("");
  const [provider, setProvider] = useState<"local" | "supabase" | "custom_api">("local");
  const [supabaseUrl, setSupabaseUrl] = useState("");
  const [anonKey, setAnonKey] = useState("");
  const [dbPassword, setDbPassword] = useState("");
  const [customUrl, setCustomUrl] = useState("");
  const [adminName, setAdminName] = useState("");
  const [adminEmail, setAdminEmail] = useState("");
  const [adminPassword, setAdminPassword] = useState("");
  const [adminSent, setAdminSent] = useState(false);
  const [verifyPassword, setVerifyPassword] = useState("");
  const [verifyPasswordAgain, setVerifyPasswordAgain] = useState("");

  const load = useCallback(async () => {
    let s = await api.getSetupStatus();
    if (s.demo_data_present && !s.setup_complete) {
      s = await api.clearDemoForSetup();
      setClearedDemo(true);
      await refreshSettings();
    }
    setStatus(s);
    if (s.server_provider === "supabase") setProvider("supabase");
    else if (s.server_provider === "custom_api") setProvider("custom_api");
    else setProvider("local");
    if (s.setup_complete) navigate(signedIn ? homePath() : "/login", { replace: true });
  }, [navigate, refreshSettings, signedIn]);

  useEffect(() => {
    void load().catch((e) => setError(e instanceof Error ? e.message : String(e)));
  }, [load]);

  async function saveWorkspace() {
    setBusy(true);
    setError(null);
    try {
      setStatus(
        await api.saveSetupWorkspace({
          organization_name: orgName,
          workspace_name: workspaceName,
        }),
      );
      await refreshSettings();
      setStep(2);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  }

  async function saveServer() {
    setBusy(true);
    setError(null);
    try {
      const p = provider === "local" ? "none" : provider;
      setStatus(
        await api.saveSetupServer({
          provider: p,
          supabase_url: provider === "supabase" ? supabaseUrl : null,
          supabase_anon_key: provider === "supabase" ? anonKey : null,
          supabase_db_password: provider === "supabase" ? dbPassword : null,
          custom_api_url: provider === "custom_api" ? customUrl : null,
        }),
      );
      await refreshSettings();
      setStep(3);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  }

  async function saveAdmin() {
    setBusy(true);
    setError(null);
    try {
      setStatus(
        await api.saveSetupAdmin({
          name: adminName,
          email: adminEmail,
          password: adminPassword,
        }),
      );
      await refreshSettings();
      setVerifyPassword("");
      setVerifyPasswordAgain("");
      setAdminSent(true);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  }

  async function confirmSavedPassword() {
    if (!verifyPassword || verifyPassword !== adminPassword || verifyPasswordAgain !== adminPassword) {
      setError("Re-enter the same password in both fields.");
      return;
    }
    setBusy(true);
    setError(null);
    try {
      await api.completeSetupRoles(true);
      await loadPositions();
      setStep(4);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  }

  async function loadPositions() {
    const [cats, pos, roleRows] = await Promise.all([
      api.listOrgCategories(),
      api.listPositions(),
      api.listRoles(),
    ]);
    setCategories(cats);
    setPositions(pos);
    setDefaultRoles(roleRows);
    setCategoryId((curr) => (curr && cats.some((c) => c.id === curr) ? curr : cats[0]?.id ?? ""));
  }

  async function addPosition() {
    const title = positionName.trim();
    if (!title) {
      setError("Enter a position title.");
      return;
    }
    setBusy(true);
    setError(null);
    try {
      let catId = categoryId;
      const named = newCategory.trim();
      if (named) {
        const existing = categories.find((c) => c.name.toLowerCase() === named.toLowerCase());
        if (existing) catId = existing.id;
        else {
          const created = await api.createOrgCategory(named);
          catId = created.id;
        }
      }
      if (!catId) {
        setError("Choose a category, or type a new one.");
        return;
      }
      const created = await api.createPosition({ category_id: catId, name: title });
      if (positionRoleId) await api.setPositionRoles(created.id, [positionRoleId]);
      setPositionName("");
      setNewCategory("");
      setCategoryId(catId);
      await loadPositions();
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  }

  async function finishRoles() {
    setBusy(true);
    setError(null);
    try {
      await api.completeSetupRoles(true);
      setStep(5);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  }

  async function finishSetup() {
    setBusy(true);
    setError(null);
    try {
      await api.completeSetup();
      await refreshSettings();
      signIn();
      navigate(homePath(), { replace: true });
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  }

  const current = STEPS[step];

  return (
    <div className="app-frame">
      <TitleBar />
      <div className="setup-shell">
        <aside className="setup-rail">
          <div className="setup-rail-copy">
            <div className="setup-kicker">First-run setup</div>
            <h1>Build the workspace your company will actually use.</h1>
          </div>
          <ol className="setup-steps">
            {STEPS.map((item, index) => (
              <li
                key={item.id}
                className={`setup-step${index === step ? " active" : ""}${index < step ? " done" : ""}`}
              >
                <span className="setup-step-index">{index < step ? "✓" : index + 1}</span>
                <span>
                  <strong>{item.label}</strong>
                  <em>{item.hint}</em>
                </span>
              </li>
            ))}
          </ol>
        </aside>

        <section className="setup-stage">
          <header className="setup-stage-head">
            <div className="setup-kicker">
              {step + 1} / {STEPS.length}
            </div>
            <h2>{current.label}</h2>
          </header>

          {error && <p className="setup-error">{error}</p>}

          {step === 0 && (
            <div className="setup-body">
              <p className="setup-lead">
                Connect where this workspace lives, create the administrator, then confirm who can do
                what. The dashboard starts empty.
              </p>
              <div className="setup-preview-grid">
                <article>
                  <strong>Server</strong>
                  <span>Local, Supabase, or another API.</span>
                </article>
                <article>
                  <strong>Admin</strong>
                  <span>Your login, stored on the server when you sync.</span>
                </article>
                <article>
                  <strong>Roles</strong>
                  <span>Keep the defaults, or walk through each one.</span>
                </article>
              </div>
              {clearedDemo && (
                <p className="setup-note">Sample data from an earlier build was removed.</p>
              )}
            </div>
          )}

          {step === 1 && (
            <form
              id="setup-form"
              className="setup-body form-grid"
              onSubmit={(e) => {
                e.preventDefault();
                void saveWorkspace();
              }}
            >
              <p className="setup-lead">This is the company name people see across the app.</p>
              <div className="form-row">
                <label>Organization</label>
                <input
                  value={orgName}
                  onChange={(e) => setOrgName(e.target.value)}
                  autoFocus
                  {...exampleField("e.g. Apex Industrial")}
                />
              </div>
              <div className="form-row">
                <label>Workspace</label>
                <input
                  value={workspaceName}
                  onChange={(e) => setWorkspaceName(e.target.value)}
                  {...exampleField("e.g. Operations")}
                />
              </div>
            </form>
          )}

          {step === 2 && (
            <form
              id="setup-form"
              className="setup-body"
              onSubmit={(e) => {
                e.preventDefault();
                void saveServer();
              }}
            >
              <p className="setup-lead">
                Choose where Confluity keeps accounts, cases, and files for every user.
              </p>
              <div className="setup-choice-grid">
                {(
                  [
                    ["local", "This computer", "Stays on this computer. No shared server."],
                    ["supabase", "Supabase", "Accounts, cases, and files for every user go to this project."],
                    ["custom_api", "Other server", "Your own HTTP API as the source."],
                  ] as const
                ).map(([id, title, copy]) => (
                  <button
                    key={id}
                    type="button"
                    className={`setup-choice${provider === id ? " selected" : ""}`}
                    onClick={() => setProvider(id)}
                  >
                    <strong>{title}</strong>
                    <span>{copy}</span>
                  </button>
                ))}
              </div>
              {provider === "supabase" && (
                <div className="form-grid" style={{ marginTop: 18 }}>
                  <div className="form-row">
                    <label>Project URL</label>
                    <input
                      value={supabaseUrl}
                      onChange={(e) => setSupabaseUrl(e.target.value)}
                      {...exampleField("e.g. https://your-project.supabase.co")}
                    />
                  </div>
                  <div className="form-row">
                    <label>Anon key</label>
                    <input
                      type="password"
                      value={anonKey}
                      onChange={(e) => setAnonKey(e.target.value)}
                      {...exampleField("e.g. eyJhbGciOi...")}
                      autoComplete="off"
                    />
                  </div>
                  <div className="form-row">
                    <label>Database password</label>
                    <input
                      type="password"
                      value={dbPassword}
                      onChange={(e) => setDbPassword(e.target.value)}
                      {...exampleField("From Project Settings → Database")}
                      autoComplete="off"
                    />
                  </div>
                </div>
              )}
              {provider === "custom_api" && (
                <div className="form-row" style={{ marginTop: 18 }}>
                  <label>Server base URL</label>
                  <input
                    value={customUrl}
                    onChange={(e) => setCustomUrl(e.target.value)}
                    {...exampleField("e.g. https://api.your-company.com")}
                  />
                </div>
              )}
            </form>
          )}

          {step === 3 && (
            <form
              id="setup-form"
              className="setup-body form-grid"
              onSubmit={(e) => {
                e.preventDefault();
                if (adminSent) void confirmSavedPassword();
                else void saveAdmin();
              }}
            >
              <p className="setup-lead">
                {provider === "supabase"
                  ? "Saves this Confluity login in the database. No confirmation email."
                  : "This person is the workspace administrator until you add more members."}
              </p>
              <div className="form-row">
                <label>Name</label>
                <input
                  value={adminName}
                  onChange={(e) => {
                    setAdminName(e.target.value);
                    setAdminSent(false);
                  }}
                  autoFocus
                  {...exampleField("e.g. Jordan Hale")}
                />
              </div>
              <div className="form-row">
                <label>Email</label>
                <input
                  type="email"
                  value={adminEmail}
                  onChange={(e) => {
                    setAdminEmail(e.target.value);
                    setAdminSent(false);
                  }}
                  {...exampleField("e.g. you@company.com")}
                />
              </div>
              <div className="form-row">
                <label>Password</label>
                <input
                  type="password"
                  value={adminPassword}
                  onChange={(e) => {
                    setAdminPassword(e.target.value);
                    setAdminSent(false);
                  }}
                  autoComplete="new-password"
                  {...exampleField("Choose a password")}
                />
              </div>
              {adminSent && (
                <div className="setup-verify">
                  <p className="setup-lead">Saved to the database. Enter that password again to continue.</p>
                  <div className="form-row">
                    <label>Password</label>
                    <input
                      type="password"
                      value={verifyPassword}
                      onChange={(e) => setVerifyPassword(e.target.value)}
                      autoComplete="new-password"
                    />
                  </div>
                  <div className="form-row">
                    <label>Re-enter password</label>
                    <input
                      type="password"
                      value={verifyPasswordAgain}
                      onChange={(e) => setVerifyPasswordAgain(e.target.value)}
                      autoComplete="new-password"
                    />
                  </div>
                </div>
              )}
            </form>
          )}

          {step === 4 && (
            <div className="setup-body">
              <p className="setup-lead">
                Roles stay as the standard pack. Add the positions your company uses — job titles, not new roles.
              </p>
              <div className="form-grid">
                <div className="form-row">
                  <label>Category</label>
                  <select value={categoryId} onChange={(e) => setCategoryId(e.target.value)}>
                    <option value="">Choose a category</option>
                    {categories.map((c) => (
                      <option key={c.id} value={c.id}>
                        {c.name}
                      </option>
                    ))}
                  </select>
                </div>
                <div className="form-row">
                  <label>Or a new category</label>
                  <input
                    value={newCategory}
                    onChange={(e) => setNewCategory(e.target.value)}
                    {...exampleField("e.g. Operations")}
                  />
                </div>
                <div className="form-row">
                  <label>Position</label>
                  <input
                    value={positionName}
                    onChange={(e) => setPositionName(e.target.value)}
                    {...exampleField("e.g. Dispatcher")}
                  />
                </div>
                <div className="form-row">
                  <label>Default role</label>
                  <select value={positionRoleId} onChange={(e) => setPositionRoleId(e.target.value)}>
                    <option value="">No role attached yet</option>
                    {defaultRoles.map((role) => (
                      <option key={role.id} value={role.id}>
                        {role.name}
                      </option>
                    ))}
                  </select>
                </div>
              </div>
              <div className="form-actions" style={{ marginTop: 12 }}>
                <button className="btn" type="button" disabled={busy} onClick={() => void addPosition()}>
                  Add position
                </button>
              </div>
              {positions.length > 0 && (
                <ul className="setup-position-list">
                  {positions.map((p) => {
                    const category = categories.find((c) => c.id === p.category_id)?.name;
                    const roleNames = p.role_ids
                      .map((id) => defaultRoles.find((r) => r.id === id)?.name)
                      .filter(Boolean)
                      .join(", ");
                    return (
                      <li key={p.id}>
                        <strong>{p.name}</strong>
                        <span>
                          {[category, roleNames].filter(Boolean).join(" · ") || "Position"}
                        </span>
                      </li>
                    );
                  })}
                </ul>
              )}
            </div>
          )}

          {step === 5 && (
            <div className="setup-body">
              <p className="setup-lead">The workspace is ready. Records start empty.</p>
              <dl className="setup-summary">
                <div>
                  <dt>Organization</dt>
                  <dd>{orgName || status?.organization_name}</dd>
                </div>
                <div>
                  <dt>Workspace</dt>
                  <dd>{workspaceName || status?.workspace_name}</dd>
                </div>
                <div>
                  <dt>Server</dt>
                  <dd>{provider === "local" ? "This computer" : provider === "supabase" ? "Supabase" : "Custom API"}</dd>
                </div>
                <div>
                  <dt>Admin</dt>
                  <dd>{adminName || adminEmail || "Configured"}</dd>
                </div>
              </dl>
            </div>
          )}

          <footer className="setup-footer">
            {step > 0 ? (
              <button className="btn" type="button" disabled={busy} onClick={() => setStep((s) => s - 1)}>
                Back
              </button>
            ) : (
              <span />
            )}
            {step === 0 && (
              <div style={{ display: "flex", gap: 8 }}>
              <button className="btn" type="button" onClick={() => navigate("/login")}>
                Back to sign in
              </button>
              <button className="btn btn-primary" type="button" onClick={() => setStep(1)}>
                Begin setup
              </button>
            </div>
            )}
            {step === 1 && (
              <button className="btn btn-primary" type="submit" form="setup-form" disabled={busy}>
                Continue
              </button>
            )}
            {step === 2 && (
              <button className="btn btn-primary" type="submit" form="setup-form" disabled={busy}>
                Continue
              </button>
            )}
            {step === 3 && !adminSent && (
              <button className="btn btn-primary" type="submit" form="setup-form" disabled={busy}>
                Create admin
              </button>
            )}
            {step === 3 && adminSent && (
              <button className="btn btn-primary" type="button" disabled={busy} onClick={() => void confirmSavedPassword()}>
                Continue
              </button>
            )}
            {step === 4 && (
              <button className="btn btn-primary" type="button" disabled={busy} onClick={() => void finishRoles()}>
                Continue
              </button>
            )}
            {step === 5 && (
              <button className="btn btn-primary" type="button" disabled={busy} onClick={() => void finishSetup()}>
                Open dashboard
              </button>
            )}
          </footer>
        </section>
      </div>
    </div>
  );
}
