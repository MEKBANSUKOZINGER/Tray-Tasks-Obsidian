import "./style.css";
import { invoke } from "@tauri-apps/api/core";
import { listen, emitTo } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { open } from "@tauri-apps/plugin-dialog";
import type { Action, Settings, Snapshot, Task } from "./types";
import {
  addDays,
  escapeHtml as esc,
  dateKey,
  reminderOptions,
  validReminder,
  futureDate,
  reminderBounds,
  moveTask,
} from "./utils";
import { previewAction, previewSnapshot } from "./preview";
const preview = new URLSearchParams(location.search).has("preview");
const isReminder =
  new URLSearchParams(location.search).get("window") === "reminder";
const isPopup = new URLSearchParams(location.search).get("window") === "popup";
const app = document.querySelector<HTMLDivElement>("#app")!;
let snapshot: Snapshot;
let view: "tasks" | "completed" | "reminders" | "settings" = "tasks";
let draft: Settings | null = null,
  reminderDraft: Record<string, number> = {},
  editing: string | null = null,
  busy = false;
type Modal = {
  kind: "delete" | "postpone" | "time" | "abandon";
  id?: string;
  date?: string;
};
let modal: Modal | null = null;
let coreConfirmed = false;
let adding = false,
  addText = "",
  popupToken = 0;
let openedModal: Modal | null = null;
let drag: {
  id: string;
  pointer: number;
  x: number;
  y: number;
  started: boolean;
  ids: string[];
  handle: HTMLElement;
  ghost?: HTMLElement;
  frame?: number;
  lastY: number;
  offsetY: number;
} | null = null;
let suppressClick = false;
let pendingSnapshot: Snapshot | null = null;
let reordering = false;
let popupHeight = 0;
let modalError = "",
  rendering = false;
let pointerPressed = false,
  pendingInline = false;
let inlineCommit: Promise<boolean> | null = null;
let timeDraft: { day: string; hour: number; minute: number } | null = null;
const wheelTimers = new Map<string, ReturnType<typeof setTimeout>>();
interface PopupContext {
  token: number;
  modal: Modal;
  snapshot: Snapshot;
  view: typeof view;
  reminderDraft: Record<string, number>;
}

let toast = "",
  toastTimer: ReturnType<typeof setTimeout>;
const labels = {
  ko: {
    today: "오늘 할 일",
    completed: "완료한 일",
    reminders: "리마인더 설정",
    settings: "설정",
    back: "돌아가기",
    daily: "데일리 노트",
    reminder: "리마인더",
    apply: "적용",
    cancel: "취소",
    save: "저장",
    add: "새 할 일 추가…",
    empty: "할 일이 없어요",
    emptySub: "오늘의 데일리 노트에서 할 일을 가져옵니다.",
    completedEmpty: "완료한 일이 없어요",
    reminderHint: "할 일을 선택하고 알림 시간을 정하세요.",
    delete: "삭제",
    postpone: "미루기",
    rename: "이름 수정",
    deleteTitle: "이 할 일을 삭제할까요?",
    deleteNote: "",
    tomorrow: "내일",
    nextWeek: "다음 주 같은 날",
    otherDate: "다른 날",
    move: "이 날짜로 미루기",
    chooseTime: "리마인더 시간",
    timeHint: "현재 시각부터 내일 23:59까지 설정할 수 있어요.",
    clearReminder: "리마인더 해제",
    welcome: "오늘 할 일, 클릭 한 번으로.",
    welcomeSub: "옵시디언을 열지 않아도 데일리 노트의 할 일을 관리하세요.",
    vault: "옵시디언 볼트 폴더",
    browse: "찾아보기",
    folder: "데일리 노트 폴더",
    folderHint: "비워두면 볼트의 최상위 폴더에 저장해요.",
    coreHint:
      "옵시디언 → 설정 → 코어 플러그인 → 데일리 노트를 켜고, 날짜 형식을 YYYY-MM-DD로 설정해주세요.",
    coreConfirm: "데일리 노트를 켰고 날짜 형식과 저장 위치를 확인했어요.",
    language: "언어",
    readHeading: "읽을 제목",
    readHint: "비워두면 노트 전체의 체크박스를 읽어요.",
    addHeading: "추가할 제목",
    addHint: "비워두면 문서 맨 마지막에 추가해요.",
    rows: "한 화면에 표시할 개수",
    font: "목록 글자 크기",
    reminderFont: "알림 글자 크기",
    display: "알림 표시 시간 (초)",
    repeat: "다시 알릴 간격 (분)",
    sound: "리마인더 소리",
    autostart: "Windows 로그인 시 자동 실행",
    icon: "사용자 트레이 아이콘",
    resetIcon: "기본 아이콘",
    localOnly: "노트는 볼트에, 설정과 알림은 이 컴퓨터에 저장됩니다.",
    connect: "연결하고 시작하기",
    ready: "볼트가 연결되었어요.",
    acknowledge: "확인",
    overdue: "지난 리마인더",
    rolloverTitle: "새로운 하루가 시작됐어요.",
    rolloverBody: "전날의 미완료 할 일을 오늘로 가져올까요?",
    yes: "예, 가져오기",
    no: "아니오",
    rolloverNote: "가져오면 전날 항목은 완료되고 예약된 알림은 유지돼요.",
    refresh: "새로고침",
    recover: "이동 복구 재시도",
    abandon: "복구 기록 해제",
    abandonNote:
      "복구 기록만 해제합니다. 이미 복사된 대상 항목은 남으므로 원본과 대상 노트를 확인해주세요.",
    error: "작업을 완료하지 못했어요",
    data: "저장·백업 폴더 열기",
    version: "개발 버전 0.3.0",
    editHint: "우클릭하거나 F2를 눌러 이름 수정",
    count: "개",
  },
  en: {
    today: "Today's tasks",
    completed: "Completed",
    reminders: "Set reminders",
    settings: "Settings",
    back: "Back",
    daily: "Daily note",
    reminder: "Reminders",
    apply: "Apply",
    cancel: "Cancel",
    save: "Save",
    add: "Add a task…",
    empty: "All clear for today",
    emptySub: "Tasks are read from your daily note.",
    completedEmpty: "No completed tasks yet",
    reminderHint: "Choose a task to set its reminder.",
    delete: "Delete",
    postpone: "Postpone",
    rename: "Rename",
    deleteTitle: "Delete this task?",
    deleteNote: "Nested items will also be removed from the daily note.",
    tomorrow: "Tomorrow",
    nextWeek: "Same day next week",
    otherDate: "Another date",
    move: "Move to this date",
    chooseTime: "Reminder time",
    timeHint: "Choose a time between now and tomorrow at 23:59.",
    clearReminder: "Remove reminder",
    welcome: "Your day, one click away.",
    welcomeSub: "Manage daily-note tasks without opening Obsidian.",
    vault: "Obsidian vault folder",
    browse: "Browse",
    folder: "Daily note folder",
    folderHint: "Leave blank to use the vault root.",
    coreHint:
      "Enable Obsidian → Settings → Core plugins → Daily notes. Set the date format to YYYY-MM-DD.",
    coreConfirm:
      "Daily notes is enabled. I've checked the date format and folder.",
    language: "Language",
    readHeading: "Read under heading",
    readHint: "Leave blank to read all checkboxes in the note.",
    addHeading: "Add under heading",
    addHint: "Leave blank to append to the end of the note.",
    rows: "Visible task rows",
    font: "Task font size",
    reminderFont: "Reminder font size",
    display: "Notification duration (seconds)",
    repeat: "Repeat interval (minutes)",
    sound: "Reminder sound",
    autostart: "Start when signing in to Windows",
    icon: "Custom tray icon",
    resetIcon: "Default icon",
    localOnly:
      "Notes stay in your vault. Preferences and reminders stay on this computer.",
    connect: "Connect and start",
    ready: "Vault connected.",
    acknowledge: "OK",
    overdue: "Missed reminder",
    rolloverTitle: "A new day has started.",
    rolloverBody: "Bring unfinished tasks from the previous day into today?",
    yes: "Yes, bring them",
    no: "No",
    rolloverNote:
      "Yesterday's items become completed. Reminder times stay the same.",
    refresh: "Refresh",
    recover: "Retry move recovery",
    abandon: "Clear recovery record",
    abandonNote:
      "Only the recovery record is cleared. Already copied tasks remain. Check both notes.",
    error: "Couldn't complete the action",
    data: "Open storage and backups",
    version: "Development build 0.3.0",
    editHint: "Right-click or press F2 to rename",
    count: "items",
  },
};
type Label = keyof typeof labels.ko;
const t = (key: Label) =>
  labels[draft?.language ?? snapshot?.settings.language ?? "ko"][key];
