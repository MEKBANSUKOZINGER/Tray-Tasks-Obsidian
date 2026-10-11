# Tray Tasks

Your Obsidian daily tasks, one click away.

A Windows desktop widget and tray app that reads and edits the checkboxes in your daily notes. Obsidian can stay closed.

**0.5.0 — development preview.** [한국어 안내](README.ko.md)

![Persistent task widget on Windows](docs/assets/widget.jpg)

![Tray Tasks running on Windows with a sample vault](docs/assets/tray-tasks.jpg)

## What it does

- Keep one unfinished task in an always-on-top widget. It docks just left of the Windows notification area; cycle tasks with Next and click its icon to toggle the list.
- Add, rename, complete, reopen and delete independent tasks. Indented checkboxes appear as regular tasks.
- Drag tasks with live placement preview; display order persists without rearranging Markdown.
- Postpone to tomorrow, the same weekday next week, or a date you choose.
- Set reminders using Today/Tomorrow and scrolling hour/minute wheels, with grouped alerts and configurable repeats.
- Carry unfinished tasks across midnight while keeping their reminder times.
- Show the widget when signing in to Windows, with the full list and Obsidian kept closed.
- Korean and English UI, custom tray icons, fonts, accent color and compact window settings.

The compact 280px task list uses Obsidian default light/dark colors, with borderless dialogs above the list and a separate centered settings window. Long titles scroll at a constant 14 pixels per second. The transparent widget stays aligned with the primary taskbar and refreshes its topmost position without taking focus. Widget selection persists across restarts. If notification-area detection is unavailable, the previous horizontal position is reused with the taskbar height locked. The Add button scrolls with the list and opens a blank input row. Enter or clicking elsewhere saves a nonempty name; blank rows are discarded. Rename edits also commit on blur. Five tasks plus Add fit without a scrollbar. Invalid postponement dates remain in the compact popup with an inline error.

Manual postponement cancels reminders. Midnight rollover preserves the exact scheduled time.

![Compact reminder with acknowledgement beside the header](docs/assets/reminder.jpg)

## First run

1. Run the Windows installer, then open Tray Tasks.
2. In Obsidian, enable **Core plugins → Daily notes**. Set the date format to **YYYY-MM-DD**.
3. Choose your vault and the matching daily-note folder in Tray Tasks. Leave the folder blank for the vault root.
4. Confirm the Daily Notes setup and save. Use the icon in the notification area to open your task list.

The folder picker reads Obsidian's Daily Notes folder when its configuration exists. You can also enter it manually. Windows may initially put the tray icon in its overflow menu. Drag the tray icon out of the overflow menu if needed. Windows controls this preference.

Existing notes are kept. Missing notes are created as empty UTF-8 files. Note templates are not executed.

## Markdown and local data

Tasks receive an identifier such as:

```md
- [ ] Write the README <!-- tray-tasks:9f53db32-1c7c-4b39-8992-e7748a04d336 -->
```

The comment is present in Markdown source and hidden in Obsidian's reading view. Manual postponement leaves a history line:

```md
- [>] Write the README <!-- tray-tasks:... --> → [[2026-10-12]]
```

Preferences, display order, reminders and backups stay outside your vault in the app's local data folder. The usual Windows location is `%LOCALAPPDATA%\com.traytasks.obsidian`.

Edits use a backup, a temporary file and a check for external changes. Moves use a recovery journal. These checks do not provide a shared lock with external editors or merge sync conflicts. See [architecture](docs/architecture.md).

## Development

Install Node.js 22.12 or newer, Rust stable (MSVC), Visual Studio C++ tools and Windows SDK. Windows WebView2 is also required.

```powershell
npm ci
npm run desktop:dev
```

The desktop wrapper discovers Visual Studio and loads its compiler environment.

```powershell
npm test
npm run desktop:test
npm run desktop:check
npm run build
npm run desktop:build
```

The installer is created under `src-tauri/target/release/bundle/nsis`.

For a browser-only UI preview, run `npm run dev` and open `http://127.0.0.1:1420/?preview`. Preview changes never access a vault.

[Verification and remaining manual checks](docs/testing.md) · [Changelog](CHANGELOG.md)

## Current scope

Windows only. Daily-note filenames must use YYYY-MM-DD. No template execution, cloud sync, conflict merging or dependency on Obsidian Tasks/Reminder plugins. Settings and reminders are specific to this computer. This preview has not yet completed reboot, installer upgrade and multiple-monitor acceptance testing.
