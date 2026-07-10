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