const svg = (name: string) => {
  const paths: Record<string, string> = {
    check: '<path d="m5 12 4 4L19 6"/>',
    calendar:
      '<rect x="3" y="5" width="18" height="16" rx="2"/><path d="M16 3v4M8 3v4M3 11h18"/>',
    clock: '<circle cx="12" cy="12" r="9"/><path d="M12 7v5l3 2"/>',
    gear: '<path d="m9 3-1 3-3 1-2 5 2 5 3 1 1 3h6l1-3 3-1 2-5-2-5-3-1-1-3Z"/><circle cx="12" cy="12" r="3"/>',
    back: '<path d="m14 5-7 7 7 7"/>',
    plus: '<path d="M12 5v14M5 12h14"/>',
    close: '<path d="m6 6 12 12M18 6 6 18"/>',
    trash: '<path d="M3 6h18M9 6V3h6v3M6 6l1 15h10l1-15M10 10v7M14 10v7"/>',
    next: '<path d="m9 5 7 7-7 7"/>',
    grip: '<circle cx="9" cy="6" r="1"/><circle cx="15" cy="6" r="1"/><circle cx="9" cy="12" r="1"/><circle cx="15" cy="12" r="1"/><circle cx="9" cy="18" r="1"/><circle cx="15" cy="18" r="1"/>',
    folder: '<path d="M3 7V5h6l2 2h10v13H3Z"/>',
    refresh: '<path d="M20 10a8 8 0 1 0-2 8M20 4v6h-6"/>',
  };
  return `<svg viewBox="0 0 24 24" aria-hidden="true">${paths[name] ?? paths.check}</svg>`;
};
const button = (action: string, label: string, glyph?: string, cls = "") =>
  `<button type="button" data-action="${action}" class="${cls}" title="${esc(label)}">${glyph ? svg(glyph) : ""}<span>${esc(label)}</span></button>`;
const fmtTime = (unix: number) =>
  new Date(unix * 1000).toLocaleString(
    snapshot.settings.language === "ko" ? "ko-KR" : "en-US",
    { month: "numeric", day: "numeric", hour: "2-digit", minute: "2-digit" },
  );
const fmtDate = (key: string) =>
  new Date(key + "T12:00:00").toLocaleDateString(
    snapshot.settings.language === "ko" ? "ko-KR" : "en-US",
    { year: "2-digit", month: "2-digit", day: "2-digit", weekday: "short" },
  );
