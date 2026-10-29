import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { AccountInfo, KeyInfo, Settings, StatusView } from "./types";

export const getStatus = () => invoke<StatusView>("get_status");
export const checkNow = () => invoke<void>("check_now");
export const saveKey = (key: string) => invoke<AccountInfo>("save_key", { key });
export const removeKey = () => invoke<void>("remove_key");
export const getKeyInfo = () => invoke<KeyInfo>("get_key_info");
export const getSettings = () => invoke<Settings>("get_settings");
export const saveSettings = (settings: Settings) => invoke<Settings>("save_settings", { settings });
export const setWidgetSize = (width: number, height: number) =>
  invoke<void>("set_widget_size", { width, height });
export const mute = (minutes: number) => invoke<void>("mute", { minutes });
export const unmute = () => invoke<void>("unmute");
export const openMonitor = (id: number) => invoke<void>("open_monitor", { id });
export const openDashboard = () => invoke<void>("open_dashboard");
export const openApiKeysPage = () => invoke<void>("open_api_keys_page");
export const openSettings = () => invoke<void>("open_settings");
export const openRepository = () => invoke<void>("open_repository");

/** Calls `handler` with the current status and on every change. */
export async function subscribeStatus(handler: (status: StatusView) => void): Promise<UnlistenFn> {
  const unlisten = await listen<StatusView>("status-changed", (event) => handler(event.payload));
  handler(await getStatus());
  return unlisten;
}
