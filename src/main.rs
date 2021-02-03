use journal::{Db, DbLocation, Entry, ReadBuf};
use std::env;
use std::fs;
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

    let db_location = DbLocation::locate()?;
    let mut read_buf = ReadBuf::default();
    let mut db = Db::read(&db_location, &mut read_buf)?;

    db.push_entry(Entry::from(entry.as_str()));
    db.write(&db_location)?;

    Ok(())
}

fn export() -> anyhow::Result<()> {
    let db_location = DbLocation::locate()?;
    let mut read_buf = ReadBuf::default();
    let db = Db::read(&db_location, &mut read_buf)?;

    println!("{}", db.markdown());

    Ok(())
}
