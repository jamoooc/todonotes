extern crate getopts;
use getopts::Options;
use std::env;
use std::{process, fs};
use std::io::{Read, Write};
use regex::Regex;
use dirs::home_dir;
use git2::Repository;

#[derive(Debug)]
enum CommandType {
  Add,
  List,
  Delete,
}

#[derive(Debug)]
pub struct Command {
  cmd: CommandType,
  arg: String,
  path: String
}

impl Command {

  fn new(cmd: CommandType, arg: String, path: String) -> Command {
    Command { cmd, arg, path }
  }

  fn print_usage(program: &str, opts: &Options) {
    let brief = format!("\nUsage: ./{} [options]", program);
    println!("{}", opts.usage(&brief));
  }

  fn get_repo_name() -> Option<String> {
    let current_dir = match env::current_dir() {
      Ok(dir) => dir,
      Err(e) => panic!("Error getting current working directory {e}"),
    };

    // step up directory tree to find git repo
    let repo = match Repository::discover(current_dir) {
      Ok(repo) => repo,
      Err(_) => {
        println!("No git repository found. Using default task list");
        return None;
      }
    };

    // strip the .git dir from the path
    let parent = match repo.path().parent() {
      Some(path) => path,
      None => {
        println!("Error getting parent path. Using default task list");
        return None;
      }
    };
    
    // get the root git dir name
    let list_name = match parent.file_stem() {
      Some(name) => name,
      None => {
        println!("Error getting root directory name. Using default task list");
        return None;
      }
    };

    match list_name.to_str() {
      Some(str) => {
        println!("Found git repository. Using todo list: {str}");
        Some(str.to_ascii_uppercase())
      },
      None => {
        println!("Error converting to String. Using default task list");
        return None;
      }
    }
  }
  
  fn create_config_file(config_dir: &str, default_list: &str) {
    println!("Creating directory: \"{}/.todo_notes\"", config_dir);
    fs::create_dir_all(format!("{}/.todo_notes", config_dir)).unwrap_or_else(|err| {
      eprintln!("Error creating \"{}/.todo_notes\" directory: {}", config_dir, err);
      process::exit(1);
    });
    Self::add_list_to_config(config_dir, default_list);
  }

  fn add_list_to_config(config_dir: &str, list_name: &str) -> String {
    println!("Creating task file: \"{config_dir}/.todo_notes/{}.txt\"", list_name.to_lowercase());

    let mut config = fs::File::options()
      .append(true)
      .read(true)
      .create(true)
      .open(format!("{}/.todo_notes/config.toml", config_dir))
      .unwrap();
    
    let mut buf = String::new();
    config.read_to_string(&mut buf).unwrap();

    let list = format!("\n{}={}/.todo_notes/{}.txt", list_name, config_dir, list_name.to_lowercase());

    // write list name, or append with a newline if the file is not empty
    match buf.lines().nth(0) {
      Some(_) => config.write(list.as_bytes()).unwrap(),
      None => config.write(list.trim_start().as_bytes()).unwrap()
    };

    // create the task list
    fs::File::create(format!("{}/.todo_notes/{}.txt", config_dir, list_name)).unwrap();

    String::from(format!("{}/.todo_notes/{}.txt", config_dir, list_name.to_lowercase()))
  }

  fn get_config() -> Result<String, ()> {
    // get the defined config file path, or use the default path
    let home_dir = home_dir().unwrap();
    let config_path = env::var("XDG_CONFIG_HOME").unwrap_or_else(|_| {
      String::from(format!("{}/.config", home_dir.display()))
    });
    
    // attempt to find a config file in the users config file path
    // and create one with a default task list if unsuccessful
    let list = String::from("DEFAULT");
    let mut config_file = fs::File::open(format!("{}/.todo_notes/config.toml", config_path)).unwrap_or_else(|_| {
      Self::create_config_file(&config_path, &list);
      fs::File::open(format!("{}/.todo_notes/config.toml", config_path)).unwrap()
    });

    let mut buf = String::new();
    config_file.read_to_string(&mut buf).unwrap();

    // if the user is in a git repo, create/use a task list for this
    // dir referenced by the uppercased repo name in the config
    let list = match Self::get_repo_name() {
      Some(repo) => repo,
      None => list
    };

    let mut list_name = String::new();
    for line in buf.lines() {
      if line.starts_with(&list) {
        list_name = line.split('=').nth(1).unwrap().to_string();
      }
    }

    if list_name.len() == 0 {
      list_name = Self::add_list_to_config(&config_path, &list);
    }

    Ok(list_name)
  }

