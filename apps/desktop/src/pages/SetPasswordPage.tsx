import { useState, type FormEvent } from "react";
import { Navigate, useNavigate } from "react-router-dom";
import { api } from "../lib/api";
import { homePath } from "../lib/preferences";
import { useApp } from "../context/AppContext";
import { TitleBar } from "../components/TitleBar";
import mark from "../assets/confluity-mark.png";
import wordmark from "../assets/confluity-wordmark.png";
import wordmarkLight from "../assets/confluity-wordmark-light.png";

export function SetPasswordPage() {
  const navigate = useNavigate();
  const { signedIn, mustSetPassword, signIn, signOut, refreshPermissions } = useApp();
  const [password, setPassword] = useState("");
  const [confirm, setConfirm] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  if (!signedIn) return <Navigate to="/login" replace />;
  if (!mustSetPassword) return <Navigate to={homePath()} replace />;

  async function submit(e: FormEvent) {
    e.preventDefault();
    setError(null);
    if (password !== confirm) {
      setError("Re-enter the same password in both fields.");
      return;
    }
    setBusy(true);
    try {
      const session = await api.completeMemberPassword(password, confirm);
      signIn({ mustSetPassword: session.must_set_password });
      await refreshPermissions();
      navigate(homePath(), { replace: true });
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setBusy(false);
    }
  }

  return (
    <div className="app-frame">
      <TitleBar />
      <div className="login-shell">
        <form className="login-card" onSubmit={submit}>
          <div className="login-brand">
            <img className="login-mark" src={mark} alt="" draggable={false} />
            <img className="login-wordmark login-wordmark-dark" src={wordmark} alt="Confluity" draggable={false} />
            <img className="login-wordmark login-wordmark-light" src={wordmarkLight} alt="" draggable={false} />
          </div>
          <h1>Choose a password</h1>
          <p className="setup-lead">
            Your sign-in code worked. Set a password for this account. It is saved to Supabase and
            replaces the code.
          </p>
          {error && <p className="setup-error">{error}</p>}
          <div className="form-row">
            <label>New password</label>
            <input
              type="password"
              value={password}
              onChange={(e) => setPassword(e.target.value)}
              autoFocus
              autoComplete="new-password"
              minLength={6}
            />
          </div>
          <div className="form-row">
            <label>Re-enter password</label>
            <input
              type="password"
              value={confirm}
              onChange={(e) => setConfirm(e.target.value)}
              autoComplete="new-password"
              minLength={6}
            />
          </div>
          <button className="btn btn-primary" type="submit" disabled={busy || password.length < 6}>
            Save password
          </button>
          <p className="login-alt">
            <button
              type="button"
              className="btn btn-ghost"
              onClick={() => {
                signOut();
                navigate("/login", { replace: true });
              }}
            >
              Use a different account
            </button>
          </p>
        </form>
      </div>
    </div>
  );
}
