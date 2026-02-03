use dirs::home_dir;
use git2::{ErrorCode, Repository};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::{env, fs};

const DEFAULT_LIST: &str = "DEFAULT";
const CONFIG_DIR: &str = ".todo_notes";
const CONFIG_FILE: &str = "config.toml";

// TODO: how to log (and set a log level)

fn get_repo_name() -> Result<Option<String>, Box<dyn std::error::Error>> {
    let cwd = env::current_dir()?;
    let repo = match Repository::discover(cwd) {
        Ok(repo) => repo,
        Err(e) if e.code() == ErrorCode::NotFound => return Ok(None),
        Err(e) => return Err(Box::new(e))
    };

    let repo_name = repo.workdir()
        .and_then(|path| path.file_name())
        .and_then(|name| name.to_str())
        .map(|s| s.to_ascii_uppercase()); // TODO: the caller should uppercase this

    Ok(repo_name)
}

fn add_list_to_config(config_file: &mut fs::File, config_dir: &Path, list_name: &str) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let list_path = config_dir.join(CONFIG_DIR).join(format!("{}.txt", list_name));
    writeln!(config_file, "{}={}", list_name, list_path.display())?;
    fs::File::create(&list_path)?;
    Ok(list_path)
}

fn get_user_config_dir() -> Result<PathBuf, Box<dyn std::error::Error>> {
    if let Ok(config_home) = env::var("XDG_CONFIG_HOME") {
        return Ok(PathBuf::from(config_home));
    };

    let mut path = home_dir().ok_or("Failed to determine home directory")?;
    path.push(".config");
    Ok(path)
}

// TODO: probably doesn't live here
pub fn get_config_entries(buf: &str) -> impl Iterator<Item = &str> {
    buf.lines().filter(|line| !line.trim().is_empty())
}

fn open_or_create_config_file(config_dir: &Path) -> Result<fs::File, Box<dyn std::error::Error>> {
    let path = config_dir.join(CONFIG_DIR).join(CONFIG_FILE);
    match fs::OpenOptions::new().read(true).write(true).open(&path) {
        Ok(file) => Ok(file),
        Err(e) if e.kind() == io::ErrorKind::NotFound => {
            fs::create_dir_all(config_dir.join(CONFIG_DIR))?;
            Ok(fs::OpenOptions::new()
                .read(true)
                .append(true)
                .create_new(true)
                .open(&path)?)
        },
        Err(e) => Err(Box::new(e))
    }
}

pub fn get_list_path(name_override: &str) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let name = if !name_override.is_empty() {
        name_override.to_uppercase()
    } else {
        match get_repo_name() {
            Ok(Some(repo)) => repo,
            Ok(None) => DEFAULT_LIST.to_string(),
            Err(e) => return Err(e),
        }
    };

    let config_path: PathBuf = get_user_config_dir()?;
    let mut config_file: fs::File = open_or_create_config_file(&config_path)?;

    let mut buf = String::new();
    config_file.read_to_string(&mut buf).unwrap();

    let mut list_path = String::new();
    for line in get_config_entries(&buf) {
        if line.starts_with(&name) {
            list_path = line.split('=').nth(1).unwrap().to_string();
        }
    }

    if list_path.is_empty() {
        let rv = add_list_to_config(&mut config_file, &config_path, &name)?;
        return Ok(rv);
    }

    Ok(PathBuf::from(list_path))
}
