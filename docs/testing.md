# Verification — 1.0.0

1.0.0 designates the 0.5.1 behavior as the first release version. Existing native UI observations below were recorded for that implementation; version metadata, documentation and explicit native window icons were updated for 1.0.0. The old GetWindow-based test traversal failed once during this release check. It now uses EnumWindows, verifies API results and cleans up test windows on failure; 20 consecutive native regression runs passed (2,000 owner raises).

Date: 2026-10-11. Windows x64, Node 24, Rust 1.99 MSVC, Visual Studio 2026 C++ tools and Windows SDK.

## Automated

- Rust: 27 tests passing.
- TypeScript: 12 tests passing.
- TypeScript/Vite production build and Windows NSIS installer generation passing.
- Clippy with all targets and warnings denied passing.
- Formatting checks passing.
- Dependency audit was clean at the previous verification; it was not rerun for this revision.

Tests additionally cover Windows owner/widget stacking through 100 owner raises without a timer. They cover source preservation, UTF-8/CRLF, fence/frontmatter exclusion, IDs, independent moves/deletes and legacy subtree-journal recovery, path constraints, concurrent-change refusal, backups, idempotent move recovery, exact reminder transfer, draft removal batch atomicity, recurrence, failed settings changes and both reorder directions.

## Manual observations

Using a separate sample vault and isolated debug app data:

0.5.1 stacking fix:
- The debug widget logged a native taskbar owner assignment; it remained docked at (1939, 1396).
- Next changed selection with an unchanged Markdown SHA-256. The widget icon opened the list, and a second click closed it. The widget checkbox completed only the selected line.
- Direct taskbar-click visual verification was unavailable through the automation surface. The native owner-order regression passed instead; real taskbar clicking, other topmost apps, Explorer restart, auto-hide and monitor/DPI changes remain acceptance checks.

Fourth revision (0.5.0):
- Primary taskbar bounds were (0, 1392)-(2560, 1440); notification area began at x=2227. The 280x40 widget docked at (1939, 1396), leaving 8px before system icons.
- Transparent background, supplied icon and absence of a move handle were inspected.
- Clicking the widget icon again hid the existing list.
- One long-title native reminder measured 111px tall, with a compact acknowledgement beside the header and no timestamp below the task.
- Acknowledgement cleared its reminder and left the Markdown checkbox unfinished.
- Browser measurements confirmed a linear 397px movement over 28.357s (14px/s), transparent widget CSS and both new Korean heading labels.
- The old 250ms stacking correction was replaced in 0.5.1 after the user observed a flash when clicking the taskbar.

Third revision:
- Native widget appeared without opening the full list; Next changed selection with an unchanged Markdown SHA-256.
- Widget checkbox completed only its selected line.
- Move handle changed physical position and persisted coordinates; restoration was retested after correcting startup events and taskbar-region bounds.
- Deletion popup stayed 106px on reopen; postponement stayed 181px without an unwanted scrollbar.
- Native app-styled calendar, joined date entry, 280px windows and horizontal title scrolling were inspected.
- Separate settings remained visible after another window was focused; Cancel returned to the list.
- Browser DOM checks confirmed overflowing title animations, a 3px title/clock gap, removal of the two settings shortcuts, calendar selection and inline past-date errors.

Second revision:
- Five tasks plus Add fit without a scrollbar; larger lists still scroll.
- Native hour/minute wheel scrolling selected a new minute and saved the exact timestamp after Apply; before Apply, persisted reminders remained empty.
- Closing the decorated settings window and reopening through the shared tray/single-instance show path returned to the compact task list.
- Native dragging persisted the chosen order and left the Markdown hash unchanged.
- Native outside-click creation appended a nonempty checkbox.
- Browser preview verified empty-row cancellation, outside-click add/rename, direct reminder selection, Today/Tomorrow selection, keyboard wheel navigation and inline rejection of past postponement dates.
- Compact, shadow-free native action windows were visually inspected.

Earlier regression observations:

- Native first-run setup appeared as a centered, decorated 640px window.
- Native pointer dragging changed persisted display order; the Markdown file hash stayed unchanged.
- The Add button scrolled with the list and opened an empty input row; Enter appended a new checkbox.
- Action popups appeared above the list with matching width and content-based height. Clicking the list cancelled deletion without changing the note.
- A reminder time choice remained a draft until Apply; only Apply wrote reminder state.
- The production executable rendered its embedded setup UI at http://tauri.localhost/; no development server is required.
- A configured app started without showing the list. Starting a second instance opened the existing list.
- A native checkbox changed the matching Markdown checkbox without altering the journal text.
- A new native task appeared in the list and was appended to the daily note.
- Direct edits to task titles in the sample vault refreshed the list.
- Restarting with two overdue reminders showed both in one missed-reminder window above the visible list, with matching width.
- One acknowledgement cleared both reminders in persisted state.
- List footer remained visible after resizing; notification acknowledgement remained fixed while its list scrolled.

Browser preview additionally covered add, completion, reopening, renaming and cancellation of a draft reminder removal.

Production executable and installer generation are checked separately from the debug UI. Installer generation does not prove that installation or upgrades have been tested.

## Remaining acceptance checks

- Install and uninstall the packaged NSIS application on a clean Windows account.
- Enable login startup, reboot, and confirm no list or Obsidian window flashes.
- Exercise Obsidian URI opening against a real configured vault.
- Check multiple monitors, DPI scales and taskbar positions.
- Run overnight, sleep/resume and repeat-reminder timing in normal use.
- Review deleted/missing custom icons, readonly notes and interrupted moves.
- Confirm required licensing and distribution metadata before public release.

## Commands

```powershell
npm test
npm run desktop:test
npm run format:check
npm run build
npm run desktop:build
```

For Rust linting, load the Visual Studio developer PowerShell, then use:
`cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings`.

Test vaults, debug data, dependencies and build outputs are excluded from Git.
