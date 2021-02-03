use chrono::{Local, NaiveDate};
use itertools::Itertools;
use serde::{Deserialize, Serialize};

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
}

#[derive(Debug, Serialize, Deserialize)]
struct Entry {
    description: String,
    date: NaiveDate,
}
