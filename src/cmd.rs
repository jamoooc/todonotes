use std::collections::HashSet;
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
    Delete { path: PathBuf, arg: Vec<String> },
}

#[derive(Debug)]
pub enum CommandError {
    EmptyList,
    UnknownCommand,
    NoItemsProvided,
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
            CommandError::NoItemsProvided => write!(f, "Expected an item"),
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
            let arg = matches.opt_strs(FLAG_DELETE);
            if arg.len() < 1 {
                return Err(CommandError::MissingArgument("item number"))
            }
            return Ok(Command::Delete{ arg, path })
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

    fn extract_id(line: &str) -> Option<&str> {
        line.strip_prefix('[')
            .and_then(|s| s.split_once(']'))
            .map(|(id, _)| id)
    }

    fn delete_item(arg: &[String], path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let display_numbers = arg.iter()
            .map(|x| x.parse::<usize>())
            .collect::<Result<Vec<_>, _>>()?;

        let contents = fs::read_to_string(&path)?;
        let items: Vec<&str> = config::get_config_entries(&contents).collect();

        let max = display_numbers.iter().max().ok_or(CommandError::InvalidItemFormat)?;
        if *max > items.len() {
            return Err(Box::new(CommandError::ItemOutOfRange));
        }

        let mut ids_to_delete: HashSet<&str> = HashSet::new();
        for num in &display_numbers {
            let item = items[num - 1];
            if let Some(id) = Self::extract_id(item) {
                ids_to_delete.insert(id);
            }
        };

        let remaining: Vec<&str> = items.into_iter().filter(|line| {
            Self::extract_id(line).map(|id| !ids_to_delete.contains(id)).unwrap_or(true)
        }).collect();

        let mut file = fs::File::options().write(true).truncate(true).open(path)?;
        file.write_all(remaining.join("\n").as_bytes())?;

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
            println!("{:0len$}. {}", i + 1, text);
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
