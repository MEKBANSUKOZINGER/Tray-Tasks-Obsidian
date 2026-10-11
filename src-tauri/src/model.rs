use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Settings {
    pub vault_path: String,
    pub daily_folder: String,
    pub read_heading: String,
    pub add_heading: String,
    pub language: String,
    pub visible_rows: u32,
    pub font_size: u32,
    pub reminder_font_size: u32,
    pub display_seconds: u32,
    pub repeat_minutes: u32,
    pub sound: bool,
    pub autostart: bool,
    pub icon_path: String,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            vault_path: String::new(),
            daily_folder: String::new(),
            read_heading: String::new(),
            add_heading: String::new(),
            language: "ko".into(),
            visible_rows: 6,
            font_size: 14,
            reminder_font_size: 15,
            display_seconds: 60,
            repeat_minutes: 5,
            sound: true,
            autostart: false,
            icon_path: String::new(),
        }
    }
}
impl Settings {
    pub fn validate(&self) -> Result<(), String> {
        if !["ko", "en"].contains(&self.language.as_str()) {
            return Err("Invalid language".into());
        }
        if !(3..=12).contains(&self.visible_rows)
            || !(11..=24).contains(&self.font_size)
            || !(11..=28).contains(&self.reminder_font_size)
            || !(5..=300).contains(&self.display_seconds)
            || !(1..=60).contains(&self.repeat_minutes)
        {
            return Err("Settings are outside supported ranges".into());
        }
        if self.read_heading.contains(['\r', '\n']) || self.add_heading.contains(['\r', '\n']) {
            return Err("A heading must fit on one line".into());
        }
        Ok(())
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Reminder {
    pub task_id: String,
    pub note_path: String,
    pub title: String,
    pub due_at: i64,
    pub next_at: i64,
    pub has_rung: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Rollover {
    pub source_date: String,
    pub task_ids: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MoveJournal {
    pub source_path: String,
    pub expected_source: String,
    pub source_after: String,
    pub target_path: String,
    pub target_block: String,
    pub original_id: String,
    pub new_id: String,
    pub transfer_reminder: bool,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Persisted {
    pub schema_version: u32,
    pub settings: Settings,
    pub orders: BTreeMap<String, Vec<String>>,
    pub reminders: BTreeMap<String, Reminder>,
    pub rollover: Option<Rollover>,
    pub journal: Option<MoveJournal>,
}
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskView {
    pub id: String,
    pub title: String,
    pub done: bool,
    pub depth: usize,
    pub parent_id: Option<String>,
    pub reminder_at: Option<i64>,
}
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AlertView {
    pub id: String,
    pub title: String,
    pub due_at: i64,
    pub overdue: bool,
}
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
    pub settings: Settings,
    pub date: String,
    pub tasks: Vec<TaskView>,
    pub alerts: Vec<AlertView>,
    pub rollover: Option<Rollover>,
    pub recovery_pending: bool,
    pub error: Option<String>,
    pub font_family: String,
    pub accent: String,
    pub theme: String,
}
