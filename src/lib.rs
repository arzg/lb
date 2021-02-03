use chrono::{Local, NaiveDate};
use etcetera::app_strategy::{AppStrategy, AppStrategyArgs, Xdg};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::path::{Path, PathBuf};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Db {
    entries: Vec<Entry>,
}

impl Db {
    pub fn push_entry(&mut self, entry: Entry) {
        self.entries.push(entry);
    }

    pub fn markdown(&self) -> String {
        self.entries
            .iter()
            .map(|entry| format!("- {}: {}", entry.date, entry.description))
            .intersperse("\n".to_string())
            .collect()
    }

    pub fn read() -> anyhow::Result<Self> {
        let db_path = Self::path()?;

        if !db_path.exists() {
            return Self::initialize();
        }

        let db_file = File::open(db_path)?;
        let db = bincode::deserialize_from(db_file)?;

        Ok(db)
    }

    pub fn write(&self) -> anyhow::Result<()> {
        let db_path = Self::path()?;
        let db_file = safe_create_file(&db_path)?;
        bincode::serialize_into(db_file, &self)?;

        Ok(())
    }

    fn initialize() -> anyhow::Result<Self> {
        let db = Self::default();
        db.write()?;
        Ok(db)
    }

    fn path() -> anyhow::Result<PathBuf> {
        let xdg = Xdg::new(AppStrategyArgs {
            top_level_domain: "io.github".to_string(),
            author: "arzg".to_string(),
            app_name: "journal".to_string(),
        })?;

        Ok(xdg.in_data_dir("db"))
    }
}

fn safe_create_file(path: &Path) -> anyhow::Result<File> {
    fs::create_dir_all(path.parent().unwrap())?;
    Ok(File::create(path)?)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Entry {
    description: String,
    date: NaiveDate,
}

impl From<&str> for Entry {
    fn from(s: &str) -> Self {
        Self {
            description: s.trim().to_string(),
            date: Local::today().naive_local(),
        }
    }
}
