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
    pub fn push_entry(&mut self, description: &str) {
        self.entries.push(Entry {
            description: description.trim().to_string(),
            date: Local::today().naive_local(),
        });
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
            let db = Self::default();
            db.write()?;
            return Ok(db);
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
struct Entry {
    description: String,
    date: NaiveDate,
}
