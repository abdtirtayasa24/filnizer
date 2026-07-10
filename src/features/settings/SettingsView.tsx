import { useEffect, useState } from "react";

import {
  AppStatus,
  ToolStatus,
  formatCommandError,
  getAppStatus,
  getConverterToolStatus,
} from "../../lib/tauri-client";

export function SettingsView() {
  const [status, setStatus] = useState<AppStatus | null>(null);
  const [tools, setTools] = useState<ToolStatus[]>([]);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let isMounted = true;

    Promise.all([getAppStatus(), getConverterToolStatus()])
      .then(([appStatus, toolStatus]) => {
        if (isMounted) {
          setStatus(appStatus);
          setTools(toolStatus);
          setError(null);
        }
      })
      .catch((commandError: unknown) => {
        if (isMounted) {
          setError(formatCommandError(commandError));
        }
      });

    return () => {
      isMounted = false;
    };
  }, []);

  return (
    <section className="content-panel" aria-labelledby="settings-heading">
      <p className="eyebrow">Settings</p>
      <h2 id="settings-heading">Keep everything local.</h2>
      <p>
        Filnizer is designed for offline workflows. Helper status is checked
        locally from the app folder.
      </p>

      <div className="status-grid">
        <div className="status-card">
          <span>Runtime network</span>
          <strong>
            {status
              ? status.runtimeNetworkEnabled
                ? "Enabled"
                : "Disabled"
              : "Checking..."}
          </strong>
        </div>
        <div className="status-card">
          <span>Version</span>
          <strong>{status?.version ?? "Checking..."}</strong>
        </div>
        {tools.map((tool) => (
          <div className="status-card" key={tool.name}>
            <span>{tool.name}</span>
            <strong>{tool.available ? "Available" : "Missing"}</strong>
            <small>{tool.path ?? tool.guidance}</small>
          </div>
        ))}
      </div>

      {error ? <p className="inline-error">Status unavailable: {error}</p> : null}
    </section>
  );
}