  fn get_command(matches: getopts::Matches) -> Command {
    let cmd: CommandType;
    let mut arg = String::from("");

    if matches.opt_present("a") {
      cmd = CommandType::Add;
      arg = String::from(matches.opt_str("a").unwrap());
    } else if matches.opt_present("d") {
      cmd = CommandType::Delete;
      arg = String::from(matches.opt_str("d").unwrap());
    } else if matches.opt_present("l") {
      cmd = CommandType::List;
    } else {
      process::exit(1);
    }

    let path = Self::get_config().unwrap();
    Command::new(cmd, arg, path)
  }

  pub fn parse_args(program: &str, args: &Vec<String>) -> Command {
    // define command line options
    let mut opts = Options::new();
    opts.optopt("a", "add", "Add \"list item\"", "");
    opts.optflag("l", "list", "List all items");
    opts.optopt("d", "delete", "Delete list item \"n\"", "");
    opts.optflag("h", "help", "Display usage info");

    // parse options
    let matches = match opts.parse(&args[1..]) {
      Ok(m) => { m }
      Err(f) => { panic!("{}", f.to_string()) }
    };

    // exit with usage info if the options include help 
    if matches.opt_present("h") {
      Self::print_usage(&program, &opts);
      process::exit(0);
    }

    // exit with the usage information if there are remaining arguments
    if !matches.free.is_empty() {
      Self::print_usage(&program, &opts);
      process::exit(1);
    }

    // create a Command struct with the option
    Self::get_command(matches)
  }

  fn add_list_item(command: Command) {
    let mut file = fs::File::options()
      .append(true)
      .read(true)
      .open(&command.path)
      .unwrap();
  
    let mut buf = String::new();
    file.read_to_string(&mut buf).unwrap();

    let nlines = buf.lines().count();
    let item = format!("\n{:0>2}. {}", nlines + 1, command.arg);

    match buf.lines().nth(0) {
      Some(_) => {
        // append item with a newline if the file is not empty
        match file.write(item.as_bytes()) {
          Ok(n) => n,
          Err(e) => panic!("Error writing to file: {e}")
        };
      },
      None => {
        // trim the newline if we're adding the first item
        match file.write(item.trim_start().as_bytes()) {
          Ok(n) => n,
          Err(e) => panic!("Error writing to file: {e}")
        };
      }
    };

    println!("Added new item: {}", item.trim_start());
    Self::print_list_items(command);
  }

  fn delete_list_item(command: Command) {
    // open the file for read/write
    let mut file = fs::File::options()
      .write(true)
      .read(true)
      .open(&command.path)
      .unwrap();
  
    // read the file into a buffer
    let mut buf = String::new();
    file.read_to_string(&mut buf).unwrap();
    
    // count and check the number of items
    let nitem: usize = command.arg.parse().unwrap();
    let nlines: usize = buf.lines().count();
  
    if nitem > nlines {
      println!("List item number exceeds list length.");
      process::exit(1);
    }
  
    // split into a vector of individual items (lines) and remove the nth item
    let mut t: Vec<&str> = buf.lines().collect();
    t.remove(nitem - 1);

    // step through creating new strings and increment the item number
    // if it's > than the deleted item. This will become our new file
    let item_num_regex = Regex::new(r"^\d{1,2}\. ").unwrap();
    let mut new_items: Vec<String> = Vec::new();
    for item in &t {
      let item_str = item_num_regex.split(item).nth(1).unwrap();
      
      let item_num: usize = item.split('.').nth(0).unwrap().parse().unwrap();
      let item_num = if item_num > nitem { item_num - 1 } else { item_num };

      new_items.push(format!("{:0>2}. {}", item_num, item_str));
    }

    // open the file, trucating to length 0 and write the updated item list
    let mut file = fs::File::options()
      .write(true)
      .truncate(true)
      .open(&command.path)
      .unwrap();
    file.write(&new_items.join("\n").as_bytes()).unwrap();
  
    println!("Deleted list item: {}", nitem);
    Self::print_list_items(command);
  }
  
  fn print_list_items(command: Command) {
    let contents = fs::read_to_string(command.path).unwrap_or_else(|e| { 
      panic!("Error opening file: {e}");
    });
    for line in contents.lines() {
      println!("{line}");
    }
  }

  pub fn run(command: Command) {
    match command.cmd {
      CommandType::Add => Self::add_list_item(command),
      CommandType::List => Self::print_list_items(command),
      CommandType::Delete => Self::delete_list_item(command),
    }
  }
}
