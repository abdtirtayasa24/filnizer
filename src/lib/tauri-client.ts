import { invoke } from "@tauri-apps/api/core";

type CommandResponse<T> = {
  data: T;
};

export type AppStatus = {
  appName: string;
  version: string;
  runtimeNetworkEnabled: boolean;
};

export type FileCategory =
  | "images"
  | "documents"
  | "pdfs"
  | "spreadsheets"
  | "presentations"
  | "videos"
  | "audio"
  | "archives"
  | "code"
  | "executables"
  | "other";

export type FileEntry = {
  path: string;
  name: string;
  extension: string | null;
  sizeBytes: number;
  modifiedUnixMs: number | null;
  category: FileCategory;
  hashStatus: unknown;
};

export type OrganizerRule = {
  id: string;
  kind: "extension" | "filenameContains";
  value: string;
  category: FileCategory;
};

export type OrganizerRuleInput = Omit<OrganizerRule, "id"> & {
  id?: string;
};

export type StartScanRequest = {
  roots: string[];
  recursive: boolean;
  includeHidden: boolean;
};

export type StartScanResponse = {
  jobId: string;
  files: FileEntry[];
};

export type ScanProgressEvent = {
  jobId: string;
  currentPath: string;
  scannedFiles: number;
};

export type OperationPlan = {
  id: string;
  jobId: string | null;
  status: "preview" | "applying" | "applied" | "failed" | "undone";
  operations: PlannedOperation[];
};

export type PlannedOperation = {
  id: string;
  kind: "move" | "rename" | "convert";
  sourcePath: string;
  targetPath: string;
  conflictPolicy: "skip" | "rename" | "overwrite";
};

export type ApplyFileResult = {
  operationId: string;
  sourcePath: string;
  targetPath: string | null;
  status: "success" | "failed" | "skipped";
  message: string | null;
};

export type ApplyOrganizerPlanResponse = {
  jobId: string;
  planId: string;
  results: ApplyFileResult[];
};

export type DuplicateSet = {
  sizeBytes: number;
  blake3: string;
  paths: string[];
};

export type ConflictPolicy = "skip" | "rename" | "overwrite";

export type ConversionFileResult = {
  inputPath: string;
  outputPath: string | null;
  status: "pending" | "running" | "completed" | "failed" | "skipped";
  message: string | null;
};

export type ConversionRequest = {
  inputPaths: string[];
  outputDirectory: string;
  outputFormat: string;
  conflictPolicy: ConflictPolicy;
};

export type ConversionResponse = {
  jobId: string;
  results: ConversionFileResult[];
};

export async function invokeCommand<T>(
  command: string,
  args?: Record<string, unknown>,
): Promise<T> {
  const response = await invoke<CommandResponse<T>>(command, args);
  return response.data;
}

export function getAppStatus(): Promise<AppStatus> {
  return invokeCommand<AppStatus>("get_app_status");
}

export function startOrganizerScan(
  request: StartScanRequest,
): Promise<StartScanResponse> {
  return invokeCommand<StartScanResponse>("start_organizer_scan", { request });
}

export function listOrganizerRules(): Promise<OrganizerRule[]> {
  return invokeCommand<OrganizerRule[]>("list_organizer_rules");
}

export function saveOrganizerRules(
  rules: OrganizerRuleInput[],
): Promise<OrganizerRule[]> {
  return invokeCommand<OrganizerRule[]>("save_organizer_rules", {
    request: { rules },
  });
}

export function previewOrganizerPlan(
  files: FileEntry[],
  destinationRoot: string,
): Promise<OperationPlan> {
  return invokeCommand<OperationPlan>("preview_organizer_plan_command", {
    request: { files, destinationRoot, cleanFilenames: true },
  });
}

export function applyOrganizerPlan(
  plan: OperationPlan,
): Promise<ApplyOrganizerPlanResponse> {
  return invokeCommand<ApplyOrganizerPlanResponse>("apply_organizer_plan_command", {
    request: { plan },
  });
}

export function undoOrganizerPlan(
  plan: OperationPlan,
): Promise<ApplyOrganizerPlanResponse> {
  return invokeCommand<ApplyOrganizerPlanResponse>("undo_organizer_plan_command", {
    request: { plan },
  });
}

export function findDuplicateFiles(files: FileEntry[]): Promise<{ sets: DuplicateSet[] }> {
  return invokeCommand<{ sets: DuplicateSet[] }>("find_duplicate_files", {
    request: { files },
  });
}

export function convertImageFiles(
  request: ConversionRequest,
): Promise<ConversionResponse> {
  return invokeCommand<ConversionResponse>("convert_image_files", { request });
}

export function convertSpreadsheetFiles(
  request: ConversionRequest,
): Promise<ConversionResponse> {
  return invokeCommand<ConversionResponse>("convert_spreadsheet_files", { request });
}

export function formatCommandError(error: unknown): string {
  if (typeof error === "string") {
    return error;
  }

  if (
    error &&
    typeof error === "object" &&
    "message" in error &&
    typeof error.message === "string"
  ) {
    return error.message;
  }

  return "The app command could not be completed.";
}
