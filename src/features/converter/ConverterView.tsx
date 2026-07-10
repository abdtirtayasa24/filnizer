import { useState } from "react";
import { open } from "@tauri-apps/plugin-dialog";

import {
  ConflictPolicy,
  ConversionFileResult,
  convertImageFiles,
  convertMarkdownFiles,
  convertMediaFiles,
  convertOfficeFiles,
  convertPdfFiles,
  convertSpreadsheetFiles,
  formatCommandError,
} from "../../lib/tauri-client";

export function ConverterView() {
  const [imagePaths, setImagePaths] = useState<string[]>([]);
  const [spreadsheetPaths, setSpreadsheetPaths] = useState<string[]>([]);
  const [mediaPaths, setMediaPaths] = useState<string[]>([]);
  const [pdfPaths, setPdfPaths] = useState<string[]>([]);
  const [markdownPaths, setMarkdownPaths] = useState<string[]>([]);
  const [officePaths, setOfficePaths] = useState<string[]>([]);
  const [outputDirectory, setOutputDirectory] = useState<string | null>(null);
  const [imageOutputFormat, setImageOutputFormat] = useState("png");
  const [spreadsheetOutputFormat, setSpreadsheetOutputFormat] = useState("xlsx");
  const [mediaOutputFormat, setMediaOutputFormat] = useState("mp3");
  const [pdfOutputFormat, setPdfOutputFormat] = useState("txt");
  const [conflictPolicy, setConflictPolicy] = useState<ConflictPolicy>("rename");
  const [results, setResults] = useState<ConversionFileResult[]>([]);
  const [isConverting, setIsConverting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  async function chooseImages() {
    const selected = await chooseFiles("Images", [
      "png",
      "jpg",
      "jpeg",
      "webp",
      "bmp",
      "tif",
      "tiff",
    ]);
    if (selected) {
      setImagePaths(selected);
      resetResults();
    }
  }

  async function chooseSpreadsheets() {
    const selected = await chooseFiles("Spreadsheets", ["csv", "xlsx"]);
    if (selected) {
      setSpreadsheetPaths(selected);
      resetResults();
    }
  }

  async function chooseMedia() {
    const selected = await chooseFiles("Media", ["mp4", "mov", "mkv", "webm", "mp3", "wav", "flac", "aac", "m4a"]);
    if (selected) {
      setMediaPaths(selected);
      resetResults();
    }
  }

  async function choosePdfs() {
    const selected = await chooseFiles("PDFs", ["pdf"]);
    if (selected) {
      setPdfPaths(selected);
      resetResults();
    }
  }

  async function chooseMarkdown() {
    const selected = await chooseFiles("Markdown", ["md", "markdown"]);
    if (selected) {
      setMarkdownPaths(selected);
      resetResults();
    }
  }

  async function chooseOffice() {
    const selected = await chooseFiles("Office documents", ["doc", "docx", "odt"]);
    if (selected) {
      setOfficePaths(selected);
      resetResults();
    }
  }

  async function chooseFiles(name: string, extensions: string[]) {
    const selected = await open({
      multiple: true,
      filters: [{ name, extensions }],
    });

    if (Array.isArray(selected)) {
      return selected.filter((path): path is string => typeof path === "string");
    }

    if (typeof selected === "string") {
      return [selected];
    }

    return null;
  }

  async function chooseOutputDirectory() {
    const selected = await open({ directory: true, multiple: false });
    if (typeof selected === "string") {
      setOutputDirectory(selected);
      resetResults();
    }
  }

  async function convertImages() {
    await convertFiles(imagePaths, imageOutputFormat, convertImageFiles, "Choose image files and an output folder before converting.");
  }

  async function convertSpreadsheets() {
    await convertFiles(
      spreadsheetPaths,
      spreadsheetOutputFormat,
      convertSpreadsheetFiles,
      "Choose spreadsheet files and an output folder before converting.",
    );
  }

  async function convertMedia() {
    await convertFiles(
      mediaPaths,
      mediaOutputFormat,
      convertMediaFiles,
      "Choose media files and an output folder before converting.",
    );
  }

  async function convertPdfs() {
    await convertFiles(
      pdfPaths,
      pdfOutputFormat,
      convertPdfFiles,
      "Choose PDF files and an output folder before converting.",
    );
  }

  async function convertMarkdown() {
    await convertFiles(
      markdownPaths,
      "pdf",
      convertMarkdownFiles,
      "Choose Markdown files and an output folder before converting.",
    );
  }

  async function convertOffice() {
    await convertFiles(
      officePaths,
      "pdf",
      convertOfficeFiles,
      "Choose Office documents and an output folder before converting.",
    );
  }

  async function convertFiles(
    inputPaths: string[],
    outputFormat: string,
    convert: typeof convertImageFiles,
    missingMessage: string,
  ) {
    if (inputPaths.length === 0 || !outputDirectory) {
      setError(missingMessage);
      return;
    }

    setIsConverting(true);
    setError(null);
    try {
      const response = await convert({
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

  function resetResults() {
    setResults([]);
    setError(null);
  }

  return (
    <section className="content-panel" aria-labelledby="converter-heading">
      <p className="eyebrow">Converter</p>
      <h2 id="converter-heading">Convert files locally.</h2>
      <p>Convert images and simple spreadsheets without uploading files.</p>

      <div className="workflow-card converter-card">
        <h3>Shared output settings</h3>
        <div className="action-row">
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
        {outputDirectory ? <p className="selected-path">{outputDirectory}</p> : null}
      </div>

      <div className="workflow-card converter-card">
        <h3>Image conversion</h3>
        <div className="action-row">
          <button type="button" className="primary-button" onClick={chooseImages}>
            Choose images
          </button>
        </div>
        <div className="converter-options">
          <label>
            Output format
            <select
              value={imageOutputFormat}
              onChange={(event) => setImageOutputFormat(event.currentTarget.value)}
            >
              <option value="png">PNG</option>
              <option value="jpg">JPG</option>
              <option value="webp">WebP</option>
              <option value="bmp">BMP</option>
              <option value="tiff">TIFF</option>
            </select>
          </label>
        </div>
        <button
          type="button"
          className="primary-button"
          onClick={convertImages}
          disabled={imagePaths.length === 0 || !outputDirectory || isConverting}
        >
          {isConverting ? "Converting..." : "Convert images"}
        </button>
        <p>{imagePaths.length} image file(s) selected.</p>
      </div>

      <div className="workflow-card converter-card">
        <h3>Spreadsheet conversion</h3>
        <div className="action-row">
          <button type="button" className="primary-button" onClick={chooseSpreadsheets}>
            Choose CSV/XLSX files
          </button>
        </div>
        <div className="converter-options">
          <label>
            Output format
            <select
              value={spreadsheetOutputFormat}
              onChange={(event) => setSpreadsheetOutputFormat(event.currentTarget.value)}
            >
              <option value="xlsx">CSV to XLSX</option>
              <option value="csv">XLSX to CSV</option>
            </select>
          </label>
        </div>
        <button
          type="button"
          className="primary-button"
          onClick={convertSpreadsheets}
          disabled={spreadsheetPaths.length === 0 || !outputDirectory || isConverting}
        >
          {isConverting ? "Converting..." : "Convert spreadsheets"}
        </button>
        <p>{spreadsheetPaths.length} spreadsheet file(s) selected.</p>
      </div>

      <div className="workflow-card converter-card">
        <h3>Media conversion</h3>
        <p>Requires app-local FFmpeg. Missing FFmpeg is reported without downloading anything.</p>
        <div className="action-row">
          <button type="button" className="primary-button" onClick={chooseMedia}>
            Choose media files
          </button>
        </div>
        <div className="converter-options">
          <label>
            Output format
            <select
              value={mediaOutputFormat}
              onChange={(event) => setMediaOutputFormat(event.currentTarget.value)}
            >
              <option value="mp3">MP3</option>
              <option value="aac">AAC</option>
              <option value="mp4">MP4</option>
              <option value="mkv">MKV</option>
            </select>
          </label>
        </div>
        <button
          type="button"
          className="primary-button"
          onClick={convertMedia}
          disabled={mediaPaths.length === 0 || !outputDirectory || isConverting}
        >
          {isConverting ? "Converting..." : "Convert media"}
        </button>
        <p>{mediaPaths.length} media file(s) selected.</p>
      </div>

      <div className="workflow-card converter-card">
        <h3>PDF conversion</h3>
        <p>Requires app-local Pdfium. Missing Pdfium is reported without downloading anything.</p>
        <div className="action-row">
          <button type="button" className="primary-button" onClick={choosePdfs}>
            Choose PDFs
          </button>
        </div>
        <div className="converter-options">
          <label>
            Output format
            <select
              value={pdfOutputFormat}
              onChange={(event) => setPdfOutputFormat(event.currentTarget.value)}
            >
              <option value="txt">Text</option>
              <option value="png">PNG images</option>
            </select>
          </label>
        </div>
        <button
          type="button"
          className="primary-button"
          onClick={convertPdfs}
          disabled={pdfPaths.length === 0 || !outputDirectory || isConverting}
        >
          {isConverting ? "Converting..." : "Convert PDFs"}
        </button>
        <p>{pdfPaths.length} PDF file(s) selected.</p>
      </div>

      <div className="workflow-card converter-card">
        <h3>Markdown to PDF</h3>
        <p>Remote URLs are rejected so conversion stays offline.</p>
        <div className="action-row">
          <button type="button" className="primary-button" onClick={chooseMarkdown}>
            Choose Markdown
          </button>
        </div>
        <button
          type="button"
          className="primary-button"
          onClick={convertMarkdown}
          disabled={markdownPaths.length === 0 || !outputDirectory || isConverting}
        >
          {isConverting ? "Converting..." : "Convert Markdown"}
        </button>
        <p>{markdownPaths.length} Markdown file(s) selected.</p>
      </div>

      <div className="workflow-card converter-card">
        <h3>Office to PDF</h3>
        <p>Requires a local LibreOffice installation. Filnizer only detects it; it does not download it.</p>
        <div className="action-row">
          <button type="button" className="primary-button" onClick={chooseOffice}>
            Choose Office documents
          </button>
        </div>
        <button
          type="button"
          className="primary-button"
          onClick={convertOffice}
          disabled={officePaths.length === 0 || !outputDirectory || isConverting}
        >
          {isConverting ? "Converting..." : "Convert Office files"}
        </button>
        <p>{officePaths.length} Office document(s) selected.</p>
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
