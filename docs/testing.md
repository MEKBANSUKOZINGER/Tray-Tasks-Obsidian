# Verification — 0.1.0

Date: 2026-10-11. Windows x64, Node 24, Rust 1.99 MSVC, Visual Studio 2026 C++ tools and Windows SDK.

## Automated

- Rust: 19 tests passing.
- TypeScript: 6 tests passing.
- TypeScript/Vite production build passing.
- Clippy with all targets and warnings denied passing.
- Formatting checks passing.
- npm dependency audit: zero reported vulnerabilities at verification time.

Tests cover source preservation, UTF-8/CRLF, fence/frontmatter exclusion, IDs, nested moves and deletes, path constraints, concurrent-change refusal, backups, idempotent move recovery, exact reminder transfer, draft removal batch atomicity, recurrence, failed settings changes and both reorder directions.

## Manual observations

Using a separate sample vault and isolated debug app data:

- Native first-run setup rendered correctly.
- The production executable rendered its embedded setup UI at http://tauri.localhost/; no development server is required.
- A configured app started without showing the list. Starting a second instance opened the existing list.
- A native checkbox changed the matching Markdown checkbox without altering the journal text.
- A new native task appeared in the list and was appended to the daily note.
- Direct edits to task titles in the sample vault refreshed the list.
- Restarting with two overdue reminders showed both in one missed-reminder window.
- One acknowledgement cleared both reminders in persisted state.
- List footer remained visible after resizing; notification acknowledgement remained fixed while its list scrolled.

Browser preview additionally covered add, completion, reopening, renaming and cancellation of a draft reminder removal.

Production executable and installer generation are checked separately from the debug UI. Installer generation does not prove that installation or upgrades have been tested.

## Before 1.0.0

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
