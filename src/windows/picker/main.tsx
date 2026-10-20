import React from "react";
import ReactDOM from "react-dom/client";
import { useSettings } from "../../store/settings";
import PickerOverlay from "./PickerOverlay";
import "./picker.css";

// Settings are saved by the main window; pick up changes made there.
window.addEventListener("storage", (e) => {
  if (e.key === "color-buddy-settings") void useSettings.persist.rehydrate();
});

const display = Number(new URLSearchParams(window.location.search).get("display"));

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <PickerOverlay display={display} />
  </React.StrictMode>,
);
