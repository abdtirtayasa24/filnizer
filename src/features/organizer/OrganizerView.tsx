export function OrganizerView() {
  return (
    <section className="content-panel" aria-labelledby="organizer-heading">
      <p className="eyebrow">Organizer</p>
      <h2 id="organizer-heading">Start with a safe preview.</h2>
      <p>
        Scan a folder, review proposed categories and renames, then apply only
        the changes you approve. Folder scanning will be connected in the next
        phase.
      </p>
      <div className="empty-card">No folder has been selected yet.</div>
    </section>
  );
}
