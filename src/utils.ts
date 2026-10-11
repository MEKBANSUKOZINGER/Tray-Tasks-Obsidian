import type { Task } from "./types";
export function escapeHtml(value: string): string {
  return value.replace(
    /[&<>"']/g,
    (c) =>
      ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" })[
        c
      ]!,
  );
}
export function dateKey(date: Date): string {
  return `${date.getFullYear()}-${String(date.getMonth() + 1).padStart(2, "0")}-${String(date.getDate()).padStart(2, "0")}`;
}
export function addDays(key: string, days: number): string {
  const [year, month, day] = key.split("-").map(Number);
  const date = new Date(year, month - 1, day, 12);
  date.setDate(date.getDate() + days);
  return dateKey(date);
}
export function localInput(date: Date): string {
  return `${dateKey(date)}T${String(date.getHours()).padStart(2, "0")}:${String(date.getMinutes()).padStart(2, "0")}`;
}
export function reminderBounds(now = new Date()): { min: string; max: string } {
  const next = new Date(now);
  next.setDate(next.getDate() + 1);
  next.setHours(23, 59, 0, 0);
  const rounded = new Date(Math.ceil(now.getTime() / 60000) * 60000);
  return { min: localInput(rounded), max: localInput(next) };
}

export function moveTask(
  tasks: Task[],
  fromId: string,
  toId: string,
  after = false,
): string[] {
  const ids = tasks.map((task) => task.id);
  if (fromId === toId || !ids.includes(fromId) || !ids.includes(toId))
    return ids;
  const rest = ids.filter((id) => id !== fromId);
  rest.splice(rest.indexOf(toId) + Number(after), 0, fromId);
  return rest;
}

export function reminderOptions(day: string, hour: number, now = new Date()) {
  const bounds = reminderBounds(now);
  const min = new Date(bounds.min),
    max = new Date(bounds.max);
  const validDay = day === dateKey(now) || day === dateKey(max);
  const hours = Array.from({ length: 24 }, (_, i) => i).filter((h) => {
    const end = new Date(day + "T" + String(h).padStart(2, "0") + ":59");
    return validDay && end >= min && end <= max;
  });
  const minutes = Array.from({ length: 60 }, (_, i) => i).filter((m) => {
    const time = new Date(
      day +
        "T" +
        String(hour).padStart(2, "0") +
        ":" +
        String(m).padStart(2, "0"),
    );
    return validDay && time >= min && time <= max;
  });
  return { hours, minutes };
}
export function validReminder(due: number, now = new Date()) {
  const max = new Date(reminderBounds(now).max).getTime() / 1000;
  return Number.isFinite(due) && due >= now.getTime() / 1000 && due <= max;
}
export function futureDate(day: string, today: string) {
  return (
    /^\d{4}-\d{2}-\d{2}$/.test(day) &&
    dateKey(new Date(day + "T12:00:00")) === day &&
    day > today
  );
}

export function calendarDays(
  year: number,
  month: number,
): Array<string | null> {
  const first = new Date(year, month, 1, 12),
    last = new Date(year, month + 1, 0, 12);
  return [
    ...Array.from({ length: first.getDay() }, () => null),
    ...Array.from({ length: last.getDate() }, (_, i) =>
      dateKey(new Date(year, month, i + 1, 12)),
    ),
  ];
}
