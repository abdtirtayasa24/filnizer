import { useEffect, useMemo, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { open } from "@tauri-apps/plugin-dialog";

import {
  ApplyFileResult,
  DuplicateSet,
  FileCategory,
  FileEntry,
  OperationPlan,
  OrganizerRule,
  ScanProgressEvent,
  applyOrganizerPlan,
  findDuplicateFiles,
  formatCommandError,
  listOrganizerRules,
  previewOrganizerPlan,
  saveOrganizerRules,
  startOrganizerScan,
  undoOrganizerPlan,
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
  const [destinationFolder, setDestinationFolder] = useState<string | null>(null);
  const [recursive, setRecursive] = useState(true);
  const [isBusy, setIsBusy] = useState(false);
  const [progress, setProgress] = useState<ScanProgressEvent | null>(null);
  const [files, setFiles] = useState<FileEntry[]>([]);
  const [plan, setPlan] = useState<OperationPlan | null>(null);
  const [operationResults, setOperationResults] = useState<ApplyFileResult[]>([]);
  const [duplicateSets, setDuplicateSets] = useState<DuplicateSet[]>([]);
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
      resetWorkflowAfterSelection();
    }
  }

  async function chooseDestinationFolder() {
    const folder = await open({ directory: true, multiple: false });
    if (typeof folder === "string") {
      setDestinationFolder(folder);
      setPlan(null);
      setOperationResults([]);
      setError(null);
    }
  }

  async function scanFolder() {
    if (!selectedFolder) {
      setError("Select a folder before scanning.");
      return;
    }

    await runBusyAction(async () => {
      setProgress(null);
      const response = await startOrganizerScan({
        roots: [selectedFolder],
        recursive,
        includeHidden: false,
      });
      setFiles(response.files);
      setPlan(null);
      setOperationResults([]);
      setDuplicateSets([]);
    });
  }

  async function previewPlan() {
    if (files.length === 0 || !destinationFolder) {
      setError("Scan files and choose a destination folder before previewing.");
      return;
    }

    await runBusyAction(async () => {
      setPlan(await previewOrganizerPlan(files, destinationFolder));
      setOperationResults([]);
    });
  }

  async function applyPlan() {
    if (!plan) {
      setError("Create a preview plan before applying changes.");
      return;
    }

    await runBusyAction(async () => {
      const response = await applyOrganizerPlan(plan);
      setOperationResults(response.results);
    });
  }

  async function undoPlan() {
    if (!plan) {
      setError("Apply a plan before undoing it.");
      return;
    }

    await runBusyAction(async () => {
      const response = await undoOrganizerPlan(plan);
      setOperationResults(response.results);
    });
  }

  async function detectDuplicates() {
    if (files.length === 0) {
      setError("Scan files before checking for duplicates.");
      return;
    }

    await runBusyAction(async () => {
      const response = await findDuplicateFiles(files);
      setDuplicateSets(response.sets);
    });
  }

  async function addExtensionRule() {
    const value = ruleExtension.trim().replace(/^\./, "");
    if (!value) {
      setError("Enter a file extension for the rule.");
      return;
    }

    await runBusyAction(async () => {
      const updatedRules = await saveOrganizerRules([
        ...rules,
        { kind: "extension", value, category: ruleCategory },
      ]);
      setRules(updatedRules);
      setRuleExtension("");
    });
  }

  async function runBusyAction(action: () => Promise<void>) {
    setIsBusy(true);
    setError(null);
    try {
      await action();
    } catch (actionError) {
      setError(formatCommandError(actionError));
    } finally {
      setIsBusy(false);
    }
  }

  function resetWorkflowAfterSelection() {
    setFiles([]);
    setPlan(null);
    setOperationResults([]);
    setDuplicateSets([]);
    setProgress(null);
    setError(null);
  }

  return (
    <section className="content-panel organizer-panel" aria-labelledby="organizer-heading">
      <p className="eyebrow">Organizer</p>
      <h2 id="organizer-heading">Review every change before it happens.</h2>
      <p>
        Scan a folder, tune simple rules, preview the move/rename plan, and undo
        safely if the original paths are still clear.
      </p>

      <div className="workflow-grid">
        <div className="workflow-card">
          <h3>1. Scan</h3>
          <div className="action-row">
            <button type="button" className="primary-button" onClick={chooseFolder}>
              Choose source
            </button>
            <button
              type="button"
              className="secondary-button"
              onClick={scanFolder}
              disabled={!selectedFolder || isBusy}
            >
              Scan folder
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
          {progress ? <p role="status">Scanned {progress.scannedFiles} file(s)</p> : null}
        </div>

        <div className="workflow-card">
          <h3>2. Rules</h3>
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

        <div className="workflow-card">
          <h3>3. Preview and apply</h3>
          <div className="action-row">
            <button type="button" className="secondary-button" onClick={chooseDestinationFolder}>
              Choose destination
            </button>
            <button
              type="button"
              className="secondary-button"
              onClick={previewPlan}
              disabled={files.length === 0 || !destinationFolder || isBusy}
            >
              Preview plan
            </button>
            <button
              type="button"
              className="primary-button"
              onClick={applyPlan}
              disabled={!plan || isBusy}
            >
              Apply plan
            </button>
            <button
              type="button"
              className="secondary-button"
              onClick={undoPlan}
              disabled={!plan || operationResults.length === 0 || isBusy}
            >
              Undo
            </button>
          </div>
          {destinationFolder ? <p className="selected-path">{destinationFolder}</p> : null}
        </div>

        <div className="workflow-card">
          <h3>4. Duplicates</h3>
          <p>Duplicate detection only reports matching files. It never deletes files.</p>
          <button
            type="button"
            className="secondary-button"
            onClick={detectDuplicates}
            disabled={files.length === 0 || isBusy}
          >
            Check duplicates
          </button>
        </div>
      </div>

      {error ? <p className="inline-error">{error}</p> : null}
      {isBusy ? <p role="status">Working locally...</p> : null}

      <ResultsSummary
        files={files}
        categoryCounts={categoryCounts}
        plan={plan}
        operationResults={operationResults}
        duplicateSets={duplicateSets}
      />
    </section>
  );
}

