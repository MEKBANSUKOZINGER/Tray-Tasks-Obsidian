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
