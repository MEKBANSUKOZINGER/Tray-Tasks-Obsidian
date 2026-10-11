# Architecture

## Desktop and storage

Tauri owns the tray icon, hidden list window, reminder window, Windows login startup and Obsidian URI opening. TypeScript renders local UI. Rust owns all filesystem edits and reminder state.

The vault remains the task source. App-local state stores preferences, display order, reminder timestamps and pending moves. The daily filename uses the local calendar date. Folder paths must remain inside the selected vault, including existing symlink targets.

The app watches the vault and also refreshes once per second. It reads Obsidian appearance preferences for font and accent color. Date rollover and reminders keep running while the list is hidden.

## Task identity and edits

The parser recognizes standard unchecked/completed Markdown checkboxes and the app's postponed marker. Fenced code and frontmatter are excluded. Hidden comments identify tasks independently of title and line position. Missing and duplicate IDs are repaired.

Parentage follows indentation. Editing a parent does not automatically complete its children. Deleting or moving a parent includes its subtree and indented continuation text. Display ordering is stored separately and preserves the parent-child hierarchy.

Each note edit reads the latest contents, saves a backup, writes a same-folder temporary file and checks that the source has not changed before replacement. External editors do not participate in a shared lock; a short race between the final comparison and replacement remains possible.

## Moves

A durable journal records source-before/source-after, destination, copied block and new IDs before changing either note.

Recovery first checks the source. If it contains a later external edit, recovery stops. Otherwise it copies the destination block only if its new root ID is absent, finishes the source update and updates reminder ownership. Repeating recovery does not copy the block twice.

Manual postponement leaves a postponed source and cancels reminders for moved descendants. Midnight rollover completes the source and transfers reminders with unchanged due timestamps. Children that were already completed retain their completion state in the copied block.

Clearing a recovery record never deletes destination content. Inspect both notes before resolving a manually changed move.

## Reminder scheduling

Draft edits live in the UI until Apply. A zero timestamp in a selection represents a pending removal. The backend validates the complete batch before changing any reminder.

Valid new times run from now through tomorrow at 23:59. Persisted reminders may outlive that range. Repeats stay anchored to the original due time. Restarting with overdue reminders schedules an immediate missed-reminder alert.

Acknowledgement clears only the displayed reminders. Completion/deletion cancels the relevant reminders. Removing a task or marking it completed directly in the vault also cancels its reminder when refreshed.

Midnight captures eligible root tasks and waits for a decision. Accept moves them to the currently active date. Reject leaves them in place and cancels only reminders that have never rung.

## Development isolation

Debug builds accept TRAY_TASKS_DATA_DIR to isolate app data. Release builds ignore this override. Browser preview mode uses mock state and never invokes vault operations.
