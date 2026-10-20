import React from "react";
import ReactDOM from "react-dom/client";
import { onSettingsChanged } from "../../lib/storage";
import { loadSettings, useSettings } from "../../store/settings";
import PickerOverlay from "./PickerOverlay";
import "./picker.css";

// Settings are changed in the main window; follow them here.
void onSettingsChanged((s) => useSettings.setState(s));

const display = Number(new URLSearchParams(window.location.search).get("display"));

void loadSettings().then(() =>
  ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
    <React.StrictMode>
      <PickerOverlay display={display} />
    </React.StrictMode>,
  ),
);
