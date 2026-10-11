export interface Settings {
  vaultPath: string;
  dailyFolder: string;
  readHeading: string;
  addHeading: string;
  language: "ko" | "en";
  visibleRows: number;
  fontSize: number;
  reminderFontSize: number;
  displaySeconds: number;
  repeatMinutes: number;
  sound: boolean;
  autostart: boolean;
  iconPath: string;
}
export interface Task {
  id: string;
  title: string;
  done: boolean;
  depth: number;
  parentId: string | null;
  reminderAt: number | null;
}
export interface Alert {
  id: string;
  title: string;
  dueAt: number;
  overdue: boolean;
}
export interface Snapshot {
  settings: Settings;
  date: string;
  tasks: Task[];
  alerts: Alert[];
  rollover: { sourceDate: string; taskIds: string[] } | null;
  error: string | null;
  recoveryPending: boolean;
  fontFamily: string;
  accent: string;
  theme: "dark" | "light" | "system";
}
export type Action =
  | { kind: "configure"; settings: Settings }
  | { kind: "add"; title: string }
  | { kind: "rename"; id: string; title: string }
  | { kind: "complete"; id: string; done: boolean }
  | { kind: "delete"; id: string }
  | { kind: "reorder"; ids: string[] }
  | { kind: "postpone"; id: string; date: string }
  | { kind: "reminders"; selections: Record<string, number> }
  | { kind: "clearReminder"; id: string }
  | { kind: "acknowledge"; ids: string[] }
  | { kind: "rollover"; accept: boolean }
  | { kind: "recover" }
  | { kind: "abandonRecovery" };
