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
export function moveTaskGroup(
  tasks: Task[],
  fromId: string,
  toId: string,
): string[] {
  const from = tasks.find((t) => t.id === fromId),
    to = tasks.find((t) => t.id === toId);
  if (!from || !to || fromId === toId || from.parentId !== to.parentId)
    return tasks.map((t) => t.id);
  const group = new Set<string>([fromId]);
  for (const task of tasks)
    if (task.parentId && group.has(task.parentId)) group.add(task.id);
  const moved = tasks.filter((t) => group.has(t.id));
  const rest = tasks.filter((t) => !group.has(t.id));
  let index = rest.findIndex((t) => t.id === toId);
  if (tasks.indexOf(from) < tasks.indexOf(to)) {
    const targetGroup = new Set<string>([toId]);
    for (const task of rest)
      if (task.parentId && targetGroup.has(task.parentId))
        targetGroup.add(task.id);
    while (index < rest.length && targetGroup.has(rest[index].id)) index++;
  }
  rest.splice(index, 0, ...moved);
  return rest.map((t) => t.id);
}
