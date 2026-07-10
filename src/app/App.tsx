const sections = ["Organizer", "Converter", "Jobs", "Settings"];

export function App() {
  return (
    <main className="app-shell">
      <aside className="sidebar" aria-label="Primary navigation">
        <div className="brand">
          <span className="brand-mark" aria-hidden="true">
            F
          </span>
          <div>
            <h1>Filnizer</h1>
            <p>Offline file helper</p>
          </div>
        </div>

        <nav>
          {sections.map((section) => (
            <button key={section} type="button" className="nav-item">
              {section}
            </button>
          ))}
        </nav>
      </aside>

      <section className="hero" aria-labelledby="welcome-heading">
        <p className="eyebrow">Windows portable utility</p>
        <h2 id="welcome-heading">Organize and convert files locally.</h2>
        <p>
          Filnizer will help clean messy folders, detect duplicates, and convert
          files without relying on online tools.
        </p>
        <div className="status-card" role="status">
          Foundation scaffold is ready. Core organizer and converter workflows
          will be added incrementally.
        </div>
      </section>
    </main>
  );
}
