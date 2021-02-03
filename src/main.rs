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
        anyhow::bail!("you must specify a subcommand (‘add’, ‘delete’, ‘edit’ or ‘export’)");
    };

    let db_location = DbLocation::locate()?;
    let db = Db::read(&db_location)?;

    match subcommand.as_str() {
        "add" => add(db, db_location)?,
        "delete" => delete(db, db_location)?,
        "edit" => edit(db, db_location)?,
        "export" => export(db)?,
        _ => anyhow::bail!("invalid subcommand (try ‘add’, ‘delete’, ‘edit’ or ‘export’ instead)"),
    }

    Ok(())
}

fn add(mut db: Db, db_location: DbLocation) -> anyhow::Result<()> {
    let entry = get_input_from_editoe("")?;

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

fn edit(mut db: Db, db_location: DbLocation) -> anyhow::Result<()> {
    if db.is_empty() {
        anyhow::bail!("you can’t edit any entries because you don’t have any yet");
    }

    println!("Which entry would you like to edit?");
    println!("{}", db.entry_overview());
    let entry_to_edit = prompt("positive number")?;

    let current_description = db.get_entry_description(entry_to_edit);
    let edited_description = get_input_from_editoe(current_description)?;

    db.replace_entry_description(entry_to_edit, edited_description);

    db.write(&db_location)?;

    Ok(())
}

fn export(db: Db) -> anyhow::Result<()> {
    println!("{}", db.markdown());

    Ok(())
}

fn get_input_from_editoe(initial_content: &str) -> anyhow::Result<String> {
    let editor = env::var("EDITOR")?;

    let mut file = NamedTempFile::new()?;
    file.write_all(initial_content.as_bytes())?;

    let path = file.into_temp_path();

    let _ = Command::new("sh")
        .arg("-c")
        .arg(&format!("{} {}", editor, path.display()))
        .status();

    let edited_content = fs::read_to_string(&path)?;
    path.close()?;

    Ok(edited_content)
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
