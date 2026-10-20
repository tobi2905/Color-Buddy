import React from "react";
import ReactDOM from "react-dom/client";
import { loadSettings } from "../../store/settings";
import MainWindow from "./MainWindow";
import "../../index.css";

// Settings come from the app database; render once they're in, so onboarding doesn't flash.
void loadSettings().then(() =>
  ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
    <React.StrictMode>
      <MainWindow />
    </React.StrictMode>,
  ),
);
