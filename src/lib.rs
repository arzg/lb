use chrono::{Local, NaiveDateTime};
use etcetera::app_strategy::{AppStrategy, AppStrategyArgs, Xdg};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::cmp::Ordering;
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
                    truncate(entry.description, 40),
                )
            })
            .join("\n")
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn read(location: &DbLocation, read_buf: &'a mut ReadBuf) -> anyhow::Result<Self> {
        let DbLocation(path) = location;

        if !path.exists() {
            return Self::initialize(location);
        }

        read_buf.file_contents = Some(fs::read(path)?);

        let db = bincode::deserialize(read_buf.file_contents.as_ref().unwrap())?;

        Ok(db)
    }

    pub fn write(&self, DbLocation(path): &DbLocation) -> anyhow::Result<()> {
        let db_file = safe_create_file(path)?;
        bincode::serialize_into(db_file, &self)?;

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

#[derive(Default)]
pub struct ReadBuf {
    file_contents: Option<Vec<u8>>,
}

#[derive(Debug, PartialEq, Eq, Ord, Serialize, Deserialize)]
pub struct Entry<'a> {
    description: &'a str,
    datetime: NaiveDateTime,
}

impl<'a> From<&'a str> for Entry<'a> {
    fn from(s: &'a str) -> Self {
        let s = s.trim();

        if let Some(first_line_ending) = s.find('\n') {
            let (first_line, rest) = s.split_at(first_line_ending);

            let datetime_on_first_line = NaiveDateTime::from_str(first_line);
            if let Ok(datetime) = datetime_on_first_line {
                return Self {
                    description: rest.trim(),
                    datetime,
                };
            }
        }

        Self {
            description: s,
            datetime: Local::now().naive_local(),
        }
    }
}

impl PartialOrd for Entry<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.datetime.partial_cmp(&other.datetime)
    }
}

fn truncate(s: &str, len: usize) -> Cow<'_, str> {
    if s.len() < len {
        s.into()
    } else {
        let s = &s[0..len];
        let mut chars: Vec<_> = s.chars().collect();
        let num_chars = chars.len();

        for c in &mut chars[num_chars - 3..] {
            *c = '.';
        }

        chars.iter().collect::<String>().into()
    }
}