function notify(message: string) {
  toast = message;
  clearTimeout(toastTimer);
  render();
  toastTimer = setTimeout(() => {
    toast = "";
    render();
  }, 5000);
}
async function dispatch(action: Action, onError?: (message: string) => void) {
  if (busy) return false;
  busy = true;
  render();
  try {
    snapshot = preview
      ? previewAction(action)
      : await invoke<Snapshot>("action", { action });
    return true;
  } catch (e) {
    if (onError) onError(String(e));
    else notify(String(e));
    return false;
  } finally {
    busy = false;
    render();
  }
}
async function refresh() {
  if (!snapshot) {
    await start();
    return;
  }
  try {
    snapshot = preview
      ? previewSnapshot()
      : await invoke<Snapshot>("get_snapshot");
    render();
  } catch (e) {
    notify(String(e));
  }
}
function setView(next: typeof view) {
  const list = app.querySelector(".task-list");
  if (list) list.scrollTop = 0;
  view = next;
  modal = null;
  modalError = "";
  timeDraft = null;
  adding = false;
  addText = "";
  editing = null;
  reminderDraft = {};
  draft = next === "settings" ? structuredClone(snapshot.settings) : null;
  render();
}
function errorBanner() {
  return snapshot.error
    ? `<aside class="error"><strong>${t("error")}</strong><p>${esc(snapshot.error)}</p><div>${button("refresh", t("refresh"))}${snapshot.recoveryPending ? button("recover", t("recover")) + button("abandon", t("abandon")) : ""}</div></aside>`
    : "";
}
function taskRow(task: Task) {
  const pending = reminderDraft[task.id];
  const effective = Object.hasOwn(reminderDraft, task.id)
    ? pending
    : task.reminderAt;
  return `<div class="task ${task.done ? "done" : ""}  ${pending ? "selected" : ""}" data-id="${esc(task.id)}">
<button class="grip icon-button"  data-drag="${esc(task.id)}" aria-label="Reorder ${esc(task.title)}">${svg("grip")}</button>
${view === "reminders" ? `<button class="selection-dot ${effective ? "active" : ""}" data-task-action="time" aria-label="${t("chooseTime")}">${svg("clock")}</button>` : `<input type="checkbox" data-check aria-label="${esc(task.title)}" ${task.done ? "checked" : ""}/>`}
<div class="task-content">${editing === task.id ? `<input id="rename-input" class="rename-input" value="${esc(task.title)}" aria-label="${t("rename")}"/>` : `<button class="task-name" data-task-action="${view === "reminders" ? "time" : "edit"}" title="${t("editHint")}">${esc(task.title)}</button>`}
${effective ? `<button class="task-time" data-task-action="time" title="${esc(fmtTime(effective!))}" aria-label="${esc(fmtTime(effective!))}">${svg("clock")}</button>` : ""}</div>
${view !== "reminders" ? `<div class="task-actions"><button class="icon-button danger" data-task-action="delete" title="${t("delete")}" aria-label="${t("delete")} ${esc(task.title)}">${svg("trash")}</button>${!task.done ? `<button class="icon-button" data-task-action="postpone" title="${t("postpone")}" aria-label="${t("postpone")} ${esc(task.title)}">${svg("next")}</button>` : ""}</div>` : ""}</div>`;
}
function listView() {
  const done = view === "completed";
  const tasks = snapshot.tasks.filter((task) => task.done === done);
  const add =
    !done && view !== "reminders"
      ? adding
        ? `<form id="add-form" class="task new-task"><input type="checkbox" disabled/><input id="add-input" name="title" value="${esc(addText)}" autocomplete="off" aria-label="${t("add")}" required/><button type="submit" aria-label="${t("save")}">↵</button></form>`
        : button("add", t("add"), "plus", "add-button")
      : "";
  return `${errorBanner()}<main class="task-list">${tasks.map(taskRow).join("")}${add}${!tasks.length && !add ? `<div class="empty">${done ? t("completedEmpty") : t("empty")}</div>` : ""}</main>
  <footer>${
    view === "reminders"
      ? button("back", t("cancel"), "back") +
        button("apply-reminders", t("apply"), "check", "primary")
      : button(
          done ? "back" : "completed",
          done ? t("back") : t("completed"),
          done ? "back" : "check",
          "icon-button",
        ) +
        button("daily", t("daily"), "calendar", "icon-button") +
        button("reminders", t("reminder"), "clock", "icon-button") +
        button("settings", t("settings"), "gear", "icon-button") +
        button("hide", t("cancel"), "close", "icon-button")
  }</footer>`;
}
function field(key: keyof Settings, label: Label, hint?: Label, type = "text") {
  return `<label class="field"><span>${t(label)}</span><input id="setting-${key}" name="${key}" type="${type}" value="${esc(String(draft![key]))}"/>${hint ? `<small>${t(hint)}</small>` : ""}</label>`;
}
function settingsView() {
  const first = !snapshot.settings.vaultPath;
  draft ??= structuredClone(snapshot.settings);
  return `<form id="settings-form" class="settings-form">

<label class="field"><span>${t("language")}</span><select id="setting-language" name="language"><option value="ko" ${draft.language === "ko" ? "selected" : ""}>한국어</option><option value="en" ${draft.language === "en" ? "selected" : ""}>English</option></select></label>
<label class="field"><span>${t("vault")}</span><div class="path-input"><input id="setting-vaultPath" name="vaultPath" value="${esc(draft.vaultPath)}" required placeholder="C:\\Vaults\\My Vault"/>${button("browse-vault", t("browse"), "folder")}</div></label>
${field("dailyFolder", "folder")}<div class="setup-note">${svg("calendar")}<p>${t("coreHint")}</p></div>
${first ? `<label class="toggle core-confirm"><input name="coreConfirm" type="checkbox" ${coreConfirmed ? "checked" : ""} required/><span>${t("coreConfirm")}</span></label>` : ""}
${field("readHeading", "readHeading")}${field("addHeading", "addHeading")}
<div class="field-grid">${field("visibleRows", "rows", undefined, "number")}${field("fontSize", "font", undefined, "number")}${field("reminderFontSize", "reminderFont", undefined, "number")}${field("displaySeconds", "display", undefined, "number")}${field("repeatMinutes", "repeat", undefined, "number")}</div>
<label class="toggle"><input name="sound" type="checkbox" ${draft.sound ? "checked" : ""}/><span>${t("sound")}</span></label>
<label class="toggle"><input name="autostart" type="checkbox" ${draft.autostart ? "checked" : ""}/><span>${t("autostart")}</span></label>
<div class="field"><span>${t("icon")}</span><small>${esc(draft.iconPath || t("resetIcon"))}</small><div class="inline-buttons">${button("browse-icon", t("browse"), "folder")}${button("reset-icon", t("resetIcon"))}</div></div>
${!first ? button("data", t("data"), "folder", "data-button") : ""}${button("taskbar-settings", snapshot.settings.language === "ko" ? "작업표시줄 아이콘 설정" : "Taskbar icon settings", undefined, "data-button")}
<div class="settings-buttons">${!first ? button("back", t("cancel")) : ""}<button type="submit" class="primary">${first ? t("connect") : t("save")}</button></div></form>`;
}
function modalView() {
  if (!modal) return "";
  const task = snapshot.tasks.find((t) => t.id === modal!.id);
  let content = "";
  if (modal.kind === "delete")
    content = `<h2>${t("deleteTitle")}</h2><p class="modal-task">${esc(task?.title ?? "")}</p><div class="modal-buttons">${button("close-modal", t("cancel"))}${button("confirm-delete", t("delete"), undefined, "danger-solid")}</div>`;
  if (modal.kind === "abandon")
    content = `<h2>${t("abandon")}</h2><p>${t("abandonNote")}</p><div class="modal-buttons">${button("close-modal", t("cancel"))}${button("confirm-abandon", t("apply"), undefined, "primary")}</div>`;
  if (modal.kind === "postpone") {
    const tomorrow = addDays(snapshot.date, 1),
      next = addDays(snapshot.date, 7);
    content = `<h2>${t("postpone")}</h2><p class="modal-task">${esc(task?.title ?? "")}</p>
<div class="postpone-shortcuts"><button data-date="${tomorrow}">${t("tomorrow")}</button><button data-date="${next}">${t("nextWeek")}</button></div>
<div class="date-entry"><input aria-label="${t("otherDate")}" id="postpone-date" type="date" min="${tomorrow}" value="${modal.date ?? ""}"/>${button("postpone-custom", t("move"))}</div>
<div class="modal-buttons">${button("close-modal", t("cancel"))}</div>`;
  }
  if (modal.kind === "time") {
    content = `<h2>${t("chooseTime")}</h2>${timePicker(task)}<div class="modal-buttons">${button("clear-reminder", t("clearReminder"), undefined, "text-danger")}${button("choose-time", t("save"), undefined, "primary")}</div>`;
  }
  return `<section class="modal ${modal.kind}-modal" role="dialog" aria-label="${t(modal.kind === "time" ? "chooseTime" : modal.kind === "postpone" ? "postpone" : modal.kind === "delete" ? "delete" : "abandon")}">${content}${modalError ? `<p class="modal-error" role="alert">${esc(modalError)}</p>` : ""}</section>`;
}
function timePicker(task?: Task) {
  const bounds = reminderBounds(),
    now = new Date();
  if (!timeDraft) {
    const due = new Date(
      Math.min(
        new Date(bounds.max).getTime(),
        Math.max(
          new Date(bounds.min).getTime(),
          ((reminderDraft[modal!.id!] || task?.reminderAt) ??
            Math.ceil((Date.now() + 600000) / 60000) * 60) * 1000,
        ),
      ),
    );
    timeDraft = {
      day: dateKey(due),
      hour: due.getHours(),
      minute: due.getMinutes(),
    };
  }
  const days = [dateKey(now), addDays(dateKey(now), 1)];
  const opts = reminderOptions(timeDraft.day, timeDraft.hour, now);
  const wheel = (key: "hour" | "minute", values: number[], label: string) =>
    `<div class="wheel-column"><span>${label}</span><div class="time-wheel" id="wheel-${key}" data-wheel="${key}" role="listbox" tabindex="0" aria-label="${label}">${values.map((v) => `<button role="option" aria-selected="${timeDraft![key] === v}" data-wheel-value="${v}" data-wheel-key="${key}" tabindex="-1">${String(v).padStart(2, "0")}</button>`).join("")}</div></div>`;
  return `<div class="day-options">${days.map((d, i) => `<button data-day="${d}" class="${timeDraft!.day === d ? "primary" : ""}" ${!reminderOptions(d, 0, now).hours.length ? "disabled" : ""}>${i ? t("tomorrow") : snapshot.settings.language === "ko" ? "오늘" : "Today"}</button>`).join("")}</div><div class="time-wheels">${wheel("hour", opts.hours, snapshot.settings.language === "ko" ? "시" : "Hour")}${wheel("minute", opts.minutes, snapshot.settings.language === "ko" ? "분" : "Minute")}</div>`;
}
function wheelValue(key: "hour" | "minute", value: number) {
  if (!timeDraft) return;
  timeDraft[key] = value;
  const options = reminderOptions(timeDraft.day, timeDraft.hour);
  if (!options.hours.includes(timeDraft.hour))
    timeDraft.hour = options.hours[0] ?? 0;
  const minutes = reminderOptions(timeDraft.day, timeDraft.hour).minutes;
  if (!minutes.includes(timeDraft.minute)) timeDraft.minute = minutes[0] ?? 0;
  modalError = "";
  render();
}
function selectedDue() {
  if (!timeDraft) return NaN;
  app.querySelectorAll<HTMLElement>(".time-wheel").forEach((wheel) => {
    const key = wheel.dataset.wheel as "hour" | "minute";
    const options = Array.from(
      wheel.querySelectorAll<HTMLElement>("[data-wheel-value]"),
    );
    const at = Math.max(
      0,
      Math.min(options.length - 1, Math.round(wheel.scrollTop / 30)),
    );
    if (options[at]) timeDraft![key] = Number(options[at].dataset.wheelValue);
  });
  return (
    new Date(
      timeDraft.day +
        "T" +
        String(timeDraft.hour).padStart(2, "0") +
        ":" +
        String(timeDraft.minute).padStart(2, "0"),
    ).getTime() / 1000
  );
}
function openModal(next: Modal) {
  modal = next;
  modalError = "";
  timeDraft = null;
  render();
}

