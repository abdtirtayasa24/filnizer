import { useEffect, useState } from "react";

import {
  JobDetailsResponse,
  JobKind,
  JobStatus,
  JobSummary,
  formatCommandError,
  getJobDetails,
  listJobs,
} from "../../lib/tauri-client";

export function JobsView() {
  const [jobs, setJobs] = useState<JobSummary[]>([]);
  const [selectedJobId, setSelectedJobId] = useState<string | null>(null);
  const [details, setDetails] = useState<JobDetailsResponse | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    void refreshJobs();
  }, []);

  useEffect(() => {
    let isMounted = true;

    if (!selectedJobId) {
      setDetails(null);
      return;
    }

    getJobDetails(selectedJobId)
      .then((response) => {
        if (isMounted) {
          setDetails(response);
          setError(null);
        }
      })
      .catch((detailsError: unknown) => {
        if (isMounted) {
          setError(formatCommandError(detailsError));
        }
      });

    return () => {
      isMounted = false;
    };
  }, [selectedJobId]);

  async function refreshJobs() {
    setIsLoading(true);
    setError(null);
    try {
      const history = await listJobs();
      setJobs(history);
      setSelectedJobId((current) => current ?? history[0]?.id ?? null);
    } catch (historyError) {
      setError(formatCommandError(historyError));
    } finally {
      setIsLoading(false);
    }
  }

  return (
    <section className="content-panel" aria-labelledby="jobs-heading">
      <p className="eyebrow">Jobs / History</p>
      <h2 id="jobs-heading">Track what happened.</h2>
      <p>Review local scans, organizer actions, duplicate checks, and conversions with saved per-file results.</p>

      <div className="history-toolbar">
        <strong>{jobs.length} history item(s)</strong>
        <button type="button" className="secondary-button" onClick={refreshJobs} disabled={isLoading}>
          {isLoading ? "Refreshing..." : "Refresh"}
        </button>
      </div>

      {error ? <p className="inline-error">History unavailable: {error}</p> : null}

      {jobs.length === 0 && !isLoading ? (
        <div className="empty-card">No jobs have run yet. Scan, organize, or convert files to create history.</div>
      ) : null}

      {jobs.length > 0 ? (
        <div className="job-history-grid">
          <div className="job-list" aria-label="Job history">
            {jobs.map((job) => (
              <button
                key={job.id}
                type="button"
                className="job-row"
                aria-current={selectedJobId === job.id ? "true" : undefined}
                onClick={() => setSelectedJobId(job.id)}
              >
                <span className="job-row-title">{job.name}</span>
                <span className="job-row-meta">
                  {formatJobKind(job.kind)} · {formatDate(job.updatedAtUnixMs)}
                </span>
                <span className={`job-status job-status-${job.status}`}>{formatJobStatus(job.status)}</span>
              </button>
            ))}
          </div>

          <div className="results-panel job-details" aria-live="polite">
            {details ? (
              <>
                <div className="job-details-header">
                  <div>
                    <h3>{details.job.name}</h3>
                    <p>{formatJobKind(details.job.kind)} · {formatDate(details.job.updatedAtUnixMs)}</p>
                  </div>
                  <span className={`job-status job-status-${details.job.status}`}>
                    {formatJobStatus(details.job.status)}
                  </span>
                </div>

                <div className="job-metrics">
                  <span>
                    <strong>{details.job.completedFiles}</strong>
                    Completed
                  </span>
                  <span>
                    <strong>{details.job.totalFiles}</strong>
                    Total
                  </span>
                  <span>
                    <strong>{details.fileResults.length}</strong>
                    File details
                  </span>
                </div>

                {details.job.errorMessage ? <p className="inline-error">{details.job.errorMessage}</p> : null}

                {details.fileResults.length > 0 ? (
                  <ul className="job-file-results">
                    {details.fileResults.slice(0, 80).map((result) => (
                      <li key={result.id}>
                        <span className="file-result-status">{result.status}</span>
                        <span className="file-result-paths">
                          {result.sourcePath}
                          {result.targetPath ? ` → ${result.targetPath}` : ""}
                        </span>
                        {result.message ? <span className="file-result-message">{result.message}</span> : null}
                      </li>
                    ))}
                  </ul>
                ) : (
                  <p>No per-file detail rows were saved for this job type.</p>
                )}
              </>
            ) : (
              <p>Select a job to inspect its details.</p>
            )}
          </div>
        </div>
      ) : null}
    </section>
  );
}

function formatDate(unixMs: number) {
  return new Date(unixMs).toLocaleString(undefined, {
    dateStyle: "medium",
    timeStyle: "short",
  });
}

function formatJobKind(kind: JobKind) {
  const labels: Record<JobKind, string> = {
    organizerScan: "Organizer scan",
    organizerApply: "Organizer apply",
    organizerUndo: "Organizer undo",
    duplicateAnalysis: "Duplicate analysis",
    conversion: "Conversion",
  };
  return labels[kind];
}

function formatJobStatus(status: JobStatus) {
  const labels: Record<JobStatus, string> = {
    queued: "Queued",
    running: "Running",
    completed: "Completed",
    failed: "Failed",
    canceled: "Canceled",
    partiallyCompleted: "Partial",
  };
  return labels[status];
}
