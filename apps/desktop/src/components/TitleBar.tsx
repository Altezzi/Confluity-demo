import { getCurrentWindow } from "@tauri-apps/api/window";
import type { PointerEvent } from "react";
import logo from "../assets/confluity-logo-trim.png";
import logoLight from "../assets/confluity-logo-light.png";

function trackPointer(e: PointerEvent<HTMLButtonElement>) {
  const r = e.currentTarget.getBoundingClientRect();
  e.currentTarget.style.setProperty("--mx", `${e.clientX - r.left}px`);
  e.currentTarget.style.setProperty("--my", `${e.clientY - r.top}px`);
}

export function TitleBar() {
  const appWindow = getCurrentWindow();

  return (
    <div className="titlebar" data-tauri-drag-region>
      <div className="titlebar-brand" data-tauri-drag-region>
        <img className="titlebar-logo titlebar-logo-dark" src={logo} alt="Confluity" draggable={false} />
        <img className="titlebar-logo titlebar-logo-light" src={logoLight} alt="" draggable={false} />
      </div>
      <div className="titlebar-controls">
        <button
          type="button"
          className="titlebar-btn"
          aria-label="Minimize"
          onPointerEnter={trackPointer}
          onPointerMove={trackPointer}
          onClick={() => void appWindow.minimize()}
        >
          <svg width="10" height="10" viewBox="0 0 10 10" fill="none">
            <path d="M1 5h8" stroke="currentColor" strokeWidth="1.2" />
          </svg>
        </button>
        <button
          type="button"
          className="titlebar-btn"
          aria-label="Maximize"
          onPointerEnter={trackPointer}
          onPointerMove={trackPointer}
          onClick={() => void appWindow.toggleMaximize()}
        >
          <svg width="10" height="10" viewBox="0 0 10 10" fill="none">
            <rect x="1.25" y="1.25" width="7.5" height="7.5" stroke="currentColor" strokeWidth="1.2" />
          </svg>
        </button>
        <button
          type="button"
          className="titlebar-btn titlebar-btn-close"
          aria-label="Close"
          onPointerEnter={trackPointer}
          onPointerMove={trackPointer}
          onClick={() => void appWindow.close()}
        >
          <svg width="10" height="10" viewBox="0 0 10 10" fill="none">
            <path d="M2 2l6 6M8 2L2 8" stroke="currentColor" strokeWidth="1.2" />
          </svg>
        </button>
      </div>
    </div>
  );
}
