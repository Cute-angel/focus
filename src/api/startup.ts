import { invoke } from "@tauri-apps/api/core";

export interface StartupStatus {
  onboardingCompleted: boolean;
  currentHotkey: string;
}

export interface HotkeyResponse {
  hotkey: string;
}

async function getStartupStatus(): Promise<StartupStatus> {
  return invoke<StartupStatus>("get_startup_status");
}

async function saveGlobalHotkey(accelerator: string): Promise<HotkeyResponse> {
  return invoke<HotkeyResponse>("set_global_hotkey", { accelerator });
}

export { getStartupStatus, saveGlobalHotkey };
