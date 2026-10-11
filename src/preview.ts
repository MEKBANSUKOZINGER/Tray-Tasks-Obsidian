import type { Action, Settings, Snapshot } from "./types";
const settings: Settings = {
  vaultPath: "Preview vault",
  dailyFolder: "Daily",
  readHeading: "",
  addHeading: "",
  language: "ko",
  visibleRows: 6,
  fontSize: 14,
  reminderFontSize: 15,
  displaySeconds: 60,
  repeatMinutes: 5,
  sound: true,
  autostart: false,
  iconPath: "",
};
let state: Snapshot = {
  settings,
  date: "2026-10-11",
  fontFamily: "Segoe UI",
  accent: "#147d64",
  error: null,
  recoveryPending: false,
  rollover: null,
  alerts: [],
  tasks: [
    {
      id: "a",
      title: "Tray Tasks 첫 번째 빌드 확인하기",
      done: false,
      depth: 0,
      parentId: null,
      reminderAt: null,
    },
    {
      id: "b",
      title: "데일리 노트의 할 일 정리",
      done: false,
      depth: 0,
      parentId: null,
      reminderAt: Date.now() / 1000 + 1200,
    },
    {
      id: "c",
      title: "자정 이월 동작 테스트하기",
      done: false,
      depth: 1,
      parentId: "b",
      reminderAt: null,
    },
    {
      id: "d",
      title: "GitHub README 초안 작성",
      done: false,
      depth: 0,
      parentId: null,
      reminderAt: null,
    },
    {
      id: "e",
      title: "프로젝트 개발 환경 준비",
      done: true,
      depth: 0,
      parentId: null,
      reminderAt: null,
    },
  ],
};
export function previewSnapshot(): Snapshot {
  return structuredClone(state);
}
export function previewAction(action: Action): Snapshot {
  switch (action.kind) {
    case "configure":
      state.settings = action.settings;
      break;
    case "add":
      state.tasks.push({
        id: crypto.randomUUID(),
        title: action.title,
        done: false,
        depth: 0,
        parentId: null,
        reminderAt: null,
      });
      break;
    case "complete": {
      const task = state.tasks.find((t) => t.id === action.id)!;
      task.done = action.done;
      if (task.done) task.reminderAt = null;
      break;
    }
    case "rename":
      state.tasks.find((t) => t.id === action.id)!.title = action.title;
      break;
    case "delete":
    case "postpone":
      state.tasks = state.tasks.filter(
        (t) => t.id !== action.id && t.parentId !== action.id,
      );
      break;
    case "reminders":
      for (const [id, due] of Object.entries(action.selections))
        state.tasks.find((t) => t.id === id)!.reminderAt = due || null;
      break;
    case "clearReminder":
      state.tasks.find((t) => t.id === action.id)!.reminderAt = null;
      break;
    case "reorder":
      state.tasks.sort(
        (a, b) => action.ids.indexOf(a.id) - action.ids.indexOf(b.id),
      );
      break;
    case "acknowledge":
      state.alerts = state.alerts.filter((a) => !action.ids.includes(a.id));
      break;
    case "rollover":
      state.rollover = null;
      break;
  }
  return previewSnapshot();
}
