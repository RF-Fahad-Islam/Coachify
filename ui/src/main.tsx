import React from "react";
import ReactDOM from "react-dom/client";
import { ThemeProvider } from "./theme/theme-provider";
import { App } from "./App";
import { rpc } from "./gen/rpc";
import "./index.css";

if (import.meta.env.DEV && typeof window !== "undefined") {
  // Expose convenient typed rpc helper in browser console with auto-session resolution
  (window as unknown as Record<string, unknown>).rpc = (
    cmd: Parameters<typeof rpc>[0],
    payload: Parameters<typeof rpc>[1],
    options?: Parameters<typeof rpc>[2]
  ) => {
    const sessionToken =
      typeof options === "object" && options !== null
        ? options.sessionToken
        : sessionStorage.getItem("coachify_session_token");
    return rpc(cmd, payload, {
      sessionToken,
      ...(typeof options === "object" ? options : {}),
    });
  };
}

ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <ThemeProvider>
      <App />
    </ThemeProvider>
  </React.StrictMode>
);
