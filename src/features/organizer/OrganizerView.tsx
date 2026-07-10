import { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { open } from "@tauri-apps/plugin-dialog";

import {
  FileEntry,
  ScanProgressEvent,
  formatCommandError,
  startOrganizerScan,
} from "../../lib/tauri-client";

export function OrganizerView() {
  const [selectedFolder, setSelectedFolder] = useState<string | null>(null);
  const [recursive, setRecursive] = useState(true);
  const [isScanning, setIsScanning] = useState(false);
  const [progress, setProgress] = useState<ScanProgressEvent | null>(null);
  const [files, setFiles] = useState<FileEntry[]>([]);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let unlisten: (() => void) | undefined;

    listen<ScanProgressEvent>("organizer://scan-progress", (event) => {
      setProgress(event.payload);
    }).then((eventUnlisten) => {
      unlisten = eventUnlisten;
    });

    return () => {
      unlisten?.();
    };
  }, []);

  async function chooseFolder() {
    const folder = await open({ directory: true, multiple: false });
    if (typeof folder === "string") {
      setSelectedFolder(folder);
      setFiles([]);
      setProgress(null);
      setError(null);
    }
  }

  async function scanFolder() {
    if (!selectedFolder) {
      setError("Select a folder before scanning.");
      return;
    }

    setIsScanning(true);
    setError(null);
    setProgress(null);

    try {
      const response = await startOrganizerScan({
        roots: [selectedFolder],
        recursive,
        includeHidden: false,
      });
      setFiles(response.files);
    } catch (scanError) {
      setError(formatCommandError(scanError));
    } finally {
      setIsScanning(false);
    }
  }

  return (
    <section className="content-panel" aria-labelledby="organizer-heading">
      <p className="eyebrow">Organizer</p>
      <h2 id="organizer-heading">Start with a safe preview.</h2>
      <p>
        Scan a folder, review proposed categories and renames, then apply only
        the changes you approve.
      </p>

      <div className="action-row">
        <button type="button" className="primary-button" onClick={chooseFolder}>
          Choose folder
        </button>
        <button
          type="button"
          className="secondary-button"
          onClick={scanFolder}
          disabled={!selectedFolder || isScanning}
        >
          {isScanning ? "Scanning..." : "Scan folder"}
        </button>
      </div>

      <label className="checkbox-row">
        <input
          type="checkbox"
          checked={recursive}
          onChange={(event) => setRecursive(event.currentTarget.checked)}
        />
        Include subfolders
      </label>

      {selectedFolder ? <p className="selected-path">{selectedFolder}</p> : null}
      {progress ? (
        <p className="progress-text" role="status">
          Scanned {progress.scannedFiles} file(s)
        </p>
      ) : null}
      {error ? <p className="inline-error">{error}</p> : null}

      <div className="empty-card">
        {files.length > 0
          ? `${files.length} file(s) found. Categorized previews are next.`
          : "No scan results yet."}
      </div>
    </section>
  );
}
