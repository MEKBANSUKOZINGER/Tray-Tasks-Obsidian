# Architecture

## Desktop and storage

Tauri owns the persistent widget, tray icon, hidden list window, separate settings window, nonmodal action popup, reminder window, Windows login startup and Obsidian URI opening. TypeScript renders local UI. Rust owns all filesystem edits and reminder state.

The vault remains the task source. App-local state stores preferences, display order, reminder timestamps and pending moves. The daily filename uses the local calendar date. Folder paths must remain inside the selected vault, including existing symlink targets.

Windows filesystem validation keeps canonical extended paths internally. The settings snapshot shows an ordinary drive or UNC path without changing persisted vault identity. The paths module converts supported verbatim prefixes before building an Obsidian open URI and percent-encodes its UTF-8 query value. Unsupported device paths and relative note paths are rejected rather than silently rewritten. The same display conversion is used when opening the app data directory externally.

The app watches the vault and also refreshes once per second. It reads Obsidian appearance preferences for font, accent color and light/dark mode, using Obsidian default base colors. Date rollover and reminders keep running while the list is hidden.

## Widget and windows

Widget position and selected task ID are optional persisted fields compatible with older state. Next cycles unfinished tasks in display order without editing Markdown; completion shares the list action. A 250ms maintenance loop reads the primary Shell_TrayWnd and TrayNotifyWnd bounds and docks the widget 8 logical pixels to the left, centered vertically on the taskbar. There is no widget drag handler. If these optional shell-class hints are unavailable, the saved horizontal position is clamped to the taskbar; a new state reserves 330 logical pixels on the right, matching the user's chosen 1950px left position on a 2560px screen. Old saved Y coordinates are ignored. No shell window is resized or reparented.

On Windows, the widget is a top-level owned window of the primary taskbar (GWLP_HWNDPARENT). Windows keeps owned windows above their owner, preventing the taskbar from briefly overtaking the widget. This does not make it a child window or modify the taskbar. A dedicated message-pump thread observes foreground, show, reorder and location-change WinEvents, ignoring this process. Only a visible overlapping window above the widget triggers SetWindowPos with HWND_TOPMOST, SWP_NOACTIVATE and SWP_ASYNCWINDOWPOS. Asynchronous positioning avoids blocking the event thread during UI shutdown. The guard unhooks on its registration thread and restores the previous owner when dropped. The 250ms loop now maintains geometry and ownership, without periodic stacking correction. Explorer restart and taskbar auto-hide behavior still require acceptance testing. Blur dismissal checks whether the pointer is over the widget, so a widget click can toggle an already visible list instead of reopening it after blur.

Reminder height is measured from content. Its acknowledgement sits beside the header; due times remain in the scheduler but are omitted from alert rows. Long titles travel at 14 logical pixels per second with linear alternating animation; reduced-motion behavior remains available.

Each popup context has a token. The frontend measures intrinsic content on every open; the backend validates the token and shows the popup after sizing. Settings use a separate decorated window and do not auto-hide on blur.

Every native window explicitly receives the bundled app icon at setup, including the settings window shown on the Windows taskbar. The executable and installer use a multi-size ICO generated from the supplied PNG. A custom tray-icon setting still applies to the tray and widget independently of the bundled window branding.

## Task identity and edits

The parser recognizes standard unchecked/completed Markdown checkboxes and the app's postponed marker. Fenced code and frontmatter are excluded. Hidden comments identify tasks independently of title and line position. Missing and duplicate IDs are repaired.

Each checkbox is independent. Indentation does not create app subtasks. Delete/move updates only the selected line and preserves other checkboxes and continuation text. Display order is stored separately; pointer capture on the stable list container lets rows be reordered during dragging without losing input capture. The provisional order remains visible while the backend saves; snapshot events are held during the save, and a failed save reloads authoritative state.

Each note edit reads the latest contents, saves a backup, writes a same-folder temporary file and checks that the source has not changed before replacement. External editors do not participate in a shared lock; a short race between the final comparison and replacement remains possible.

## Moves

A durable journal records source-before/source-after, destination, copied block and new IDs before changing either note.

Recovery first checks the source. If it contains a later external edit, recovery stops. Otherwise it copies the destination block only if its new root ID is absent, finishes the source update and updates reminder ownership. Repeating recovery does not copy the block twice.

Manual postponement leaves a postponed source and cancels the selected task reminder. Midnight rollover completes the source and transfers reminders with unchanged due timestamps. Indented neighboring items are handled independently. Recovery can still finish older journals that contain entire subtrees.

Clearing a recovery record never deletes destination content. Inspect both notes before resolving a manually changed move.

## Reminder scheduling

The compact picker uses Today/Tomorrow buttons and hour/minute wheels. Past options are excluded and timestamps are checked again on save. Popup actions stay open until success, with validation and backend errors returned to the popup. Draft edits live in the UI until Apply. A zero timestamp in a selection represents a pending removal. The backend validates the complete batch before changing any reminder.

Valid new times run from now through tomorrow at 23:59. Persisted reminders may outlive that range. Repeats stay anchored to the original due time. Restarting with overdue reminders schedules an immediate missed-reminder alert.

Acknowledgement clears only the displayed reminders. Completion/deletion cancels the relevant reminders. Removing a task or marking it completed directly in the vault also cancels its reminder when refreshed.

Midnight captures all eligible unfinished checkboxes and waits for a decision. Accept moves them to the currently active date. Reject leaves them in place and cancels only reminders that have never rung.

## Development isolation

Debug builds accept TRAY_TASKS_DATA_DIR to isolate app data. These isolated debug runs skip autostart registration to avoid changing an installed app preference. Release builds ignore this override. Browser preview mode uses mock state and never invokes vault operations.
