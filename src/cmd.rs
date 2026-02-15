use std::error::Error;
use std::io::{Read, Write};
use std::fs;
use uuid::Uuid;
use std::path::{Path, PathBuf};
use log::{debug};

use crate::config;
use crate::{FLAG_CREATE,FLAG_DELETE,FLAG_SWITCH,FLAG_RESET,FLAG_PRINT};

#[derive(Debug)]
pub struct ListItem {
    pub id: String,
    pub item: String,
}

#[derive(Debug)]
pub enum Command {
    Print  { path: PathBuf },
    Reset  { path: PathBuf },
    Create { path: PathBuf, arg: String },
    Delete { path: PathBuf, arg: String },
}

#[derive(Debug)]
pub enum CommandError {
    EmptyList,
    UnknownCommand,
    ItemOutOfRange,
    InvalidItemFormat,
    MissingArgument(&'static str),
    ConfigError(Box<dyn std::error::Error>),
}

impl std::fmt::Display for CommandError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CommandError::EmptyList => write!(f, "The list is empty"),
            CommandError::UnknownCommand => write!(f, "Unknown command"),
            CommandError::ConfigError(err) => write!(f, "Config error {}", err),
            CommandError::ItemOutOfRange => write!(f, "Item number out of range"),
            CommandError::InvalidItemFormat => write!(f, "Failed to parse list item"),
            CommandError::MissingArgument(arg) => write!(f, "Missing argument: {}", arg),
        }
    }
}

impl std::error::Error for CommandError {}

impl From<Box<dyn std::error::Error>> for CommandError {
    fn from(err: Box<dyn std::error::Error>) -> Self {
        CommandError::ConfigError(err)
    }
}

impl Command {
    pub fn get_command(matches: getopts::Matches) -> Result<Command, CommandError> {
        let path = if let Some(list_name) = matches.opt_str(FLAG_SWITCH) {
            config::resolve_named_list(&list_name)?
        } else {
            config::resolve_repo_list()?
        };

        if matches.opt_present(FLAG_CREATE) {
            return match matches.opt_str(FLAG_CREATE) {
                Some(arg) => Ok(Command::Create{ arg, path }),
                None => Err(CommandError::MissingArgument("item to add"))
            }
        }

        if matches.opt_present(FLAG_DELETE) {
            return match matches.opt_str(FLAG_DELETE) {
                Some(arg) => Ok(Command::Delete{ arg, path }),
                None => Err(CommandError::MissingArgument("item number"))
            }
        }

        if matches.opt_present(FLAG_PRINT) {
            return Ok(Command::Print{ path })
        }

        if matches.opt_present(FLAG_RESET) {
            return Ok(Command::Reset{ path })
        }

        Err(CommandError::UnknownCommand)
    }

    fn generate_id() -> String {
        Uuid::new_v4().simple().to_string()[0..8].to_string()
    }

    fn create_item(arg: &str, path: &Path) -> Result<(), std::io::Error> {
        let id = Self::generate_id();
        let list_item = ListItem { id, item: arg.to_string() };

        let mut file = fs::File::options().append(true).read(true).open(&path)?;
        let mut buf = String::new();
        file.read_to_string(&mut buf)?;

        if buf.is_empty() {
            write!(file, "[{}] {}", list_item.id, list_item.item)?;
        } else {
            writeln!(file, "[{}] {}", list_item.id, list_item.item)?;
        }

        Ok(())
    }

    fn parse_item_text(item: &str) -> Result<&str, CommandError> {
        item.split_once(". ").map(|(_, text)| text).ok_or(CommandError::InvalidItemFormat)
    }

    fn delete_item(arg: &str, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let mut file = fs::File::options().write(true).read(true).open(&path)?;

        let mut buf = String::new();
        file.read_to_string(&mut buf)?;

        // collect the list items to delete in a vector
        let mut item_numbers: Vec<usize> = arg
            .split_whitespace()
            .map(|x| x.parse::<usize>())
            .collect::<Result<Vec<_>, _>>()?;

        // sort items in reverse so we don't affect indexing by
        // removing earlier items, and remove any duplicates
        item_numbers.sort_by(|a, b| b.cmp(a));
        item_numbers.dedup();

        // get the max list num and check it doesn't exceed the total items
        let max_item = item_numbers.iter().max().ok_or(CommandError::EmptyList)?;
        let nlines = config::get_config_entries(&buf).count();
        if *max_item > nlines {
            return Err(Box::new(CommandError::ItemOutOfRange));
        }

        // split current list into a vector of list items (lines),
        // remove each given item and store a reference to print
        // the removed items
        let mut list_items: Vec<&str> = config::get_config_entries(&buf).collect();
        let mut removed_items: Vec<&str> = Vec::new();
        for n in item_numbers.iter() {
            removed_items.push(list_items.remove(n - 1));
        }

        let mut new_items: Vec<String> = Vec::new();
        for (i, item) in list_items.iter().enumerate() {
            let item_text = Self::parse_item_text(item)?;
            new_items.push(format!("{:0>2}. {}", i + 1, item_text));
        }

        // open the file, trucate and write the updated item list
        let mut file = fs::File::options().write(true).truncate(true).open(&path)?;
        file.write_all(&new_items.join("\n").as_bytes())?;

        removed_items.reverse(); // order the items by item num
        debug!("Deleted {} list items:", item_numbers.len());
        for item in removed_items.iter() {
            debug!("\t{}", item);
        }

        Ok(())
    }

    fn reset_list(path: &Path) -> Result<(), std::io::Error> {
        fs::File::options().write(true).truncate(true).open(&path)?;
        let list_name = path.file_stem().and_then(|n| n.to_str()).unwrap_or("unknown");
        debug!("List \"{}\" state reset.", list_name);
        Ok(())
    }

    fn strip_id_prefix(line: &str) -> &str {
        if let Some(idx) = line.find(']') {
            line[idx + 1..].trim_start()
        } else {
            line
        }
    }

    fn print_items(path: &Path) -> Result<(), std::io::Error> {
        let contents = fs::read_to_string(path)?;
        let len = contents.lines().count().to_string().len();
        for (i, line) in contents.lines().enumerate() {
            let text = Self::strip_id_prefix(line);
            println!("{:0len$}. {}", i, text);
        }
        Ok(())
    }

    pub fn run(command: Command) -> Result<(), Box<dyn Error>> {
        match command {
            Command::Create { arg, path } => Self::create_item(&arg, &path)?,
            Command::Print  { path      } => Self::print_items(&path)?,
            Command::Delete { arg, path } => Self::delete_item(&arg, &path)?,
            Command::Reset  { path      } => Self::reset_list(&path)?,
        }
        Ok(())
    }
}
