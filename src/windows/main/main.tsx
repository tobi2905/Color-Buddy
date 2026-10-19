import React from "react";
import ReactDOM from "react-dom/client";
import MainWindow from "./MainWindow";
import "../../index.css";

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <MainWindow />
  </React.StrictMode>,
);
