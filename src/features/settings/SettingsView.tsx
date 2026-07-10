import { useEffect, useState } from "react";

import { AppStatus, formatCommandError, getAppStatus } from "../../lib/tauri-client";

export function SettingsView() {
  const [status, setStatus] = useState<AppStatus | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let isMounted = true;

    getAppStatus()
      .then((appStatus) => {
        if (isMounted) {
          setStatus(appStatus);
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
        Filnizer is designed for offline workflows. Helper tool status and
        default behavior controls will be added here.
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
      </div>

      {error ? <p className="inline-error">Status unavailable: {error}</p> : null}
    </section>
  );
}
