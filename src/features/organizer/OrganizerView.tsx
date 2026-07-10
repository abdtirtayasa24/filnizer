import { useEffect, useMemo, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { open } from "@tauri-apps/plugin-dialog";

import {
  FileCategory,
  FileEntry,
  OrganizerRule,
  ScanProgressEvent,
  formatCommandError,
  listOrganizerRules,
  saveOrganizerRules,
  startOrganizerScan,
} from "../../lib/tauri-client";

const categoryOptions: { value: FileCategory; label: string }[] = [
  { value: "images", label: "Images" },
  { value: "documents", label: "Documents" },
  { value: "pdfs", label: "PDFs" },
  { value: "spreadsheets", label: "Spreadsheets" },
  { value: "presentations", label: "Presentations" },
  { value: "videos", label: "Videos" },
  { value: "audio", label: "Audio" },
  { value: "archives", label: "Archives" },
  { value: "code", label: "Code" },
  { value: "executables", label: "Executables" },
  { value: "other", label: "Other" },
];

export function OrganizerView() {
  const [selectedFolder, setSelectedFolder] = useState<string | null>(null);
  const [recursive, setRecursive] = useState(true);
  const [isScanning, setIsScanning] = useState(false);
  const [progress, setProgress] = useState<ScanProgressEvent | null>(null);
  const [files, setFiles] = useState<FileEntry[]>([]);
  const [rules, setRules] = useState<OrganizerRule[]>([]);
  const [ruleExtension, setRuleExtension] = useState("");
  const [ruleCategory, setRuleCategory] = useState<FileCategory>("documents");
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

  useEffect(() => {
    listOrganizerRules()
      .then(setRules)
      .catch((rulesError: unknown) => setError(formatCommandError(rulesError)));
  }, []);

  const categoryCounts = useMemo(() => {
    return files.reduce<Record<string, number>>((counts, file) => {
      counts[file.category] = (counts[file.category] ?? 0) + 1;
      return counts;
    }, {});
  }, [files]);

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

  async function addExtensionRule() {
    const value = ruleExtension.trim().replace(/^\./, "");
    if (!value) {
      setError("Enter a file extension for the rule.");
      return;
    }

    try {
      const updatedRules = await saveOrganizerRules([
        ...rules,
        { kind: "extension", value, category: ruleCategory },
      ]);
      setRules(updatedRules);
      setRuleExtension("");
      setError(null);
    } catch (ruleError) {
      setError(formatCommandError(ruleError));
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

      <div className="rule-card" aria-labelledby="rules-heading">
        <h3 id="rules-heading">Custom extension rules</h3>
        <div className="rule-form">
          <input
            type="text"
            value={ruleExtension}
            placeholder="jpg"
            aria-label="File extension"
            onChange={(event) => setRuleExtension(event.currentTarget.value)}
          />
          <select
            value={ruleCategory}
            aria-label="Rule category"
            onChange={(event) => setRuleCategory(event.currentTarget.value as FileCategory)}
          >
            {categoryOptions.map((category) => (
              <option key={category.value} value={category.value}>
                {category.label}
              </option>
            ))}
          </select>
          <button type="button" className="secondary-button" onClick={addExtensionRule}>
            Add rule
          </button>
        </div>
        <p>{rules.length} custom rule(s) saved locally.</p>
      </div>

      {selectedFolder ? <p className="selected-path">{selectedFolder}</p> : null}
      {progress ? (
        <p className="progress-text" role="status">
          Scanned {progress.scannedFiles} file(s)
        </p>
      ) : null}
      {error ? <p className="inline-error">{error}</p> : null}

      <div className="empty-card">
        {files.length > 0
          ? `${files.length} file(s) found. ${Object.keys(categoryCounts).length} category group(s) detected.`
          : "No scan results yet."}
      </div>
    </section>
  );
}
