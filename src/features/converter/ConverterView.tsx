import { useState } from "react";
import { open } from "@tauri-apps/plugin-dialog";

import {
  ConflictPolicy,
  ConversionFileResult,
  convertImageFiles,
  formatCommandError,
} from "../../lib/tauri-client";

export function ConverterView() {
  const [inputPaths, setInputPaths] = useState<string[]>([]);
  const [outputDirectory, setOutputDirectory] = useState<string | null>(null);
  const [outputFormat, setOutputFormat] = useState("png");
  const [conflictPolicy, setConflictPolicy] = useState<ConflictPolicy>("rename");
  const [results, setResults] = useState<ConversionFileResult[]>([]);
  const [isConverting, setIsConverting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  async function chooseImages() {
    const selected = await open({
      multiple: true,
      filters: [
        {
          name: "Images",
          extensions: ["png", "jpg", "jpeg", "webp", "bmp", "tif", "tiff"],
        },
      ],
    });

    if (Array.isArray(selected)) {
      setInputPaths(selected.filter((path): path is string => typeof path === "string"));
      setResults([]);
      setError(null);
    } else if (typeof selected === "string") {
      setInputPaths([selected]);
      setResults([]);
      setError(null);
    }
  }

  async function chooseOutputDirectory() {
    const selected = await open({ directory: true, multiple: false });
    if (typeof selected === "string") {
      setOutputDirectory(selected);
      setResults([]);
      setError(null);
    }
  }

  async function convertImages() {
    if (inputPaths.length === 0 || !outputDirectory) {
      setError("Choose image files and an output folder before converting.");
      return;
    }

    setIsConverting(true);
    setError(null);
    try {
      const response = await convertImageFiles({
        inputPaths,
        outputDirectory,
        outputFormat,
        conflictPolicy,
      });
      setResults(response.results);
    } catch (conversionError) {
      setError(formatCommandError(conversionError));
    } finally {
      setIsConverting(false);
    }
  }

  return (
    <section className="content-panel" aria-labelledby="converter-heading">
      <p className="eyebrow">Converter</p>
      <h2 id="converter-heading">Convert files locally.</h2>
      <p>
        Start with image conversion. More local backends will appear as they are
        connected.
      </p>

      <div className="workflow-card converter-card">
        <h3>Image conversion</h3>
        <div className="action-row">
          <button type="button" className="primary-button" onClick={chooseImages}>
            Choose images
          </button>
          <button
            type="button"
            className="secondary-button"
            onClick={chooseOutputDirectory}
          >
            Choose output folder
          </button>
        </div>

        <div className="converter-options">
          <label>
            Output format
            <select
              value={outputFormat}
              onChange={(event) => setOutputFormat(event.currentTarget.value)}
            >
              <option value="png">PNG</option>
              <option value="jpg">JPG</option>
              <option value="webp">WebP</option>
              <option value="bmp">BMP</option>
              <option value="tiff">TIFF</option>
            </select>
          </label>
          <label>
            If output exists
            <select
              value={conflictPolicy}
              onChange={(event) => setConflictPolicy(event.currentTarget.value as ConflictPolicy)}
            >
              <option value="rename">Rename new file</option>
              <option value="skip">Skip file</option>
              <option value="overwrite">Overwrite</option>
            </select>
          </label>
        </div>

        <button
          type="button"
          className="primary-button"
          onClick={convertImages}
          disabled={inputPaths.length === 0 || !outputDirectory || isConverting}
        >
          {isConverting ? "Converting..." : "Convert images"}
        </button>

        <p>{inputPaths.length} image file(s) selected.</p>
        {outputDirectory ? <p className="selected-path">{outputDirectory}</p> : null}
      </div>

      {error ? <p className="inline-error">{error}</p> : null}
      {results.length > 0 ? (
        <div className="results-panel" aria-live="polite">
          <h3>Conversion results</h3>
          <ul>
            {results.map((result) => (
              <li key={`${result.inputPath}-${result.outputPath ?? result.status}`}>
                {result.status}: {result.outputPath ?? result.message}
              </li>
            ))}
          </ul>
        </div>
      ) : null}
    </section>
  );
}
