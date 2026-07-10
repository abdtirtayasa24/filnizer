import { useState } from "react";

import logoUrl from "../../src-tauri/icons/192x192.png";
import { ConverterView } from "../features/converter/ConverterView";
import { JobsView } from "../features/jobs/JobsView";
import { OrganizerView } from "../features/organizer/OrganizerView";
import { SettingsView } from "../features/settings/SettingsView";

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

  return (
    <main className="app-shell">
      <aside className="sidebar" aria-label="Primary navigation">
        <div className="brand">
          <img className="brand-logo" src={logoUrl} alt="" aria-hidden="true" />
          <div>
            <h1>Filnizer</h1>
            <p>Offline file helper</p>
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
              {section.label}
            </button>
          ))}
        </nav>
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
