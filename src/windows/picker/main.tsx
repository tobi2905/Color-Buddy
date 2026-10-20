import React from "react";
import ReactDOM from "react-dom/client";
import PickerOverlay from "./PickerOverlay";
import "./picker.css";

const display = Number(new URLSearchParams(window.location.search).get("display"));

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <PickerOverlay display={display} />
  </React.StrictMode>,
);
