import type { Lang } from "./i18n";

export type Level = "noKey" | "unknown" | "ok" | "warning" | "down" | "error";
export type ErrorKind = "invalidKey" | "network" | "rateLimited" | "unexpected";

export interface Monitor {
  id: number;
  name: string;
  url: string | null;
  status: "down" | "looksDown";
  reason: string | null;
  downSince: string | null;
}

export interface StatusView {
  level: Level;
  alerts: Monitor[];
  warnings: Monitor[];
  error: ErrorKind | null;
  lastCheck: string | null;
  mutedUntil: string | null;
  checking: boolean;
  widgetVisible: boolean;
}

export interface Settings {
  intervalMinutes: number;
  countLooksDown: boolean;
  showWhenOk: boolean;
  widgetWidth: number;
  widgetHeight: number;
  /** null follows the system language. */
  language: Lang | null;
}

export interface AccountInfo {
  email: string | null;
  monitorsCount: number;
}

export interface KeyInfo {
  hasKey: boolean;
  maskedKey: string | null;
}
