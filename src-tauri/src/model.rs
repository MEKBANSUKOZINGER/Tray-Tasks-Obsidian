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
    pub widget_width: u32,
    pub widget_icon_only: bool,
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
            widget_width: 140,
            widget_icon_only: false,
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
    pub fn widget_size(&self) -> [u32; 2] {
        [
            if self.widget_icon_only {
                40
            } else {
                self.widget_width
            },
            40,
        ]
    }
    pub fn validate(&self) -> Result<(), String> {
        if !["ko", "en"].contains(&self.language.as_str()) {
            return Err("Invalid language".into());
        }
        if !(120..=600).contains(&self.widget_width)
            || !(3..=12).contains(&self.visible_rows)
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
    pub widget_position: Option<[i32; 2]>,
    pub widget_task_id: Option<String>,
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
    pub widget_task_id: Option<String>,
    pub alerts: Vec<AlertView>,
    pub rollover: Option<Rollover>,
    pub recovery_pending: bool,
    pub error: Option<String>,
    pub font_family: String,
    pub accent: String,
    pub theme: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn old_preferences_get_a_half_width_widget_without_resetting_existing_fields() {
        let settings: Settings =
            serde_json::from_str(r#"{"vaultPath":"C:\\Vault","autostart":true,"fontSize":17}"#)
                .unwrap();
        assert_eq!(settings.widget_size(), [140, 40]);
        assert_eq!(settings.vault_path, r"C:\Vault");
        assert!(settings.autostart);
        assert_eq!(settings.font_size, 17);
        assert!(!settings.widget_icon_only);
    }
    #[test]
    fn widget_width_bounds_and_icon_mode_keep_the_saved_width() {
        let mut settings = Settings::default();
        for width in [120, 140, 280, 600] {
            settings.widget_width = width;
            assert!(settings.validate().is_ok());
            assert_eq!(settings.widget_size(), [width, 40]);
            settings.widget_icon_only = true;
            assert_eq!(settings.widget_size(), [40, 40]);
            settings.widget_icon_only = false;
            assert_eq!(settings.widget_size(), [width, 40]);
        }
        for width in [0, 119, 601, u32::MAX] {
            settings.widget_width = width;
            assert!(settings.validate().is_err());
        }
    }
}
