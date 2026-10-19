import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import { applyPreferences, loadPreferences } from "./lib/preferences";
import "./styles/global.css";

applyPreferences(loadPreferences());

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);
