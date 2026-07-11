import { useEffect, useState } from "react";
import { open } from "@tauri-apps/plugin-dialog";

import {
  AppSettings,
  AppStatus,
  ConflictPolicy,
  ToolStatus,
  formatCommandError,
  getAppSettings,
  getAppStatus,
  getConverterToolStatus,
  installLibreOffice,
  saveAppSettings,
} from "../../lib/tauri-client";

const retentionOptions: Array<{ label: string; value: number | null }> = [
  { label: "30 days", value: 30 },
  { label: "90 days", value: 90 },
  { label: "180 days", value: 180 },
  { label: "Keep until I clear it", value: null },
];

export function SettingsView() {
  const [status, setStatus] = useState<AppStatus | null>(null);
  const [settings, setSettings] = useState<AppSettings | null>(null);
  const [tools, setTools] = useState<ToolStatus[]>([]);
  const [isSaving, setIsSaving] = useState(false);
  const [isInstallingLibreOffice, setIsInstallingLibreOffice] = useState(false);
  const [message, setMessage] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  const libreOffice = tools.find((tool) => tool.name === "LibreOffice");

  useEffect(() => {
    let isMounted = true;

    loadSettings()
      .catch((commandError: unknown) => {
        if (isMounted) {
          setError(formatCommandError(commandError));
        }
      });

    async function loadSettings() {
      const [appStatus, toolStatus, appSettings] = await Promise.all([
        getAppStatus(),
        getConverterToolStatus(),
        getAppSettings(),
      ]);
      if (isMounted) {
        setStatus(appStatus);
        setTools(toolStatus);
        setSettings(appSettings);
        setError(null);
      }
    }

    return () => {
      isMounted = false;
    };
  }, []);

  async function chooseDefaultOutputDirectory() {
    const selected = await open({ directory: true, multiple: false });
    if (typeof selected === "string") {
      updateSettings({ defaultOutputDirectory: selected });
    }
  }

  async function refreshToolStatus() {
    setTools(await getConverterToolStatus());
  }

  async function runLibreOfficeInstall() {
    if (!window.confirm("Download and install LibreOffice using Windows winget now?")) {
      return;
    }

    setIsInstallingLibreOffice(true);
    setMessage(null);
    setError(null);
    try {
      const result = await installLibreOffice();
      setMessage(result.message);
      await refreshToolStatus();
    } catch (installError) {
      setError(formatCommandError(installError));
    } finally {
      setIsInstallingLibreOffice(false);
    }
  }

  async function persistSettings() {
    if (!settings) {
      return;
    }

    setIsSaving(true);
    setMessage(null);
    setError(null);
    try {
      setSettings(await saveAppSettings(settings));
      setMessage("Settings saved locally.");
    } catch (saveError) {
      setError(formatCommandError(saveError));
    } finally {
      setIsSaving(false);
    }
  }

  function updateSettings(patch: Partial<AppSettings>) {
    setSettings((current) => (current ? { ...current, ...patch } : current));
    setMessage(null);
  }

  return (
    <section className="content-panel" aria-labelledby="settings-heading">
      <p className="eyebrow">Settings</p>
      <h2 id="settings-heading">Local-first, with confirmed installs.</h2>
      <p>Set safe defaults, check helper tools, and control whether Filnizer may install LibreOffice after asking you first.</p>

      <div className="settings-layout">
        <div className="workflow-card settings-card settings-card-wide">
          <h3>Defaults</h3>
          <p>These preferences are stored in the local SQLite database on this computer.</p>

          <div className="settings-form-grid">
            <label>
              Default output folder
              <div className="setting-inline-action">
                <button type="button" className="secondary-button" onClick={chooseDefaultOutputDirectory}>
                  Choose folder
                </button>
                <button
                  type="button"
                  className="secondary-button"
                  onClick={() => updateSettings({ defaultOutputDirectory: null })}
                  disabled={!settings?.defaultOutputDirectory}
                >
                  Clear
                </button>
              </div>
            </label>
            <p className="selected-path">{settings?.defaultOutputDirectory ?? "No default folder selected."}</p>

            <label>
              If output exists
              <select
                value={settings?.defaultConflictPolicy ?? "rename"}
                onChange={(event) =>
                  updateSettings({ defaultConflictPolicy: event.currentTarget.value as ConflictPolicy })
                }
              >
                <option value="rename">Rename new file</option>
                <option value="skip">Skip file</option>
                <option value="overwrite">Overwrite</option>
              </select>
            </label>

            <label>
              History retention
              <select
                value={settings?.historyRetentionDays ?? "forever"}
                onChange={(event) => {
                  const value = event.currentTarget.value;
                  updateSettings({ historyRetentionDays: value === "forever" ? null : Number(value) });
                }}
              >
                {retentionOptions.map((option) => (
                  <option key={option.value ?? "forever"} value={option.value ?? "forever"}>
                    {option.label}
                  </option>
                ))}
              </select>
            </label>

            <label className="checkbox-row settings-checkbox-row">
              <input
                type="checkbox"
                checked={settings?.showPrivacyNote ?? true}
                onChange={(event) => updateSettings({ showPrivacyNote: event.currentTarget.checked })}
              />
              Show privacy note in the app
            </label>

            <label className="checkbox-row settings-checkbox-row">
              <input
                type="checkbox"
                checked={settings?.allowNetworkInstalls ?? true}
                onChange={(event) => updateSettings({ allowNetworkInstalls: event.currentTarget.checked })}
              />
              Allow confirmed LibreOffice network install prompts
            </label>
          </div>

          <div className="action-row">
            <button type="button" className="primary-button" onClick={persistSettings} disabled={!settings || isSaving}>
              {isSaving ? "Saving..." : "Save settings"}
            </button>
          </div>
          {message ? <p className="settings-success">{message}</p> : null}
        </div>

        <div className="workflow-card settings-card privacy-card">
          <h3>Network install policy</h3>
          <p>{status?.networkPolicy ?? "Checking network policy..."}</p>
          <strong>{status ? (status.runtimeNetworkEnabled ? "Confirmed install only" : "Network silent") : "Checking..."}</strong>
          <button
            type="button"
            className="secondary-button"
            onClick={runLibreOfficeInstall}
            disabled={Boolean(libreOffice?.available) || isInstallingLibreOffice}
          >
            {isInstallingLibreOffice ? "Installing LibreOffice..." : "Install LibreOffice"}
          </button>
        </div>
      </div>

      <div className="status-grid settings-status-grid">
        <div className="status-card">
          <span>Runtime network</span>
          <strong>{status ? (status.runtimeNetworkEnabled ? "Confirmed install only" : "Disabled") : "Checking..."}</strong>
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

      {error ? <p className="inline-error">Settings unavailable: {error}</p> : null}
    </section>
  );
}
