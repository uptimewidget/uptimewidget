import { lang, t } from "./i18n";

/** Compact duration like "13 T 16 Std" / "13d 16h". */
export function durationSince(iso: string | null, now: number): string | null {
  if (!iso) return null;
  const minutes = Math.max(0, Math.floor((now - Date.parse(iso)) / 60000));
  const days = Math.floor(minutes / 1440);
  const hours = Math.floor(minutes / 60) % 24;
  const mins = minutes % 60;
  const { d, h, m } = t.duration;
  if (days > 0) return `${days}${d} ${hours}${h}`;
  if (hours > 0) return `${hours}${h} ${mins}${m}`;
  return `${mins}${m}`;
}

export function clockTime(iso: string | null): string | null {
  if (!iso) return null;
  return new Date(iso).toLocaleTimeString(lang, {
    hour: "2-digit",
    minute: "2-digit",
  });
}

/** "502 Bad Gateway · seit 13 T 16 Std" */
export function monitorDetail(reason: string | null, downSince: string | null, now: number): string {
  const duration = durationSince(downSince, now);
  return [reason, duration && `${t.since} ${duration}`].filter(Boolean).join(" · ");
}
