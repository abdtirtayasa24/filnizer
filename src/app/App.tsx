import { useEffect, useState } from "react";

import logoUrl from "../../src-tauri/icons/192x192.png";
import { ConverterView } from "../features/converter/ConverterView";
import { JobsView } from "../features/jobs/JobsView";
import { OrganizerView } from "../features/organizer/OrganizerView";
import { SettingsView } from "../features/settings/SettingsView";
import {
  formatCommandError,
  getAppSettings,
  getConverterToolStatus,
  installLibreOffice,
  saveAppSettings,
} from "../lib/tauri-client";

type SectionId = "organizer" | "converter" | "jobs" | "settings";

type Section = {
  id: SectionId;
  label: string;
};

const sections: Section[] = [
  { id: "organizer", label: "Organizer" },
  { id: "converter", label: "Converter" },
  { id: "jobs", label: "Jobs / History" },
  { id: "settings", label: "Settings" },
];

export function App() {
  const [activeSection, setActiveSection] = useState<SectionId>("organizer");

  useEffect(() => {
    let isMounted = true;

    async function offerLibreOfficeInstall() {
      try {
        const [settings, tools] = await Promise.all([
          getAppSettings(),
          getConverterToolStatus(),
        ]);
        const libreOffice = tools.find((tool) => tool.name === "LibreOffice");
        if (
          !isMounted ||
          !settings.allowNetworkInstalls ||
          settings.libreofficeInstallPrompted ||
          libreOffice?.available
        ) {
          return;
        }

        const nextSettings = {
          ...settings,
          libreofficeInstallPrompted: true,
        };
        await saveAppSettings(nextSettings);

        const shouldInstall = window.confirm(
          "LibreOffice is required for Office-to-PDF conversion. Filnizer can download and install LibreOffice in the background using Windows winget. Do you want to install it now?",
        );
        if (!shouldInstall) {
          return;
        }

        const result = await installLibreOffice();
        window.alert(result.message);
      } catch (error) {
        window.alert(`LibreOffice installation could not start: ${formatCommandError(error)}`);
      }
    }

    void offerLibreOfficeInstall();

    return () => {
      isMounted = false;
    };
  }, []);

  return (
    <main className="app-shell">
      <aside className="sidebar" aria-label="Primary navigation">
        <div className="brand">
          <img className="brand-logo" src={logoUrl} alt="" aria-hidden="true" />
          <div>
            <h1>Filnizer</h1>
            <p>Local file helper</p>
          </div>
        </div>

        <nav>
          {sections.map((section) => (
            <button
              key={section.id}
              type="button"
              className="nav-item"
              aria-current={activeSection === section.id ? "page" : undefined}
              onClick={() => setActiveSection(section.id)}
            >
              <span>{section.label}</span>
            </button>
          ))}
        </nav>

        <div className="sidebar-footer" aria-label="Runtime status">
          <span className="status-dot" aria-hidden="true" />
          <span>Local-first workspace</span>
        </div>
      </aside>

      {renderSection(activeSection)}
    </main>
  );
}

function renderSection(section: SectionId) {
  switch (section) {
    case "organizer":
      return <OrganizerView />;
    case "converter":
      return <ConverterView />;
    case "jobs":
      return <JobsView />;
    case "settings":
      return <SettingsView />;
  }
}