function reminderView() {
  const pending = snapshot.rollover;
  return `<div class="reminder-shell">
${pending ? `<section class="rollover"><p>${t("rolloverBody")}</p><small>${pending.taskIds.length} ${t("count")} · ${pending.sourceDate}</small><div class="modal-buttons">${button("rollover-no", t("no"))}${button("rollover-yes", t("yes"), undefined, "primary")}</div></section>` : ""}
${snapshot.alerts.length ? `<section class="alert-section"><h2>${snapshot.alerts.some((a) => a.overdue) ? t("overdue") : t("reminder")}</h2><div class="alert-list">${snapshot.alerts.map((a) => `<article class="alert-item"><span class="dot"></span><div><p>${esc(a.title)}</p><small>${esc(fmtTime(a.dueAt))}</small></div></article>`).join("")}</div>${button("acknowledge", t("acknowledge"), "check", "primary acknowledge")}</section>` : ""}
${!pending && !snapshot.alerts.length ? `<div class="empty"><h2>${t("empty")}</h2></div>` : ""}${snapshot.error ? `<p class="error">${esc(snapshot.error)}</p>` : ""}</div>`;
}
function captureDraft(form: HTMLFormElement) {
  const data = new FormData(form),
    next = structuredClone(draft!);
  for (const key of [
    "vaultPath",
    "dailyFolder",
    "readHeading",
    "addHeading",
    "iconPath",
  ] as const) {
    const val = data.get(key);
    if (val !== null) next[key] = String(val);
  }
  next.language = String(data.get("language")) as "ko" | "en";
  for (const key of [
    "visibleRows",
    "fontSize",
    "reminderFontSize",
    "displaySeconds",
    "repeatMinutes",
  ] as const)
    next[key] = Number(data.get(key));
  coreConfirmed = data.has("coreConfirm");
  next.sound = data.has("sound");
  next.autostart = data.has("autostart");
  draft = next;
}
function render() {
  if (!snapshot || drag?.started) return;
  rendering = true;
  const scrollTop = app.querySelector(".task-list")?.scrollTop ?? 0;
  const active = document.activeElement as HTMLInputElement | null,
    focusId = active?.id,
    value = active?.tagName === "INPUT" ? active.value : undefined;
  const caret =
    active?.tagName === "INPUT" && ["text", "search"].includes(active.type)
      ? active.selectionStart
      : null;
  document.documentElement.lang = draft?.language ?? snapshot.settings.language;
  document.documentElement.style.setProperty(
    "--font",
    snapshot.fontFamily + ", Segoe UI, sans-serif",
  );
  document.documentElement.style.setProperty("--accent", snapshot.accent);
  document.documentElement.style.setProperty(
    "--task-font",
    snapshot.settings.fontSize + "px",
  );
  document.documentElement.style.setProperty(
    "--reminder-font",
    snapshot.settings.reminderFontSize + "px",
  );
  const systemDark = matchMedia("(prefers-color-scheme: dark)").matches;
  document.documentElement.dataset.theme =
    snapshot.theme === "system"
      ? systemDark
        ? "dark"
        : "light"
      : snapshot.theme;
  app.className = isPopup
    ? "popup-app"
    : isReminder
      ? "notification-app"
      : view === "settings"
        ? "settings-app"
        : "list-app";
  document.body.classList.toggle("preview-mode", preview);

  app.innerHTML =
    (isPopup
      ? modalView()
      : isReminder
        ? reminderView()
        : view === "settings"
          ? settingsView()
          : listView()) +
    (preview && !isReminder && !isPopup ? modalView() : "") +
    (toast ? `<div class="toast" role="alert">${esc(toast)}</div>` : "");
  const list = app.querySelector(".task-list");
  if (list) list.scrollTop = scrollTop;
  app.toggleAttribute("data-busy", busy);
  app.querySelectorAll<HTMLElement>(".time-wheel").forEach((wheel) => {
    const selected = wheel.querySelector<HTMLElement>('[aria-selected="true"]');
    if (selected)
      wheel.scrollTop = Array.from(wheel.children).indexOf(selected) * 30;
  });
  if (isPopup && modal) {
    const height = app.querySelector<HTMLElement>(".modal")!.scrollHeight + 2;
    if (height !== popupHeight) {
      popupHeight = height;
      void invoke("resize_popup", { height }).catch((e) => console.error(e));
    }
  }
  if (focusId) {
    const input = document.getElementById(focusId) as HTMLInputElement | null;
    if (input) {
      if (value !== undefined) input.value = value;
      input.focus();
      if (caret !== null) input.setSelectionRange(caret, caret);
    }
  }
  const rename = document.getElementById(
    "rename-input",
  ) as HTMLInputElement | null;
  if (editing && !focusId) {
    rename?.focus();
    rename?.select();
  }
  rename?.addEventListener("keydown", async (e) => {
    if (e.key === "Enter") {
      e.preventDefault();
      await finishInline();
    }
    if (e.key === "Escape") {
      e.stopPropagation();
      editing = null;
      render();
    }
  });
  if (adding && !focusId) {
    document.getElementById("add-input")?.focus();
    document.querySelector(".new-task")?.scrollIntoView({ block: "nearest" });
  }
  if (!preview && !isReminder && !isPopup) {
    if (modal && modal !== openedModal) {
      openedModal = modal;
      popupToken++;
      void invoke("open_popup", {
        context: { token: popupToken, modal, snapshot, view, reminderDraft },
      }).catch((e) => {
        modal = null;
        openedModal = null;
        notify(String(e));
      });
    } else if (!modal && openedModal) {
      openedModal = null;
      void invoke("close_popup");
    }

    const list = app.querySelector<HTMLElement>(".task-list");
    const children = Array.from(list?.children ?? []) as HTMLElement[];
    const rows =
      children.length <= (view === "tasks" ? 6 : 5)
        ? Math.max(snapshot.settings.visibleRows, children.length)
        : snapshot.settings.visibleRows;
    const contents = children
      .slice(0, rows)
      .reduce((sum, row) => sum + row.getBoundingClientRect().height, 0);
    const footer = app.querySelector<HTMLElement>("footer");
    const height =
      view === "settings"
        ? 570
        : Math.ceil(
            contents +
              (footer?.getBoundingClientRect().height ?? 36) +
              18 +
              (app.querySelector<HTMLElement>(".error")?.getBoundingClientRect()
                .height ?? 0),
          );
    void invoke("resize_main", { height, settings: view === "settings" }).catch(
      (e) => console.error(e),
    );
  }
  rendering = false;
}
app.addEventListener("submit", async (e) => {
  e.preventDefault();
  const form = e.target as HTMLFormElement;
  if (form.id === "add-form") {
    await finishInline();
  }
  if (form.id === "settings-form") {
    captureDraft(form);
    if (await dispatch({ kind: "configure", settings: draft! })) {
      setView("tasks");
    }
  }
});
app.addEventListener("input", (e) => {
  if ((e.target as HTMLInputElement).id === "add-input")
    addText = (e.target as HTMLInputElement).value;
  const form = (e.target as HTMLElement).closest<HTMLFormElement>(
    "#settings-form",
  );
  if (form && draft) captureDraft(form);
});
app.addEventListener("change", async (e) => {
  if (inlineCommit) await inlineCommit;
  const input = e.target as HTMLInputElement;
  const settingsForm = input.closest<HTMLFormElement>("#settings-form");
  if (settingsForm && draft) captureDraft(settingsForm);
  if (input.matches("[data-check]"))
    await dispatch({
      kind: "complete",
      id: input.closest<HTMLElement>("[data-id]")!.dataset.id!,
      done: input.checked,
    });
  if (input.id === "postpone-date") {
    modal!.date = input.value;
    modalError = "";
  }
  if (input.name === "language") {
    captureDraft(input.closest("form")!);
    render();
  }
});
app.addEventListener("contextmenu", (e) => {
  const name = (e.target as HTMLElement).closest("[data-task-action=edit]");
  if (name) {
    e.preventDefault();
    editing = name.closest<HTMLElement>("[data-id]")!.dataset.id!;
    render();
  }
});
app.addEventListener("dblclick", (e) => {
  const name = (e.target as HTMLElement).closest("[data-task-action=edit]");
  if (name) {
    editing = name.closest<HTMLElement>("[data-id]")!.dataset.id!;
    render();
  }
});
app.addEventListener("keydown", (e) => {
  if (e.key === "F2") {
    const row = (e.target as HTMLElement).closest<HTMLElement>("[data-id]");
    if (row) {
      e.preventDefault();
      editing = row.dataset.id!;
      render();
    }
  }
});

