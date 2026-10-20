const relative = new Intl.RelativeTimeFormat(undefined, { numeric: "auto" });

export function timeAgo(pickedAt: number, now = Date.now()): string {
  const seconds = Math.round((pickedAt - now) / 1000);
  if (Math.abs(seconds) < 60) return relative.format(seconds, "second");
  const minutes = Math.round(seconds / 60);
  if (Math.abs(minutes) < 60) return relative.format(minutes, "minute");
  const hours = Math.round(minutes / 60);
  if (Math.abs(hours) < 24) return relative.format(hours, "hour");
  return relative.format(Math.round(hours / 24), "day");
}
