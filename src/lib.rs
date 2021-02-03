use chrono::{Local, NaiveDate};
use etcetera::app_strategy::{AppStrategy, AppStrategyArgs, Xdg};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::str::FromStr;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Db<'a> {
    #[serde(borrow)]
    entries: Vec<Entry<'a>>,
}

impl<'a> Db<'a> {
    pub fn push_entry(&mut self, entry: Entry<'a>) {
        self.entries.push(entry);
    }

    pub fn markdown(&self) -> String {
        self.entries
            .iter()
            .map(|entry| format!("- {}: {}", entry.date, entry.description))
            .intersperse("\n".to_string())
            .collect()
    }

    pub fn read(read_buf: &'a mut ReadBuf) -> anyhow::Result<Self> {
        let db_path = Self::path()?;

        if !db_path.exists() {
            return Self::initialize();
        }

        read_buf.file_contents = Some(fs::read(db_path)?);

        let db = bincode::deserialize(read_buf.file_contents.as_ref().unwrap())?;

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

#[derive(Default)]
pub struct ReadBuf {
    file_contents: Option<Vec<u8>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Entry<'a> {
    description: &'a str,
    date: NaiveDate,
}

impl<'a> From<&'a str> for Entry<'a> {
    fn from(s: &'a str) -> Self {
        if let Some(first_line_ending) = s.find('\n') {
            let (first_line, rest) = s.split_at(first_line_ending);

            let date_on_first_line = NaiveDate::from_str(first_line);
            if let Ok(date) = date_on_first_line {
                return Self {
                    description: rest,
                    date,
                };
            }
        }

        Self {
            description: s,
            date: Local::today().naive_local(),
        }
    }
}