function cancelDrag() {
  if (!drag) return;
  drag.ghost?.remove();
  if (drag.frame) cancelAnimationFrame(drag.frame);
  drag = null;
  if (pendingSnapshot) {
    snapshot = pendingSnapshot;
    pendingSnapshot = null;
  }
  render();
}
app.addEventListener("pointerdown", (e) => {
  if (modal && !isPopup && !(e.target as HTMLElement).closest(".modal")) {
    e.preventDefault();
    e.stopImmediatePropagation();
    modal = null;
    render();
    return;
  }
  const handle = (e.target as HTMLElement).closest<HTMLElement>("[data-drag]");
  if (!handle || view === "reminders" || busy || e.button !== 0) return;
  e.preventDefault();
  app.setPointerCapture(e.pointerId);
  drag = {
    id: handle.dataset.drag!,
    pointer: e.pointerId,
    x: e.clientX,
    y: e.clientY,
    lastY: e.clientY,
    offsetY:
      e.clientY -
      handle.closest<HTMLElement>(".task")!.getBoundingClientRect().top,
    started: false,
    ids: snapshot.tasks.map((t) => t.id),
    handle,
  };
});
function updateDrag() {
  if (!drag?.started) return;
  const list = app.querySelector<HTMLElement>(".task-list")!;
  const rect = list.getBoundingClientRect();
  if (drag.lastY < rect.top + 20) list.scrollTop -= 6;
  if (drag.lastY > rect.bottom - 20) list.scrollTop += 6;
  const row = document
    .elementFromPoint(
      Math.min(rect.right - 10, Math.max(rect.left + 30, drag.x)),
      Math.min(rect.bottom - 2, Math.max(rect.top + 2, drag.lastY)),
    )
    ?.closest<HTMLElement>(".task[data-id]");
  if (row && row.dataset.id !== drag.id) {
    const box = row.getBoundingClientRect();
    const current = drag.ids.map((id) =>
      snapshot.tasks.find((t) => t.id === id)!,
    );
    drag.ids = moveTask(
      current,
      drag.id,
      row.dataset.id!,
      drag.lastY > box.top + box.height / 2,
    );
    const source = app.querySelector<HTMLElement>(`[data-id="${drag.id}"]`)!;
    const after = drag.ids.indexOf(drag.id) > drag.ids.indexOf(row.dataset.id!);
    if (after) row.after(source);
    else row.before(source);
  }
  drag.frame = requestAnimationFrame(updateDrag);
}
app.addEventListener("pointermove", (e) => {
  if (!drag || e.pointerId !== drag.pointer) return;
  drag.lastY = e.clientY;
  if (!drag.started && Math.hypot(e.clientX - drag.x, e.clientY - drag.y) > 4) {
    drag.started = true;
    const row = drag.handle.closest<HTMLElement>(".task")!;
    const box = row.getBoundingClientRect();
    drag.ghost = row.cloneNode(true) as HTMLElement;
    drag.ghost.removeAttribute("data-id");
    drag.ghost.className = "task drag-ghost";
    drag.ghost.style.width = box.width + "px";
    drag.ghost.style.left = box.left + "px";
    document.body.append(drag.ghost);
    row.classList.add("drag-placeholder");
    updateDrag();
  }
  if (drag.ghost) drag.ghost.style.top = e.clientY - drag.offsetY + "px";
});
app.addEventListener("pointerup", async (e) => {
  if (!drag || drag.pointer !== e.pointerId) return;
  const current = drag;
  const list = app.querySelector(".task-list")!.getBoundingClientRect();
  const inside =
    e.clientX >= list.left &&
    e.clientX <= list.right &&
    e.clientY >= list.top &&
    e.clientY <= list.bottom;
  suppressClick = current.started;
  if (current.started && inside) {
    reordering = true;
    const latest = pendingSnapshot ?? snapshot;
    const tasks = new Map(latest.tasks.map((t) => [t.id, t]));
    // Keep the provisional order through the save; rollback only if persistence fails.
    if (
      current.ids.length === tasks.size &&
      current.ids.every((id) => tasks.has(id))
    ) {
      latest.tasks = current.ids.map((id) => tasks.get(id)!);
      snapshot = latest;
      pendingSnapshot = null;
    }
  }
  cancelDrag();
  setTimeout(() => {
    suppressClick = false;
  }, 0);
  if (current.started && inside) {
    const ok = await dispatch({ kind: "reorder", ids: current.ids });
    reordering = false;
    pendingSnapshot = null;
    if (!ok) await refresh();
    else render();
  }
});
app.addEventListener("pointercancel", cancelDrag);
app.addEventListener("lostpointercapture", () => {
  if (drag) cancelDrag();
});
app.addEventListener("click", async (e) => {
  const clicked = e.target as HTMLElement;
  const target = (e.target as HTMLElement).closest<HTMLButtonElement>("button");
  const row = clicked.closest<HTMLElement>(".task[data-id]");
  if (pendingInline) {
    pendingInline = false;
    await finishInline();
  }
  if (inlineCommit) await inlineCommit;
  if (busy || suppressClick) return;
  if (!target) {
    if (view === "reminders" && row)
      openModal({ kind: "time", id: row.dataset.id });
    return;
  }
  if (isPopup) {
    await popupChoice(target);
    return;
  }
  const taskAction = target.dataset.taskAction,
    id = target.closest<HTMLElement>("[data-id]")?.dataset.id;
  if (taskAction && id) {
    if (taskAction === "edit") return;
    if (taskAction === "delete" || taskAction === "postpone") {
      openModal({ kind: taskAction, id });
      return;
    }
    if (taskAction === "time") {
      openModal({ kind: "time", id });
      return;
    }
  }
  if (target.dataset.day && timeDraft) {
    timeDraft.day = target.dataset.day;
    wheelValue("hour", timeDraft.hour);
    return;
  }
  if (target.dataset.wheelKey) {
    wheelValue(
      target.dataset.wheelKey as "hour" | "minute",
      Number(target.dataset.wheelValue),
    );
    return;
  }
  if (target.dataset.date && modal?.id) {
    await finishModalChoice({
      token: popupToken,
      kind: "postpone",
      date: target.dataset.date,
    });
    return;
  }
  switch (target.dataset.action) {
    case "add":
      adding = true;
      addText = "";
      render();
      break;
    case "taskbar-settings":
      if (!preview) await invoke("open_taskbar_settings");
      break;
    case "settings":
      setView("settings");
      break;
    case "back":
      setView("tasks");
      break;
    case "completed":
      setView("completed");
      break;
    case "reminders":
      setView("reminders");
      break;
    case "hide":
      if (!preview) await getCurrentWindow().hide();
      break;
    case "refresh":
      await refresh();
      break;
    case "daily":
      if (!preview) {
        try {
          await invoke("open_daily");
          await getCurrentWindow().hide();
        } catch (e) {
          notify(String(e));
        }
      }
      break;
    case "data":
      if (!preview) {
        try {
          await invoke("open_app_data");
        } catch (e) {
          notify(String(e));
        }
      }
      break;
    case "close-modal":
      modal = null;
      render();
      break;
    case "confirm-delete":
      if (modal?.id && (await dispatch({ kind: "delete", id: modal.id }))) {
        modal = null;
        render();
      }
      break;
    case "apply-reminders":
      if (await dispatch({ kind: "reminders", selections: reminderDraft }))
        setView("tasks");
      break;
    case "postpone-custom":
      await finishModalChoice({
        token: popupToken,
        kind: "postpone",
        date: (document.getElementById("postpone-date") as HTMLInputElement)
          .value,
      });
      break;
    case "choose-time":
      await finishModalChoice({
        token: popupToken,
        kind: "choose-time",
        due: selectedDue(),
      });
      break;
    case "clear-reminder":
      await finishModalChoice({ token: popupToken, kind: "clear-reminder" });
      break;
    case "acknowledge":
      await dispatch({
        kind: "acknowledge",
        ids: snapshot.alerts.map((a) => a.id),
      });
      break;
    case "rollover-yes":
      await dispatch({ kind: "rollover", accept: true });
      break;
    case "rollover-no":
      await dispatch({ kind: "rollover", accept: false });
      break;
    case "recover":
      await dispatch({ kind: "recover" });
      break;
    case "abandon":
      openModal({ kind: "abandon" });
      break;
    case "confirm-abandon":
      if (await dispatch({ kind: "abandonRecovery" })) {
        modal = null;
        render();
      }
      break;
    case "browse-vault": {
      captureDraft(document.getElementById("settings-form") as HTMLFormElement);
      const path = preview
        ? "C:\\Example Vault"
        : await open({ directory: true, multiple: false, title: t("vault") });
      if (typeof path === "string") {
        draft!.vaultPath = path;
        try {
          draft!.dailyFolder = preview
            ? "Daily"
            : await invoke("detect_daily_folder", { path });
        } catch (e) {
          notify(String(e));
        }
        render();
      }
      break;
    }
    case "browse-icon": {
      captureDraft(document.getElementById("settings-form") as HTMLFormElement);
      const path = preview
        ? "icon.ico"
        : await open({
            multiple: false,
            filters: [{ name: "Icons", extensions: ["ico", "png"] }],
          });
      if (typeof path === "string") {
        draft!.iconPath = path;
        render();
      }
      break;
    }
    case "reset-icon":
      captureDraft(document.getElementById("settings-form") as HTMLFormElement);
      draft!.iconPath = "";
      render();
      break;
  }
});
document.addEventListener("keydown", async (e) => {
  if (e.key === "Escape") {
    if (drag) {
      cancelDrag();
      return;
    }
    if (isPopup) {
      if (!preview) await invoke("close_popup");
      return;
    }
    if (adding) {
      adding = false;
      addText = "";
      render();
      return;
    }
    if (modal) {
      modal = null;
      render();
    } else if (view !== "tasks" && snapshot.settings.vaultPath)
      setView("tasks");
    else if (!preview) await getCurrentWindow().hide();
  }
});
async function initialSnapshot(): Promise<Snapshot> {
  for (let attempt = 0; attempt < 20; attempt++) {
    try {
      return await invoke<Snapshot>("get_snapshot");
    } catch (e) {
      if (!String(e).includes("state not managed") || attempt === 19) throw e;
      await new Promise((resolve) => setTimeout(resolve, 100));
    }
  }
  throw new Error("App initialization timed out");
}
async function start() {
  snapshot = preview ? previewSnapshot() : await initialSnapshot();
  if (!snapshot.settings.vaultPath) {
    view = "settings";
    draft = structuredClone(snapshot.settings);
  }
  if (preview) {
    const mode = new URLSearchParams(location.search).get("mode");
    if (mode === "setup") {
      snapshot.settings.vaultPath = "";
      setView("settings");
    }
    if (mode === "reminder")
      snapshot.alerts = [
        {
          id: "a",
          title: snapshot.tasks[0].title,
          dueAt: Date.now() / 1000 - 120,
          overdue: true,
        },
      ];
    if (mode === "rollover")
      snapshot.rollover = { sourceDate: "2026-10-10", taskIds: ["a", "b"] };
  }
  if (isPopup) {
    await listen<PopupContext>("popup-context", (event) =>
      applyPopup(event.payload),
    );
    const context = await invoke<PopupContext | null>("get_popup");
    if (context) applyPopup(context);
  } else render();
  matchMedia("(prefers-color-scheme: dark)").addEventListener("change", () =>
    render(),
  );
  if (!preview) {
    await listen("popup-closed", () => {
      modal = null;
      openedModal = null;
      render();
    });
    if (!isPopup)
      await listen<any>("popup-choice", (event) =>
        receivePopupChoice(event.payload),
      );
    await listen<Snapshot>("snapshot", (event) => {
      if (drag || reordering) {
        pendingSnapshot = event.payload;
        return;
      }
      snapshot = event.payload;
      render();
    });
    await listen("show-settings", () => setView("settings"));
    await listen("reset-view", () => {
      if (snapshot.settings.vaultPath) setView("tasks");
    });
    if (isPopup)
      await listen<{ token: number; error: string }>(
        "popup-result",
        (event) => {
          if (event.payload.token === popupToken) {
            busy = false;
            modalError = event.payload.error;
            render();
          }
        },
      );
    await listen<string>("app-error", (event) => notify(event.payload));
    if (!isReminder && !isPopup)
      await getCurrentWindow().onFocusChanged(({ payload: focused }) => {
        if (!focused && !modal && !drag) void finishInline();
        if (
          !focused &&
          view !== "settings" &&
          !modal &&
          !editing &&
          !adding &&
          !drag
        )
          void getCurrentWindow().hide();
      });
  }
}

