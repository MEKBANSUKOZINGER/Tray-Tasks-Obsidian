use crate::{markdown, model::*, storage};
use chrono::{Days, Local, NaiveDate, TimeZone};
use std::{
    collections::{BTreeMap, HashSet},
    fs,
    io::Write,
    path::{Path, PathBuf},
};
use uuid::Uuid;

pub struct Engine {
    pub data: Persisted,
    pub data_dir: PathBuf,
    pub today: NaiveDate,
    pub alert_until: i64,
    pub displayed_ids: Vec<String>,
    pub error: Option<String>,
}
impl Engine {
    pub fn new(data_dir: PathBuf) -> Result<Self, String> {
        let mut data = storage::load(&data_dir)?;
        data.schema_version = 1;
        let now = Local::now().timestamp();
        // A restarted app must surface missed reminders immediately, including ones that already repeated.
        for r in data.reminders.values_mut() {
            if r.due_at <= now {
                r.next_at = now;
            }
        }
        let mut engine = Self {
            data,
            data_dir,
            today: Local::now().date_naive(),
            alert_until: 0,
            displayed_ids: vec![],
            error: None,
        };
        if engine.data.journal.is_some() {
            if let Err(e) = engine.recover_move() {
                engine.error = Some(e);
            }
        }
        Ok(engine)
    }
    pub fn persist(&self) -> Result<(), String> {
        storage::save(&self.data_dir, &self.data)
    }
    fn root(&self) -> Result<PathBuf, String> {
        let path = PathBuf::from(&self.data.settings.vault_path);
        if self.data.settings.vault_path.is_empty() {
            return Err("Choose an Obsidian vault first.".into());
        }
        path.canonicalize()
            .map_err(|e| format!("Vault is unavailable: {e}"))
    }
    pub fn note_path(&self, date: NaiveDate) -> Result<PathBuf, String> {
        let folder = storage::relative_path(&self.root()?, &self.data.settings.daily_folder)?;
        let rel = folder
            .strip_prefix(self.root()?)
            .map_err(|e| e.to_string())?
            .join(format!("{date}.md"));
        storage::relative_path(&self.root()?, &rel.to_string_lossy())
    }
    fn known_path(&self, relative: &str) -> Result<PathBuf, String> {
        storage::relative_path(&self.root()?, relative)
    }
    fn note_key(&self, path: &Path) -> Result<String, String> {
        Ok(path
            .strip_prefix(self.root()?)
            .map_err(|e| e.to_string())?
            .to_string_lossy()
            .replace('\\', "/"))
    }
    pub fn ensure_note(&self, date: NaiveDate) -> Result<PathBuf, String> {
        let path = self.note_path(date)?;
        fs::create_dir_all(path.parent().ok_or("Invalid note path")?).map_err(|e| e.to_string())?;
        match fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
        {
            Ok(mut file) => {
                file.write_all(b"").map_err(|e| e.to_string())?;
            }
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(e) => return Err(format!("Cannot create daily note: {e}")),
        }
        Ok(path)
    }
    pub fn today_text(&self) -> Result<(PathBuf, String), String> {
        let path = self.ensure_note(self.today)?;
        let old = storage::read(&path)?;
        let updated = markdown::ensure_ids(&old, &self.data.settings.read_heading);
        storage::checked_write(&path, &old, &updated, &self.data_dir)?;
        Ok((path, updated))
    }
    pub fn configure(&mut self, mut settings: Settings) -> Result<(), String> {
        settings.validate()?;
        if settings.vault_path.trim().is_empty() {
            return Err("Choose an Obsidian vault.".into());
        }
        let root = PathBuf::from(settings.vault_path.trim())
            .canonicalize()
            .map_err(|e| format!("Vault is unavailable: {e}"))?;
        if !root.is_dir() {
            return Err("The vault path must be a folder.".into());
        }
        storage::relative_path(&root, &settings.daily_folder)?;
        if !settings.icon_path.is_empty() && !Path::new(&settings.icon_path).is_file() {
            return Err("Custom icon file was not found.".into());
        }
        settings.vault_path = root.to_string_lossy().to_string();
        let switching_vault = self.data.settings.vault_path != settings.vault_path;
        if switching_vault && self.data.journal.is_some() {
            return Err("Resolve the pending move before switching vaults.".into());
        }
        let previous_data = self.data.clone();
        let previous_display = self.displayed_ids.clone();
        if switching_vault {
            self.data.reminders.clear();
            self.data.orders.clear();
            self.data.rollover = None;
            self.displayed_ids.clear();
        }
        self.data.settings = settings;
        if let Err(e) = self.ensure_note(self.today).and_then(|_| self.persist()) {
            self.data = previous_data;
            self.displayed_ids = previous_display;
            return Err(e);
        }
        self.error = None;
        Ok(())
    }
    pub fn detect_folder(path: &str) -> Result<String, String> {
        let root = PathBuf::from(path.trim())
            .canonicalize()
            .map_err(|e| e.to_string())?;
        let settings_path = root.join(".obsidian").join("daily-notes.json");
        if settings_path.exists() {
            let v: serde_json::Value =
                serde_json::from_str(&storage::read(&settings_path)?).map_err(|e| e.to_string())?;
            let format = v
                .get("format")
                .and_then(|v| v.as_str())
                .unwrap_or("YYYY-MM-DD");
            if !format.is_empty() && format != "YYYY-MM-DD" {
                return Err("Set Obsidian Daily Notes date format to YYYY-MM-DD.".into());
            }
            let folder = v.get("folder").and_then(|v| v.as_str()).unwrap_or("");
            storage::relative_path(&root, folder)?;
            return Ok(folder.into());
        }
        Ok(String::new())
    }
    fn edit_today<F>(&mut self, edit: F) -> Result<(), String>
    where
        F: FnOnce(&str) -> Result<String, String>,
    {
        if self.data.journal.is_some() {
            return Err("A move needs recovery before editing more tasks.".into());
        }
        let (path, text) = self.today_text()?;
        let updated = edit(&text)?;
        storage::checked_write(&path, &text, &updated, &self.data_dir)?;
        self.error = None;
        Ok(())
    }
    pub fn add(&mut self, title: &str) -> Result<(), String> {
        let title = title.trim();
        if title.is_empty() || title.contains(['\r', '\n']) {
            return Err("Enter a task name on one line.".into());
        }
        let block = format!("- [ ] {} <!-- tray-tasks:{} -->", title, Uuid::new_v4());
        let heading = self.data.settings.add_heading.clone();
        self.edit_today(|text| Ok(markdown::append(text, &block, &heading)))
    }
    pub fn rename(&mut self, id: &str, title: &str) -> Result<(), String> {
        self.edit_today(|text| markdown::rename(text, id, title))?;
        if let Some(r) = self.data.reminders.get_mut(id) {
            r.title = title.trim().into();
        }
        self.persist()
    }
    pub fn complete(&mut self, id: &str, done: bool) -> Result<(), String> {
        self.edit_today(|text| markdown::set_done(text, id, done))?;
        if done {
            self.data.reminders.remove(id);
            self.displayed_ids.retain(|x| x != id);
        }
        self.persist()
    }
    pub fn delete(&mut self, id: &str) -> Result<(), String> {
        self.edit_today(|text| markdown::remove(text, id))?;
        self.data.reminders.remove(id);
        self.displayed_ids.retain(|active| active != id);
        self.persist()
    }
    pub fn reorder(&mut self, ids: Vec<String>) -> Result<(), String> {
        let (_, text) = self.today_text()?;
        let actual: HashSet<_> = markdown::parse(&text, &self.data.settings.read_heading)
            .into_iter()
            .filter(|t| !t.deferred)
            .map(|t| t.id)
            .collect();
        let supplied: HashSet<_> = ids.iter().cloned().collect();
        if supplied != actual || ids.len() != actual.len() {
            return Err("The task list changed; refresh before reordering.".into());
        }
        self.data.orders.insert(self.today.to_string(), ids);
        self.persist()
    }
    pub fn postpone(&mut self, id: &str, date: &str) -> Result<(), String> {
        let destination =
            NaiveDate::parse_from_str(date, "%Y-%m-%d").map_err(|_| "Invalid date")?;
        if destination <= self.today {
            return Err("Choose a future date.".into());
        }
        self.move_task(self.today, destination, id, false)
    }
    fn move_task(
        &mut self,
        from: NaiveDate,
        to: NaiveDate,
        id: &str,
        rollover: bool,
    ) -> Result<(), String> {
        if self.data.journal.is_some() {
            self.recover_move()?;
        }
        let source_path = self.ensure_note(from)?;
        let old = storage::read(&source_path)?;
        let normalized = markdown::ensure_ids(&old, &self.data.settings.read_heading);
        storage::checked_write(&source_path, &old, &normalized, &self.data_dir)?;
        let source_task = markdown::find(&normalized, id)?;
        if source_task.done || source_task.deferred {
            return Ok(());
        }
        let target_path = self.ensure_note(to)?;
        let (source_after, target_block, new_id, mapping) =
            markdown::moved(&normalized, id, &to.to_string(), rollover)?;
        self.data.journal = Some(MoveJournal {
            source_path: self.note_key(&source_path)?,
            expected_source: normalized,
            source_after,
            target_path: self.note_key(&target_path)?,
            target_block,
            original_id: id.into(),
            new_id,
            transfer_reminder: rollover,
        });
        // Recovery also supports journals created by earlier versions.
        let _ = mapping;
        self.persist()?;
        self.recover_move()
    }
    pub fn recover_move(&mut self) -> Result<(), String> {
        let Some(j) = self.data.journal.clone() else {
            return Ok(());
        };
        let source_path = self.known_path(&j.source_path)?;
        let target_path = self.known_path(&j.target_path)?;
        let current_source = storage::read(&source_path)?;
        if current_source != j.expected_source && current_source != j.source_after {
            return Err("Move paused: the source note changed. The recovery record is preserved; no newer content was overwritten.".into());
        }
        let current_target = storage::read(&target_path)?;
        if !markdown::parse(&current_target, "")
            .iter()
            .any(|t| t.id == j.new_id)
        {
            let target_after = markdown::append(
                &current_target,
                &j.target_block,
                &self.data.settings.add_heading,
            );
            storage::checked_write(&target_path, &current_target, &target_after, &self.data_dir)?;
        }
        storage::checked_write(
            &source_path,
            &current_source,
            &j.source_after,
            &self.data_dir,
        )?;
        let original_task = markdown::find(&j.expected_source, &j.original_id)?;
        let original = markdown::parse(&j.expected_source[original_task.block], "");
        let moved = markdown::parse(&j.target_block, "");
        for (old, new) in original.iter().zip(moved.iter()) {
            if let Some(mut r) = self.data.reminders.remove(&old.id) {
                if j.transfer_reminder && !new.done {
                    r.task_id = new.id.clone();
                    r.note_path = j.target_path.clone();
                    self.data.reminders.insert(new.id.clone(), r);
                }
            }
            for active in &mut self.displayed_ids {
                if *active == old.id {
                    *active = if j.transfer_reminder {
                        new.id.clone()
                    } else {
                        String::new()
                    };
                }
            }
        }
        self.displayed_ids.retain(|id| !id.is_empty());
        self.data.journal = None;
        self.error = None;
        self.persist()
    }
    pub fn abandon_recovery(&mut self) -> Result<(), String> {
        // Do not delete target content: a pending move may already have copied it.
        self.data.journal = None;
        self.error = None;
        self.persist()
    }
    pub fn set_reminders(&mut self, selections: BTreeMap<String, i64>) -> Result<(), String> {
        let now = Local::now().timestamp();
        let tomorrow = self
            .today
            .checked_add_days(Days::new(1))
            .ok_or("Date overflow")?;
        let latest = Local
            .from_local_datetime(&tomorrow.and_hms_opt(23, 59, 59).unwrap())
            .latest()
            .ok_or("Invalid local date")?
            .timestamp();
        let (path, text) = self.today_text()?;
        let key = self.note_key(&path)?;
        let tasks = markdown::parse(&text, &self.data.settings.read_heading);
        let mut prepared = Vec::new();
        for (id, due_at) in selections {
            let task = tasks
                .iter()
                .find(|t| t.id == id && !t.done && !t.deferred)
                .ok_or("The selected task is no longer available.")?;
            if due_at != 0 && (due_at < now || due_at > latest) {
                return Err("Reminder time must be between now and tomorrow 23:59.".into());
            }
            prepared.push(Reminder {
                task_id: id,
                note_path: key.clone(),
                title: task.title.clone(),
                due_at,
                next_at: due_at,
                has_rung: false,
            });
        }
        for r in prepared {
            self.displayed_ids.retain(|id| id != &r.task_id);
            if r.due_at == 0 {
                self.data.reminders.remove(&r.task_id);
            } else {
                self.data.reminders.insert(r.task_id.clone(), r);
            }
        }
        self.persist()
    }
    pub fn clear_reminder(&mut self, id: &str) -> Result<(), String> {
        self.data.reminders.remove(id);
        self.displayed_ids.retain(|x| x != id);
        self.persist()
    }
    pub fn acknowledge(&mut self, ids: Vec<String>) -> Result<(), String> {
        for id in ids {
            self.data.reminders.remove(&id);
            self.displayed_ids.retain(|x| x != &id);
        }
        self.persist()
    }
    pub fn rollover(&mut self, accept: bool) -> Result<(), String> {
        let Some(pending) = self.data.rollover.clone() else {
            return Ok(());
        };
        let from = NaiveDate::parse_from_str(&pending.source_date, "%Y-%m-%d")
            .map_err(|e| e.to_string())?;
        if accept {
            for id in &pending.task_ids {
                let text = storage::read(&self.ensure_note(from)?)?;
                if !markdown::parse(&text, "").iter().any(|t| &t.id == id) {
                    continue;
                }
                self.move_task(from, self.today, id, true)?;
            }
        } else {
            let key = self.note_key(&self.note_path(from)?)?;
            self.data
                .reminders
                .retain(|_, r| r.note_path != key || r.has_rung);
        }
        self.data.rollover = None;
        self.persist()
    }
    pub fn tick_at(&mut self, now: i64, date: NaiveDate) -> Result<bool, String> {
        if self.data.settings.vault_path.is_empty() {
            return Ok(false);
        }
        if date != self.today {
            let previous = self.today;
            if self.data.rollover.is_none() {
                let path = self.note_path(previous)?;
                if path.exists() {
                    let old = storage::read(&path)?;
                    let text = markdown::ensure_ids(&old, &self.data.settings.read_heading);
                    storage::checked_write(&path, &old, &text, &self.data_dir)?;
                    let tasks = markdown::parse(&text, &self.data.settings.read_heading);
                    let candidates: HashSet<_> = tasks
                        .iter()
                        .filter(|t| !t.done && !t.deferred)
                        .map(|t| t.id.clone())
                        .collect();
                    let ids: Vec<_> = tasks
                        .iter()
                        .filter(|t| candidates.contains(&t.id))
                        .map(|t| t.id.clone())
                        .collect();
                    if !ids.is_empty() {
                        self.data.rollover = Some(Rollover {
                            source_date: previous.to_string(),
                            task_ids: ids,
                        });
                    }
                }
            }
            self.today = date;
            self.ensure_note(date)?;
            self.persist()?;
        }
        if now >= self.alert_until {
            self.displayed_ids.clear();
        }
        let mut changed = false;
        let mut notes = BTreeMap::new();
        for r in self.data.reminders.values() {
            if !notes.contains_key(&r.note_path) {
                let path = self.known_path(&r.note_path)?;
                let tasks = if path.exists() {
                    markdown::parse(&storage::read(&path)?, "")
                } else {
                    vec![]
                };
                notes.insert(r.note_path.clone(), tasks);
            }
        }
        let mut cancelled = Vec::new();
        let mut newly_due = Vec::new();
        let interval = i64::from(self.data.settings.repeat_minutes) * 60;
        for (id, r) in &mut self.data.reminders {
            match notes
                .get(&r.note_path)
                .and_then(|tasks| tasks.iter().find(|t| t.id == *id))
            {
                Some(t) if !t.done && !t.deferred => {
                    if r.title != t.title {
                        r.title = t.title.clone();
                        changed = true;
                    }
                }
                _ => {
                    cancelled.push(id.clone());
                    continue;
                }
            }
            if now >= r.next_at {
                newly_due.push(id.clone());
                r.has_rung = true;
                r.next_at = next_repeat(r.due_at, now, interval);
                changed = true;
            }
        }
        for id in cancelled {
            self.data.reminders.remove(&id);
            self.displayed_ids.retain(|x| x != &id);
            changed = true;
        }
        let has_new = !newly_due.is_empty();
        if has_new {
            for id in newly_due {
                if !self.displayed_ids.contains(&id) {
                    self.displayed_ids.push(id);
                }
            }
            self.alert_until = now + i64::from(self.data.settings.display_seconds);
        }
        if changed {
            self.persist()?;
        }
        Ok(has_new)
    }
    pub fn snapshot(&self) -> Snapshot {
        let mut error = self.error.clone();
        let tasks = if self.data.settings.vault_path.is_empty() {
            vec![]
        } else {
            match self.today_text() {
                Ok((_, text)) => {
                    let raw: Vec<_> = markdown::parse(&text, &self.data.settings.read_heading)
                        .into_iter()
                        .filter(|t| !t.deferred)
                        .collect();
                    let order = self
                        .data
                        .orders
                        .get(&self.today.to_string())
                        .cloned()
                        .unwrap_or_default();
                    let rank =
                        |id: &String| order.iter().position(|x| x == id).unwrap_or(usize::MAX);
                    let mut tasks: Vec<_> = raw
                        .iter()
                        .map(|t| TaskView {
                            id: t.id.clone(),
                            title: t.title.clone(),
                            done: t.done,
                            depth: 0,
                            parent_id: None,
                            reminder_at: self.data.reminders.get(&t.id).map(|r| r.due_at),
                        })
                        .collect();
                    tasks.sort_by_key(|t| rank(&t.id));
                    tasks
                }
                Err(e) => {
                    error = Some(e);
                    vec![]
                }
            }
        };
        let now = Local::now().timestamp();
        let alerts = self
            .displayed_ids
            .iter()
            .filter_map(|id| {
                self.data.reminders.get(id).map(|r| AlertView {
                    id: id.clone(),
                    title: r.title.clone(),
                    due_at: r.due_at,
                    overdue: now - r.due_at > 60,
                })
            })
            .collect();
        let (font_family, accent, theme) = self.appearance();
        Snapshot {
            recovery_pending: self.data.journal.is_some(),
            settings: self.data.settings.clone(),
            date: self.today.to_string(),
            tasks,
            alerts,
            rollover: self.data.rollover.clone(),
            error,
            font_family,
            accent,
            theme,
        }
    }
    fn appearance(&self) -> (String, String, String) {
        let default = ("Segoe UI".into(), "#8a5cf5".into(), "dark".into());
        let Ok(root) = self.root() else {
            return default;
        };
        let Ok(text) = fs::read_to_string(root.join(".obsidian").join("appearance.json")) else {
            return default;
        };
        let Ok(v) = serde_json::from_str::<serde_json::Value>(&text) else {
            return default;
        };
        let font = v
            .get("textFontFamily")
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .unwrap_or("Segoe UI")
            .to_string();
        let accent = v
            .get("accentColor")
            .and_then(|v| v.as_str())
            .filter(|s| {
                s.starts_with('#')
                    && [4, 7, 9].contains(&s.len())
                    && s[1..].chars().all(|c| c.is_ascii_hexdigit())
            })
            .unwrap_or("#8a5cf5")
            .to_string();
        let theme = match v.get("theme").and_then(|v| v.as_str()) {
            Some("obsidian") => "dark",
            Some("moonstone") => "light",
            None => "dark",
            _ => "system",
        };
        (font, accent, theme.into())
    }
}
pub fn next_repeat(due: i64, now: i64, interval: i64) -> i64 {
    due + ((now - due).max(0) / interval + 1) * interval
}
#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> (tempfile::TempDir, Engine) {
        let dir = tempfile::tempdir().unwrap();
        let vault = dir.path().join("vault");
        fs::create_dir(&vault).unwrap();
        let mut e = Engine::new(dir.path().join("app")).unwrap();
        let s = Settings {
            vault_path: vault.to_string_lossy().into(),
            ..Settings::default()
        };
        e.configure(s).unwrap();
        (dir, e)
    }
    #[test]
    fn postponing_one_item_preserves_indented_task_and_its_reminder() {
        let (_dir, mut e) = fixture();
        e.add("parent").unwrap();
        let path = e.note_path(e.today).unwrap();
        let text = storage::read(&path).unwrap();
        fs::write(
            &path,
            format!("{text}  - [ ] child <!-- tray-tasks:child -->\n"),
        )
        .unwrap();
        let snap = e.snapshot();
        let id = snap.tasks[0].id.clone();
        let due = Local::now().timestamp() + 60;
        e.set_reminders(BTreeMap::from([(id.clone(), due), ("child".into(), due)]))
            .unwrap();
        let tomorrow = e.today.checked_add_days(Days::new(1)).unwrap();
        e.postpone(&id, &tomorrow.to_string()).unwrap();
        assert_eq!(e.data.reminders.len(), 1);
        assert_eq!(e.data.reminders["child"].due_at, due);
        assert_eq!(e.snapshot().tasks.len(), 1);
        assert!(storage::read(&path).unwrap().contains("- [>] parent"));
        let copied = markdown::parse(&storage::read(&e.note_path(tomorrow).unwrap()).unwrap(), "");
        assert_eq!(copied.len(), 1);
        assert!(!copied[0].done);
        assert_ne!(copied[0].id, id);
    }
    #[test]
    fn rollover_transfers_exact_due_time_and_completes_source() {
        let (_dir, mut e) = fixture();
        e.add("work").unwrap();
        let id = e.snapshot().tasks[0].id.clone();
        let due = Local::now().timestamp() + 3600;
        e.set_reminders(BTreeMap::from([(id.clone(), due)]))
            .unwrap();
        let yesterday = e.today;
        let tomorrow = yesterday.checked_add_days(Days::new(1)).unwrap();
        e.tick_at(Local::now().timestamp(), tomorrow).unwrap();
        assert!(e.data.rollover.is_some());
        e.rollover(true).unwrap();
        let new = e.snapshot().tasks[0].id.clone();
        assert_ne!(id, new);
        assert_eq!(e.data.reminders[&new].due_at, due);
        assert!(
            markdown::find(
                &storage::read(&e.note_path(yesterday).unwrap()).unwrap(),
                &id
            )
            .unwrap()
            .done
        );
    }
    #[test]
    fn cancelling_rollover_removes_only_unfired_reminders() {
        let (_dir, mut e) = fixture();
        e.add("a").unwrap();
        e.add("b").unwrap();
        let tasks = e.snapshot().tasks;
        let due = Local::now().timestamp() + 60;
        e.set_reminders(tasks.iter().map(|t| (t.id.clone(), due)).collect())
            .unwrap();
        e.data.reminders.get_mut(&tasks[0].id).unwrap().has_rung = true;
        let tomorrow = e.today.checked_add_days(Days::new(1)).unwrap();
        e.tick_at(Local::now().timestamp(), tomorrow).unwrap();
        e.rollover(false).unwrap();
        assert!(e.data.reminders.contains_key(&tasks[0].id));
        assert!(!e.data.reminders.contains_key(&tasks[1].id));
    }
    #[test]
    fn out_of_range_reminder_batch_is_atomic() {
        let (_dir, mut e) = fixture();
        e.add("a").unwrap();
        e.add("b").unwrap();
        let t = e.snapshot().tasks;
        let now = Local::now().timestamp();
        assert!(e
            .set_reminders(BTreeMap::from([
                (t[0].id.clone(), now + 60),
                (t[1].id.clone(), now + 7 * 86400)
            ]))
            .is_err());
        assert!(e.data.reminders.is_empty());
    }
    #[test]
    fn recovery_does_not_duplicate_already_copied_target() {
        let (_dir, mut e) = fixture();
        e.add("a").unwrap();
        let id = e.snapshot().tasks[0].id.clone();
        let path = e.note_path(e.today).unwrap();
        let old = storage::read(&path).unwrap();
        let tomorrow = e.today.checked_add_days(Days::new(1)).unwrap();
        let target = e.ensure_note(tomorrow).unwrap();
        let (after, block, new_id, _) =
            markdown::moved(&old, &id, &tomorrow.to_string(), false).unwrap();
        e.data.journal = Some(MoveJournal {
            source_path: e.note_key(&path).unwrap(),
            expected_source: old,
            source_after: after,
            target_path: e.note_key(&target).unwrap(),
            target_block: block.clone(),
            original_id: id,
            new_id: new_id.clone(),
            transfer_reminder: false,
        });
        fs::write(&target, markdown::append("", &block, "")).unwrap();
        e.persist().unwrap();
        let mut resumed = Engine::new(e.data_dir.clone()).unwrap();
        resumed.recover_move().unwrap();
        assert_eq!(
            markdown::parse(&storage::read(&target).unwrap(), "")
                .iter()
                .filter(|t| t.id == new_id)
                .count(),
            1
        );
        assert!(resumed.data.journal.is_none());
    }
    #[test]
    fn recovery_refuses_to_overwrite_later_source_edits() {
        let (_dir, mut e) = fixture();
        e.add("a").unwrap();
        let id = e.snapshot().tasks[0].id.clone();
        let path = e.note_path(e.today).unwrap();
        let old = storage::read(&path).unwrap();
        let target = e
            .ensure_note(e.today.checked_add_days(Days::new(1)).unwrap())
            .unwrap();
        let (after, block, new_id, _) = markdown::moved(&old, &id, "tomorrow", false).unwrap();
        e.data.journal = Some(MoveJournal {
            source_path: e.note_key(&path).unwrap(),
            expected_source: old,
            source_after: after,
            target_path: e.note_key(&target).unwrap(),
            target_block: block,
            original_id: id,
            new_id,
            transfer_reminder: false,
        });
        fs::write(&path, "new external content").unwrap();
        assert!(e.recover_move().is_err());
        assert_eq!(storage::read(&path).unwrap(), "new external content");
        assert!(e.data.journal.is_some());
    }
    #[test]
    fn reminder_removal_batch_is_validated_before_any_mutation() {
        let (_dir, mut e) = fixture();
        e.add("a").unwrap();
        e.add("b").unwrap();
        let t = e.snapshot().tasks;
        let now = Local::now().timestamp();
        e.set_reminders(BTreeMap::from([(t[0].id.clone(), now + 60)]))
            .unwrap();
        assert!(e
            .set_reminders(BTreeMap::from([
                (t[0].id.clone(), 0),
                (t[1].id.clone(), now + 7 * 86400)
            ]))
            .is_err());
        assert!(e.data.reminders.contains_key(&t[0].id));
        e.set_reminders(BTreeMap::from([(t[0].id.clone(), 0)]))
            .unwrap();
        assert!(e.data.reminders.is_empty());
    }
    #[test]
    fn rollover_moves_all_unfinished_checkboxes_independently() {
        let (_dir, mut e) = fixture();
        let path = e.note_path(e.today).unwrap();
        fs::write(&path,"- [ ] root <!-- tray-tasks:root -->\n  - [x] middle <!-- tray-tasks:middle -->\n    - [ ] leaf <!-- tray-tasks:leaf -->\n").unwrap();
        let tomorrow = e.today.checked_add_days(Days::new(1)).unwrap();
        e.tick_at(Local::now().timestamp(), tomorrow).unwrap();
        assert_eq!(
            e.data.rollover.as_ref().unwrap().task_ids,
            vec!["root", "leaf"]
        );
        e.rollover(true).unwrap();
        assert_eq!(e.snapshot().tasks.len(), 2);
    }
    #[test]
    fn failed_configuration_preserves_preferences_and_reminders() {
        let (_dir, mut e) = fixture();
        e.add("a").unwrap();
        let id = e.snapshot().tasks[0].id.clone();
        let now = Local::now().timestamp();
        e.set_reminders(BTreeMap::from([(id.clone(), now + 60)]))
            .unwrap();
        let old = e.data.settings.daily_folder.clone();
        fs::write(e.root().unwrap().join("blocked"), "not a directory").unwrap();
        let mut settings = e.data.settings.clone();
        settings.daily_folder = "blocked".into();
        assert!(e.configure(settings).is_err());
        assert_eq!(e.data.settings.daily_folder, old);
        assert!(e.data.reminders.contains_key(&id));
    }
    #[test]
    fn deleting_one_item_preserves_neighbor_reminders_and_flat_order() {
        let (_dir, mut e) = fixture();
        let path = e.note_path(e.today).unwrap();
        fs::write(
            &path,
            "- [ ] a <!-- tray-tasks:a -->\n  - [ ] b <!-- tray-tasks:b -->\n",
        )
        .unwrap();
        let due = Local::now().timestamp() + 60;
        e.set_reminders(BTreeMap::from([("a".into(), due), ("b".into(), due)]))
            .unwrap();
        e.reorder(vec!["b".into(), "a".into()]).unwrap();
        let tasks = e.snapshot().tasks;
        assert_eq!(tasks[0].id, "b");
        assert!(tasks.iter().all(|t| t.depth == 0 && t.parent_id.is_none()));
        e.delete("a").unwrap();
        assert_eq!(e.snapshot().tasks[0].id, "b");
        assert_eq!(e.data.reminders["b"].due_at, due);
        assert_eq!(
            storage::read(&path).unwrap(),
            "  - [ ] b <!-- tray-tasks:b -->\n"
        );
    }
    #[test]
    fn earlier_subtree_journal_can_finish_after_upgrade() {
        let (_dir, mut e) = fixture();
        let source = e.note_path(e.today).unwrap();
        let old = "- [ ] a <!-- tray-tasks:a -->\n  - [ ] b <!-- tray-tasks:b -->\n";
        fs::write(&source, old).unwrap();
        let due = Local::now().timestamp() + 60;
        e.set_reminders(BTreeMap::from([("a".into(), due), ("b".into(), due)]))
            .unwrap();
        let target = e
            .ensure_note(e.today.checked_add_days(Days::new(1)).unwrap())
            .unwrap();
        e.data.journal = Some(MoveJournal {
            source_path: e.note_key(&source).unwrap(),
            expected_source: old.into(),
            source_after: old.replace("[ ]", "[x]"),
            target_path: e.note_key(&target).unwrap(),
            target_block: old
                .replace("tray-tasks:a", "tray-tasks:new-a")
                .replace("tray-tasks:b", "tray-tasks:new-b"),
            original_id: "a".into(),
            new_id: "new-a".into(),
            transfer_reminder: true,
        });
        e.persist().unwrap();
        let resumed = Engine::new(e.data_dir.clone()).unwrap();
        assert!(resumed.data.journal.is_none());
        assert_eq!(resumed.data.reminders["new-a"].due_at, due);
        assert_eq!(resumed.data.reminders["new-b"].due_at, due);
        assert_eq!(
            markdown::parse(&storage::read(&target).unwrap(), "").len(),
            2
        );
    }
    #[test]
    fn repeats_are_anchored_to_original_due_time() {
        assert_eq!(next_repeat(1000, 1060, 300), 1300);
        assert_eq!(next_repeat(1000, 1901, 300), 2200);
    }
}
