import { useNavigate } from "react-router-dom";
import { homePath } from "../lib/preferences";
import { useApp } from "../context/AppContext";
import { TitleBar } from "../components/TitleBar";
import mark from "../assets/confluity-mark.png";
import wordmark from "../assets/confluity-wordmark.png";
import wordmarkLight from "../assets/confluity-wordmark-light.png";

export function LoginPage() {
  const navigate = useNavigate();
  const { signIn } = useApp();

  return (
    <div className="app-frame">
      <TitleBar />
      <div className="login-shell">
        <div className="login-card">
          <div className="login-brand">
            <img className="login-mark" src={mark} alt="" draggable={false} />
            <img className="login-wordmark login-wordmark-dark" src={wordmark} alt="Confluity" draggable={false} />
            <img className="login-wordmark login-wordmark-light" src={wordmarkLight} alt="" draggable={false} />
          </div>
          <h1>Sample workspace</h1>
          <p className="setup-lead">
            Apex Industrial is filled with sample people, tickets, and records so you can look through the pages. Nothing here is a live company.
          </p>
          <button
            className="btn btn-primary"
            type="button"
            onClick={() => {
              signIn({ mustSetPassword: false });
              navigate(homePath(), { replace: true });
            }}
          >
            Open the sample workspace
          </button>
        </div>
      </div>
    </div>
  );
}
