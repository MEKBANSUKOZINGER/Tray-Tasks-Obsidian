# Architecture

## Desktop and storage

Tauri owns the tray icon, hidden list window, nonmodal action popup, reminder window, Windows login startup and Obsidian URI opening. TypeScript renders local UI. Rust owns all filesystem edits and reminder state.

The vault remains the task source. App-local state stores preferences, display order, reminder timestamps and pending moves. The daily filename uses the local calendar date. Folder paths must remain inside the selected vault, including existing symlink targets.

The app watches the vault and also refreshes once per second. It reads Obsidian appearance preferences for font, accent color and light/dark mode, using Obsidian default base colors. Date rollover and reminders keep running while the list is hidden.

## Task identity and edits

The parser recognizes standard unchecked/completed Markdown checkboxes and the app's postponed marker. Fenced code and frontmatter are excluded. Hidden comments identify tasks independently of title and line position. Missing and duplicate IDs are repaired.

Each checkbox is independent. Indentation does not create app subtasks. Delete/move updates only the selected line and preserves other checkboxes and continuation text. Display order is stored separately; pointer capture on the stable list container lets rows be reordered during dragging without losing input capture.

Each note edit reads the latest contents, saves a backup, writes a same-folder temporary file and checks that the source has not changed before replacement. External editors do not participate in a shared lock; a short race between the final comparison and replacement remains possible.

## Moves

A durable journal records source-before/source-after, destination, copied block and new IDs before changing either note.

Recovery first checks the source. If it contains a later external edit, recovery stops. Otherwise it copies the destination block only if its new root ID is absent, finishes the source update and updates reminder ownership. Repeating recovery does not copy the block twice.

Manual postponement leaves a postponed source and cancels the selected task reminder. Midnight rollover completes the source and transfers reminders with unchanged due timestamps. Indented neighboring items are handled independently. Recovery can still finish older journals that contain entire subtrees.

Clearing a recovery record never deletes destination content. Inspect both notes before resolving a manually changed move.

## Reminder scheduling

Draft edits live in the UI until Apply. A zero timestamp in a selection represents a pending removal. The backend validates the complete batch before changing any reminder.

Valid new times run from now through tomorrow at 23:59. Persisted reminders may outlive that range. Repeats stay anchored to the original due time. Restarting with overdue reminders schedules an immediate missed-reminder alert.

Acknowledgement clears only the displayed reminders. Completion/deletion cancels the relevant reminders. Removing a task or marking it completed directly in the vault also cancels its reminder when refreshed.

Midnight captures all eligible unfinished checkboxes and waits for a decision. Accept moves them to the currently active date. Reject leaves them in place and cancels only reminders that have never rung.

## Development isolation

Debug builds accept TRAY_TASKS_DATA_DIR to isolate app data. These isolated debug runs skip autostart registration to avoid changing an installed app preference. Release builds ignore this override. Browser preview mode uses mock state and never invokes vault operations.
