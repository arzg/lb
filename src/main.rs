use chrono::{Local, NaiveDate};
use etcetera::app_strategy::{AppStrategy, AppStrategyArgs, Xdg};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::NamedTempFile;

fn main() -> anyhow::Result<()> {
    let mut args = env::args().skip(1);

    let subcommand = if let Some(s) = args.next() {
        s
    } else {
        anyhow::bail!("you must specify a subcommand (‘add’ or ‘export’)");
    };

    match subcommand.as_str() {
        "add" => add()?,
        "export" => export()?,
        _ => anyhow::bail!("invalid subcommand (try ‘add’ or ‘export’ instead)"),
    }

    Ok(())
}

fn add() -> anyhow::Result<()> {
    let editor = env::var("EDITOR")?;

    let file = NamedTempFile::new()?;
    let path = file.into_temp_path();

    let _ = Command::new("sh")
        .arg("-c")
        .arg(&format!("{} {}", editor, path.display()))
        .status();

    let entry = fs::read_to_string(&path)?;
    path.close()?;

    let mut db = read_db()?;
    db.push_entry(&entry);

    write_db(&db)?;

    Ok(())
}

fn export() -> anyhow::Result<()> {
    let db = read_db()?;
    println!("{}", db.markdown());

    Ok(())
}

fn read_db() -> anyhow::Result<Db> {
    let db_path = get_db_path()?;

    if !db_path.exists() {
        let db = Db::default();
        write_db(&db)?;
        return Ok(db);
    }

    let db_file = File::open(db_path)?;
    let db = bincode::deserialize_from(db_file)?;

    Ok(db)
}

fn write_db(db: &Db) -> anyhow::Result<()> {
    let db_path = get_db_path()?;
    let db_file = safe_create_file(&db_path)?;
    bincode::serialize_into(db_file, &db)?;

    Ok(())
}

fn get_db_path() -> anyhow::Result<PathBuf> {
    let xdg = Xdg::new(AppStrategyArgs {
        top_level_domain: "io.github".to_string(),
        author: "arzg".to_string(),
        app_name: "journal".to_string(),
    })?;

    Ok(xdg.in_data_dir("db"))
}

fn safe_create_file(path: &Path) -> anyhow::Result<File> {
    fs::create_dir_all(path.parent().unwrap())?;
    Ok(File::create(path)?)
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct Db {
    entries: Vec<Entry>,
}

impl Db {
    fn push_entry(&mut self, description: &str) {
        self.entries.push(Entry {
            description: description.trim().to_string(),
            date: Local::today().naive_local(),
        });
    }

    fn markdown(&self) -> String {
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
