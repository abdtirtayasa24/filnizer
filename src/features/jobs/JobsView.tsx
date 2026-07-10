export function JobsView() {
  return (
    <section className="content-panel" aria-labelledby="jobs-heading">
      <p className="eyebrow">Jobs / History</p>
      <h2 id="jobs-heading">Track what happened.</h2>
      <p>
        Scans, organization plans, undo actions, duplicate checks, and
        conversions will be listed here with per-file details.
      </p>
      <div className="empty-card">No jobs have run yet.</div>
    </section>
  );
}
