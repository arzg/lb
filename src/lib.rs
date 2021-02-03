use chrono::{Local, NaiveDateTime};
use etcetera::app_strategy::{AppStrategy, AppStrategyArgs, Xdg};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::cmp::Ordering;
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::str::FromStr;
use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Db {
    entries: Vec<Entry>,
}

impl Db {
    pub fn push_entry(&mut self, entry: Entry) {
        self.entries.push(entry);
        self.entries.sort_unstable();
    }

    pub fn delete_entry(&mut self, idx: usize) {
        self.entries.remove(idx);
    }

    pub fn markdown(&self) -> String {
        self.entries
            .iter()
            .map(|entry| format!("- {}: {}", entry.datetime.date(), entry.description))
            .join("\n")
    }

    pub fn entry_overview(&self) -> String {
        self.entries
            .iter()
            .enumerate()
            .map(|(idx, entry)| {
                format!(
                    "[{:04}] {}: {}",
                    idx,
                    entry.datetime.date(),
                    truncate(&entry.description, 40),
                )
            })
            .join("\n")
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn replace_entry_description(&mut self, idx: usize, description: String) {
        self.entries[idx].description = description;
    }

    pub fn get_entry_description(&self, idx: usize) -> &str {
        &self.entries[idx].description
    }

    pub fn read(location: &DbLocation) -> anyhow::Result<Self> {
        let DbLocation(path) = location;

        if !path.exists() {
            return Self::initialize(location);
        }

        let file = File::open(path)?;
        let db = bincode::deserialize_from(file)?;

        Ok(db)
    }

    pub fn write(&self, DbLocation(path): &DbLocation) -> anyhow::Result<()> {
        let file = safe_create_file(path)?;
        bincode::serialize_into(file, &self)?;

        Ok(())
    }

    fn initialize(location: &DbLocation) -> anyhow::Result<Self> {
        let db = Self::default();
        db.write(location)?;

        Ok(db)
    }
}

pub struct DbLocation(PathBuf);

impl DbLocation {
    pub fn locate() -> anyhow::Result<Self> {
        let xdg = Xdg::new(AppStrategyArgs {
            top_level_domain: "io.github".to_string(),
            author: "arzg".to_string(),
            app_name: "journal".to_string(),
        })?;

        Ok(Self(xdg.in_data_dir("db")))
    }
}

fn safe_create_file(path: &Path) -> anyhow::Result<File> {
    fs::create_dir_all(path.parent().unwrap())?;
    Ok(File::create(path)?)
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Entry {
    description: String,
    datetime: NaiveDateTime,
}

impl From<&str> for Entry {
    fn from(s: &str) -> Self {
        let s = s.trim();

        if let Some(first_line_ending) = s.find('\n') {
            let (first_line, rest) = s.split_at(first_line_ending);

            let datetime_on_first_line = NaiveDateTime::from_str(first_line);
            if let Ok(datetime) = datetime_on_first_line {
                return Self {
                    description: rest.trim().to_string(),
                    datetime,
                };
            }
        }

        Self {
            description: s.to_string(),
            datetime: Local::now().naive_local(),
        }
    }
}

impl PartialOrd for Entry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.datetime.partial_cmp(&other.datetime)
    }
}

impl Ord for Entry {
    fn cmp(&self, other: &Self) -> Ordering {
        self.datetime.cmp(&other.datetime)
    }
}

fn truncate(s: &str, len: usize) -> Cow<'_, str> {
    let mut graphemes: Vec<_> = s.graphemes(true).collect();

    if graphemes.len() < len {
        s.into()
    } else {
        let trimmed = &mut graphemes[..len];
        let last_three = &mut trimmed[len - 3..];

        for c in last_three {
            *c = ".";
        }

        trimmed.iter().copied().collect::<String>().into()
    }
}
