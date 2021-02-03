use journal::{Db, DbLocation, Entry};
use std::io::{self, Write};
use std::process::Command;
use std::str::FromStr;
use std::{env, fs};
use tempfile::NamedTempFile;

fn main() -> anyhow::Result<()> {
    let mut args = env::args().skip(1);

    let subcommand = if let Some(s) = args.next() {
        s
    } else {
        anyhow::bail!("you must specify a subcommand (‘add’, ‘delete’ or ‘export’)");
    };

    let db_location = DbLocation::locate()?;
    let db = Db::read(&db_location)?;

    match subcommand.as_str() {
        "add" => add(db, db_location)?,
        "delete" => delete(db, db_location)?,
        "export" => export(db)?,
        _ => anyhow::bail!("invalid subcommand (try ‘add’, ‘delete’ or ‘export’ instead)"),
    }

    Ok(())
}

fn add(mut db: Db, db_location: DbLocation) -> anyhow::Result<()> {
    let editor = env::var("EDITOR")?;

    let file = NamedTempFile::new()?;
    let path = file.into_temp_path();

    let _ = Command::new("sh")
        .arg("-c")
        .arg(&format!("{} {}", editor, path.display()))
        .status();

    let entry = fs::read_to_string(&path)?;
    path.close()?;

    db.push_entry(Entry::from(entry.as_str()));
    db.write(&db_location)?;

    Ok(())
}

fn delete(mut db: Db, db_location: DbLocation) -> anyhow::Result<()> {
    if db.is_empty() {
        anyhow::bail!("you can’t delete any entries because you don’t have any yet");
    }

    println!("Which entry would you like to delete?");
    println!("{}", db.entry_overview());
    let entry_to_delete = prompt("positive number")?;

    db.delete_entry(entry_to_delete);
    db.write(&db_location)?;

    Ok(())
}

fn export(db: Db) -> anyhow::Result<()> {
    println!("{}", db.markdown());

    Ok(())
}

fn prompt<T>(data_type_name: &str) -> anyhow::Result<T>
where
    T: FromStr,
    T::Err: std::error::Error + Send + Sync + 'static,
{
    loop {
        print!("> ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        match input.trim().parse() {
            Ok(value) => return Ok(value),
            Err(e) => {
                println!("Error: {:?}", anyhow::Error::new(e));
                eprintln!("Note: expected a {}", data_type_name);
            }
        }
    }
}