type ResultsSummaryProps = {
  files: FileEntry[];
  categoryCounts: Record<string, number>;
  plan: OperationPlan | null;
  operationResults: ApplyFileResult[];
  duplicateSets: DuplicateSet[];
};

function ResultsSummary({
  files,
  categoryCounts,
  plan,
  operationResults,
  duplicateSets,
}: ResultsSummaryProps) {
  return (
    <div className="results-panel" aria-live="polite">
      <h3>Summary</h3>
      <p>{files.length > 0 ? `${files.length} file(s) scanned.` : "No scan results yet."}</p>

      {Object.entries(categoryCounts).length > 0 ? (
        <ul className="pill-list" aria-label="Detected categories">
          {Object.entries(categoryCounts).map(([category, count]) => (
            <li key={category}>{`${category}: ${count}`}</li>
          ))}
        </ul>
      ) : null}

      {plan ? (
        <div className="result-block">
          <strong>{plan.operations.length} planned operation(s)</strong>
          <ol>
            {plan.operations.slice(0, 5).map((operation) => (
              <li key={operation.id}>{operation.targetPath}</li>
            ))}
          </ol>
        </div>
      ) : null}

      {operationResults.length > 0 ? (
        <div className="result-block">
          <strong>{operationResults.length} operation result(s)</strong>
          <ul>
            {operationResults.slice(0, 5).map((result) => (
              <li key={`${result.operationId}-${result.status}`}>
                {result.status}: {result.message ?? result.targetPath}
              </li>
            ))}
          </ul>
        </div>
      ) : null}

      {duplicateSets.length > 0 ? (
        <div className="result-block">
          <strong>{duplicateSets.length} duplicate set(s)</strong>
          <ul>
            {duplicateSets.slice(0, 5).map((set) => (
              <li key={set.blake3}>{set.paths.length} matching files</li>
            ))}
          </ul>
        </div>
      ) : null}
    </div>
  );
}
