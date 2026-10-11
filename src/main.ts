import "./style.css";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { open } from "@tauri-apps/plugin-dialog";
import type { Action, Settings, Snapshot, Task } from "./types";
import {
  addDays,
  escapeHtml as esc,
  localInput,
  reminderBounds,
  moveTaskGroup,
} from "./utils";
import { previewAction, previewSnapshot } from "./preview";
const preview = new URLSearchParams(location.search).has("preview");
const isReminder =
  new URLSearchParams(location.search).get("window") === "reminder";
const app = document.querySelector<HTMLDivElement>("#app")!;
let snapshot: Snapshot;
let view: "tasks" | "completed" | "reminders" | "settings" = "tasks";
let draft: Settings | null = null,
  reminderDraft: Record<string, number> = {},
  editing: string | null = null,
  dragged: string | null = null,
  busy = false;
let modal: {
  kind: "delete" | "postpone" | "time" | "abandon";
  id?: string;
  date?: string;
} | null = null;
let coreConfirmed = false;
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
    deleteNote: "하위 항목도 함께 삭제되며 데일리 노트에 반영됩니다.",
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
    version: "개발 버전 0.1.0",
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
    version: "Development build 0.1.0",
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
async function dispatch(action: Action) {
  if (busy) return false;
  busy = true;
  render();
  try {
    snapshot = preview
      ? previewAction(action)
      : await invoke<Snapshot>("action", { action });
    return true;
  } catch (e) {
    notify(String(e));
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
  view = next;
  modal = null;
  editing = null;
  reminderDraft = {};
  draft = next === "settings" ? structuredClone(snapshot.settings) : null;
  render();
}
function header(title: string, subtitle: string) {
  return `<header><div class="brand" data-tauri-drag-region><div class="brand-icon">${svg("check")}</div><div><h1>${esc(title)}</h1><p>${esc(subtitle)}</p></div></div><div class="header-actions">${button("settings", t("settings"), "gear", "icon-button")}${button("hide", t("cancel"), "close", "icon-button")}</div></header>`;
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
  return `<div class="task ${task.done ? "done" : ""} ${view === "reminders" ? "selecting" : ""} ${pending ? "selected" : ""}" data-id="${esc(task.id)}" style="--depth:${Math.min(task.depth, 5)}">
<button class="grip icon-button" draggable="${view !== "reminders"}" data-drag="${esc(task.id)}" aria-label="Reorder ${esc(task.title)}">${svg("grip")}</button>
${view === "reminders" ? `<button class="selection-dot ${pending ? "active" : ""}" data-task-action="time" aria-label="${t("chooseTime")}">${pending ? svg("check") : ""}</button>` : `<input type="checkbox" data-check aria-label="${esc(task.title)}" ${task.done ? "checked" : ""}/>`}
<div class="task-content">${editing === task.id ? `<input id="rename-input" class="rename-input" value="${esc(task.title)}" aria-label="${t("rename")}"/>` : `<button class="task-name" data-task-action="${view === "reminders" ? "time" : "edit"}" title="${t("editHint")}">${esc(task.title)}</button>`}
${effective ? `<button class="task-time" data-task-action="time">${svg("clock")}${esc(fmtTime(effective!))}</button>` : ""}</div>
${view !== "reminders" ? `<div class="task-actions"><button class="icon-button danger" data-task-action="delete" title="${t("delete")}" aria-label="${t("delete")} ${esc(task.title)}">${svg("trash")}</button>${!task.done ? `<button class="icon-button" data-task-action="postpone" title="${t("postpone")}" aria-label="${t("postpone")} ${esc(task.title)}">${svg("next")}</button>` : ""}</div>` : ""}</div>`;
}
function listView() {
  const done = view === "completed",
    tasks = snapshot.tasks.filter((task) => task.done === done);
  return `${header(view === "reminders" ? t("reminders") : done ? t("completed") : t("today"), fmtDate(snapshot.date))}${errorBanner()}
<div class="list-meta"><span class="dot"></span><span>${view === "reminders" ? t("reminderHint") : `${tasks.length} ${t("count")} · ${esc(snapshot.settings.dailyFolder || "/")}`}</span><button class="icon-button" data-action="refresh" title="${t("refresh")}">${svg("refresh")}</button></div>
<main class="task-list" style="--rows:${snapshot.settings.visibleRows}">${tasks.length ? tasks.map(taskRow).join("") : `<div class="empty"><div class="empty-icon">${svg(done ? "check" : "plus")}</div><h2>${done ? t("completedEmpty") : t("empty")}</h2><p>${t("emptySub")}</p></div>`}</main>
${!done && view !== "reminders" ? `<form id="add-form" class="add-form"><span>${svg("plus")}</span><input id="add-input" name="title" autocomplete="off" placeholder="${t("add")}" aria-label="${t("add")}"/><button type="submit" class="add-submit" aria-label="${t("save")}">↵</button></form>` : ""}
<footer>${view === "reminders" ? `${button("back", t("cancel"), "back")}${button("apply-reminders", t("apply"), "check", "primary")}` : done ? button("back", t("back"), "back") + button("daily", t("daily"), "calendar") + button("reminders", t("reminder"), "clock") : `${button("completed", t("completed"), "check")}${button("daily", t("daily"), "calendar")}${button("reminders", t("reminder"), "clock")}`}</footer>
<div class="status"><span>Obsidian daily notes</span><span>${snapshot.tasks.filter((t) => !t.done).length} to do</span></div>`;
}
function field(key: keyof Settings, label: Label, hint?: Label, type = "text") {
  return `<label class="field"><span>${t(label)}</span><input id="setting-${key}" name="${key}" type="${type}" value="${esc(String(draft![key]))}"/>${hint ? `<small>${t(hint)}</small>` : ""}</label>`;
}
function settingsView() {
  const first = !snapshot.settings.vaultPath;
  draft ??= structuredClone(snapshot.settings);
  return `${header(first ? "Tray Tasks" : t("settings"), first ? t("welcome") : t("version"))}<form id="settings-form" class="settings-form">
${first ? `<div class="welcome"><h2>${t("welcome")}</h2><p>${t("welcomeSub")}</p></div>` : ""}
<label class="field"><span>${t("language")}</span><select id="setting-language" name="language"><option value="ko" ${draft.language === "ko" ? "selected" : ""}>한국어</option><option value="en" ${draft.language === "en" ? "selected" : ""}>English</option></select></label>
<label class="field"><span>${t("vault")}</span><div class="path-input"><input id="setting-vaultPath" name="vaultPath" value="${esc(draft.vaultPath)}" required placeholder="C:\\Vaults\\My Vault"/>${button("browse-vault", t("browse"), "folder")}</div></label>
${field("dailyFolder", "folder", "folderHint")}<div class="setup-note">${svg("calendar")}<p>${t("coreHint")}</p></div>
${first ? `<label class="toggle core-confirm"><input name="coreConfirm" type="checkbox" ${coreConfirmed ? "checked" : ""} required/><span>${t("coreConfirm")}</span></label>` : ""}
${field("readHeading", "readHeading", "readHint")}${field("addHeading", "addHeading", "addHint")}
<div class="field-grid">${field("visibleRows", "rows", undefined, "number")}${field("fontSize", "font", undefined, "number")}${field("reminderFontSize", "reminderFont", undefined, "number")}${field("displaySeconds", "display", undefined, "number")}${field("repeatMinutes", "repeat", undefined, "number")}</div>
<label class="toggle"><input name="sound" type="checkbox" ${draft.sound ? "checked" : ""}/><span>${t("sound")}</span></label>
<label class="toggle"><input name="autostart" type="checkbox" ${draft.autostart ? "checked" : ""}/><span>${t("autostart")}</span></label>
<div class="field"><span>${t("icon")}</span><small>${esc(draft.iconPath || t("resetIcon"))}</small><div class="inline-buttons">${button("browse-icon", t("browse"), "folder")}${button("reset-icon", t("resetIcon"))}</div></div>
${!first ? button("data", t("data"), "folder", "data-button") : ""}<p class="settings-footnote">${t("localOnly")}</p>
<div class="settings-buttons">${!first ? button("back", t("cancel")) : ""}<button type="submit" class="primary">${first ? t("connect") : t("save")}</button></div></form>`;
}
function modalView() {
  if (!modal) return "";
  const task = snapshot.tasks.find((t) => t.id === modal!.id);
  let content = "";
  if (modal.kind === "delete")
    content = `<h2>${t("deleteTitle")}</h2><p class="modal-task">${esc(task?.title ?? "")}</p><p>${t("deleteNote")}</p><div class="modal-buttons">${button("close-modal", t("cancel"))}${button("confirm-delete", t("delete"), undefined, "danger-solid")}</div>`;
  if (modal.kind === "abandon")
    content = `<h2>${t("abandon")}</h2><p>${t("abandonNote")}</p><div class="modal-buttons">${button("close-modal", t("cancel"))}${button("confirm-abandon", t("apply"), undefined, "primary")}</div>`;
  if (modal.kind === "postpone") {
    const tomorrow = addDays(snapshot.date, 1),
      next = addDays(snapshot.date, 7);
    content = `<h2>${t("postpone")}</h2><p class="modal-task">${esc(task?.title ?? "")}</p>
<button class="date-option" data-date="${tomorrow}"><span>${t("tomorrow")}</span><small>${esc(fmtDate(tomorrow))}</small></button><button class="date-option" data-date="${next}"><span>${t("nextWeek")}</span><small>${esc(fmtDate(next))}</small></button>
<label class="field"><span>${t("otherDate")}</span><input id="postpone-date" type="date" min="${tomorrow}" value="${modal.date ?? ""}"/></label>
${modal.date ? `<button class="date-option chosen" data-date="${modal.date}"><span>${t("move")}</span><small>${esc(fmtDate(modal.date))}</small></button>` : ""}<div class="modal-buttons">${button("close-modal", t("cancel"))}</div>`;
  }
  if (modal.kind === "time") {
    const bounds = reminderBounds(),
      due =
        (reminderDraft[modal.id!] || task?.reminderAt) ??
        Math.ceil((Date.now() + 600000) / 60000) * 60;
    content = `<h2>${t("chooseTime")}</h2><p class="modal-task">${esc(task?.title ?? "")}</p><label class="field"><input id="reminder-time" type="datetime-local" min="${bounds.min}" max="${bounds.max}" value="${localInput(new Date(due * 1000))}" required/></label><p>${t("timeHint")}</p><div class="modal-buttons">${button("close-modal", t("cancel"))}${button("choose-time", t("save"), undefined, "primary")}</div>${task?.reminderAt ? button("clear-reminder", t("clearReminder"), undefined, "text-danger") : ""}`;
  }
  return `<div class="modal-overlay"><section class="modal" role="dialog" aria-modal="true">${content}</section></div>`;
}
function reminderView() {
  const pending = snapshot.rollover;
  return `<div class="reminder-shell"><div class="notification-brand"><div class="brand-icon">${svg("clock")}</div><span>Tray Tasks</span></div>
${pending ? `<section class="rollover"><h2>${t("rolloverTitle")}</h2><p>${t("rolloverBody")}</p><small>${pending.taskIds.length} ${t("count")} · ${pending.sourceDate}</small><p class="hint">${t("rolloverNote")}</p><div class="modal-buttons">${button("rollover-no", t("no"))}${button("rollover-yes", t("yes"), undefined, "primary")}</div></section>` : ""}
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
  if (!snapshot) return;
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
  app.className = isReminder ? "notification-app" : "";
  app.innerHTML =
    (isReminder
      ? reminderView()
      : view === "settings"
        ? settingsView()
        : listView()) +
    (!isReminder ? modalView() : "") +
    (toast ? `<div class="toast" role="alert">${esc(toast)}</div>` : "");
  app.toggleAttribute("data-busy", busy);
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
      if (
        await dispatch({ kind: "rename", id: editing!, title: rename.value })
      ) {
        editing = null;
        render();
      }
    }
    if (e.key === "Escape") {
      e.stopPropagation();
      editing = null;
      render();
    }
  });
  if (!preview && !isReminder) {
    const height =
      view === "settings"
        ? 740
        : Math.min(
            680,
            190 +
              snapshot.settings.visibleRows * 56 +
              (snapshot.error ? 110 : 0),
          );
    void invoke("resize_main", { height }).catch((e) => console.error(e));
  }
}
app.addEventListener("submit", async (e) => {
  e.preventDefault();
  const form = e.target as HTMLFormElement;
  if (form.id === "add-form") {
    const input = form.elements.namedItem("title") as HTMLInputElement;
    if (await dispatch({ kind: "add", title: input.value })) {
      const next = document.getElementById("add-input") as HTMLInputElement;
      next.value = "";
      next.focus();
    }
  }
  if (form.id === "settings-form") {
    captureDraft(form);
    if (await dispatch({ kind: "configure", settings: draft! })) {
      setView("tasks");
      notify(t("ready"));
    }
  }
});
app.addEventListener("input", (e) => {
  const form = (e.target as HTMLElement).closest<HTMLFormElement>(
    "#settings-form",
  );
  if (form && draft) captureDraft(form);
});
app.addEventListener("change", async (e) => {
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
    render();
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
app.addEventListener("dragstart", (e) => {
  const handle = (e.target as HTMLElement).closest<HTMLElement>("[data-drag]");
  if (!handle) {
    e.preventDefault();
    return;
  }
  dragged = handle.dataset.drag!;
  e.dataTransfer?.setData("text/plain", dragged);
  if (e.dataTransfer) e.dataTransfer.effectAllowed = "move";
});
app.addEventListener("dragover", (e) => {
  if ((e.target as HTMLElement).closest("[data-id]")) e.preventDefault();
});
app.addEventListener("drop", async (e) => {
  e.preventDefault();
  const row = (e.target as HTMLElement).closest<HTMLElement>("[data-id]");
  if (row && dragged)
    await dispatch({
      kind: "reorder",
      ids: moveTaskGroup(snapshot.tasks, dragged, row.dataset.id!),
    });
  dragged = null;
});
app.addEventListener("click", async (e) => {
  const target = (e.target as HTMLElement).closest<HTMLButtonElement>("button");
  if (!target || busy) return;
  const taskAction = target.dataset.taskAction,
    id = target.closest<HTMLElement>("[data-id]")?.dataset.id;
  if (taskAction && id) {
    if (taskAction === "edit") return;
    if (taskAction === "delete" || taskAction === "postpone") {
      modal = { kind: taskAction, id };
      render();
      return;
    }
    if (taskAction === "time") {
      modal = { kind: "time", id };
      render();
      return;
    }
  }
  if (target.dataset.date && modal?.id) {
    if (
      await dispatch({
        kind: "postpone",
        id: modal.id,
        date: target.dataset.date,
      })
    ) {
      modal = null;
      render();
    }
    return;
  }
  switch (target.dataset.action) {
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
    case "choose-time": {
      const input = document.getElementById(
        "reminder-time",
      ) as HTMLInputElement;
      if (!input.reportValidity()) break;
      const due = Math.floor(new Date(input.value).getTime() / 1000);
      if (!Number.isFinite(due) || due < Date.now() / 1000) {
        notify(t("timeHint"));
        break;
      }
      const taskId = modal!.id!;
      if (view === "reminders") {
        reminderDraft[taskId] = due;
        modal = null;
        render();
      } else if (
        await dispatch({ kind: "reminders", selections: { [taskId]: due } })
      ) {
        modal = null;
        render();
      }
      break;
    }
    case "clear-reminder":
      if (modal?.id) {
        if (view === "reminders") {
          reminderDraft[modal.id] = 0;
          modal = null;
          render();
        } else if (await dispatch({ kind: "clearReminder", id: modal.id })) {
          modal = null;
          render();
        }
      }
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
      modal = { kind: "abandon" };
      render();
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
  render();
  if (!preview) {
    await listen<Snapshot>("snapshot", (event) => {
      snapshot = event.payload;
      render();
    });
    await listen("show-settings", () => setView("settings"));
    await listen<string>("app-error", (event) => notify(event.payload));
    if (!isReminder)
      await getCurrentWindow().onFocusChanged(({ payload: focused }) => {
        if (!focused && view !== "settings" && !modal && !editing)
          void getCurrentWindow().hide();
      });
  }
}
void start().catch((error) => {
  app.innerHTML = `<div class="startup-error"><h1>Tray Tasks</h1><p>${esc(String(error))}</p><button data-action="refresh">Retry</button></div>`;
});