function applyPopup(context: PopupContext) {
  snapshot = context.snapshot;
  modal = context.modal;
  popupToken = context.token;
  view = context.view;
  reminderDraft = context.reminderDraft;
  modalError = "";
  timeDraft = null;
  busy = false;
  render();
}
type PopupChoice = { token: number; kind: string; date?: string; due?: number };
async function popupChoice(target: HTMLButtonElement) {
  if (!modal || busy) return;
  if (target.dataset.day && timeDraft) {
    timeDraft.day = target.dataset.day;
    wheelValue("hour", timeDraft.hour);
    return;
  }
  if (target.dataset.wheelKey) {
    wheelValue(
      target.dataset.wheelKey as "hour" | "minute",
      Number(target.dataset.wheelValue),
    );
    return;
  }
  if (target.dataset.action === "close-modal") {
    await invoke("close_popup");
    return;
  }
  const choice: PopupChoice = {
    token: popupToken,
    kind: target.dataset.action ?? "postpone",
    date: target.dataset.date,
  };
  if (choice.kind === "postpone-custom") {
    choice.kind = "postpone";
    choice.date = (
      document.getElementById("postpone-date") as HTMLInputElement
    ).value;
  }
  if (choice.kind === "choose-time") choice.due = selectedDue();
  if (!validateChoice(choice)) return;
  busy = true;
  render();
  try {
    await invoke("choose_popup", { choice });
  } catch (e) {
    busy = false;
    modalError = String(e);
    render();
  }
}
function validateChoice(choice: PopupChoice) {
  let error = "";
  if (
    choice.kind === "postpone" &&
    !futureDate(choice.date ?? "", dateKey(new Date()))
  )
    error =
      snapshot.settings.language === "ko"
        ? "오늘 이후의 날짜를 선택해주세요."
        : "Choose a date after today.";
  if (choice.kind === "choose-time" && !validReminder(choice.due ?? NaN))
    error = t("timeHint");
  if (error) {
    modalError = error;
    render();
    return false;
  }
  return true;
}
async function finishModalChoice(choice: PopupChoice) {
  if (!modal || choice.token !== popupToken || busy || !validateChoice(choice))
    return false;
  const id = modal.id;
  let action: Action | undefined;
  switch (choice.kind) {
    case "confirm-delete":
      if (id) action = { kind: "delete", id };
      break;
    case "confirm-abandon":
      action = { kind: "abandonRecovery" };
      break;
    case "postpone":
      if (id && choice.date)
        action = { kind: "postpone", id, date: choice.date };
      break;
    case "choose-time":
      if (id && choice.due) {
        if (view === "reminders") reminderDraft[id] = choice.due;
        else action = { kind: "reminders", selections: { [id]: choice.due } };
      }
      break;
    case "clear-reminder":
      if (id) {
        if (view === "reminders") reminderDraft[id] = 0;
        else action = { kind: "clearReminder", id };
      }
      break;
    default:
      return false;
  }
  modalError = "";
  if (
    action &&
    !(await dispatch(action, (message) => {
      modalError = message;
    }))
  )
    return false;
  modal = null;
  timeDraft = null;
  render();
  return true;
}
async function receivePopupChoice(choice: PopupChoice) {
  if (!modal || choice.token !== popupToken || busy) return;
  const ok = await finishModalChoice(choice);
  if (!ok && !preview)
    await emitTo("popup", "popup-result", {
      token: choice.token,
      error: modalError || t("error"),
    });
}
async function finishInline(): Promise<boolean> {
  if (inlineCommit) return inlineCommit;
  const input = document.getElementById(
    editing ? "rename-input" : adding ? "add-input" : "",
  ) as HTMLInputElement | null;
  if (!input) return true;
  const id = editing,
    title = input.value.trim();
  editing = null;
  adding = false;
  addText = "";
  if (!title) {
    render();
    return true;
  }
  if (id && snapshot.tasks.find((t) => t.id === id)?.title === title) {
    render();
    return true;
  }
  inlineCommit = dispatch(
    id ? { kind: "rename", id, title } : { kind: "add", title },
  );
  try {
    return await inlineCommit;
  } finally {
    inlineCommit = null;
  }
}
document.addEventListener(
  "pointerdown",
  (e) => {
    pointerPressed = true;
    const active = document.activeElement;
    const target = e.target as HTMLElement;
    if (
      active instanceof HTMLInputElement &&
      active !== target &&
      !target.closest("#add-form")
    )
      active.blur();
  },
  true,
);
document.addEventListener(
  "pointerup",
  () => {
    pointerPressed = false;
    setTimeout(() => {
      if (pendingInline) {
        pendingInline = false;
        void finishInline();
      }
    }, 0);
  },
  true,
);
app.addEventListener("focusout", (e) => {
  if (rendering) return;
  const input = e.target as HTMLInputElement;
  if (input.id === "add-input" || input.id === "rename-input") {
    if (pointerPressed) pendingInline = true;
    else void finishInline();
  }
});
app.addEventListener(
  "scroll",
  (e) => {
    const wheel = e.target as HTMLElement;
    const key = wheel.dataset.wheel as "hour" | "minute" | undefined;
    if (!key || !timeDraft || rendering) return;
    clearTimeout(wheelTimers.get(key));
    // Wait for snap settling so a fast wheel gesture selects its final value.
    wheelTimers.set(
      key,
      setTimeout(() => {
        if (!wheel.isConnected || !timeDraft) return;
        const options = Array.from(
          wheel.querySelectorAll<HTMLElement>("[data-wheel-value]"),
        );
        const index = Math.max(
          0,
          Math.min(options.length - 1, Math.round(wheel.scrollTop / 30)),
        );
        const value = Number(options[index]?.dataset.wheelValue);
        if (Number.isFinite(value) && timeDraft[key] !== value)
          wheelValue(key, value);
      }, 120),
    );
  },
  true,
);
app.addEventListener("keydown", (e) => {
  const wheel = (e.target as HTMLElement).closest<HTMLElement>("[data-wheel]");
  if (
    !wheel ||
    !timeDraft ||
    !["ArrowUp", "ArrowDown", "Home", "End"].includes(e.key)
  )
    return;
  e.preventDefault();
  const key = wheel.dataset.wheel as "hour" | "minute";
  const values = Array.from(
    wheel.querySelectorAll<HTMLElement>("[data-wheel-value]"),
  ).map((v) => Number(v.dataset.wheelValue));
  const at = values.indexOf(timeDraft[key]);
  const next =
    e.key === "Home"
      ? 0
      : e.key === "End"
        ? values.length - 1
        : Math.max(
            0,
            Math.min(values.length - 1, at + (e.key === "ArrowDown" ? 1 : -1)),
          );
  wheelValue(key, values[next]);
});
void start().catch((error) => {
  app.innerHTML = `<div class="startup-error"><h1>Tray Tasks</h1><p>${esc(String(error))}</p><button data-action="refresh">Retry</button></div>`;
});
