import { invoke } from "@tauri-apps/api/core";

type CommandResponse<T> = {
  data: T;
};

export type AppStatus = {
  appName: string;
  version: string;
  runtimeNetworkEnabled: boolean;
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
