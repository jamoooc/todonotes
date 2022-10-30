pub mod config {

  use getopts::Options;
  use std::{process, env, fs};
  use dirs::home_dir;
  use git2::Repository;
  use std::io::{Read, Write};
  
  use crate::cmd::cmd::Command;

  pub fn parse_args(program: &str, args: &Vec<String>) -> Result<Command, getopts::Fail> {
    // define command line options
    let mut opts = Options::new();
    opts.optopt("a", "add", "Add \"list item\"", "");
    opts.optflag("l", "list", "List all items");
    opts.optopt("d", "delete", "Delete list item \"n\"", "");
    opts.optflag("h", "help", "Display usage info");

    // parse options
    let matches = match opts.parse(&args[1..]) {
      Ok(m) => { m }
      Err(f) => {
        match f {
          getopts::Fail::ArgumentMissing(f) => {
            return Err(getopts::Fail::ArgumentMissing(f));
          },
          getopts::Fail::UnrecognizedOption(f) => {
            return Err(getopts::Fail::UnrecognizedOption(f));
          },
          getopts::Fail::OptionMissing(f) => {
            return Err(getopts::Fail::OptionMissing(f));
          },
          getopts::Fail::OptionDuplicated(f) => {
            return Err(getopts::Fail::OptionDuplicated(f));
          },
          getopts::Fail::UnexpectedArgument(f) => {
            return Err(getopts::Fail::UnexpectedArgument(f));
          }
        }
      }
    };

    // exit with usage info if the options include help 
    if matches.opt_present("h") {
      print_usage(&program, &opts);
      process::exit(0);
    }

    // exit with the usage information if there are remaining arguments
    if !matches.free.is_empty() {
      print_usage(&program, &opts);
      process::exit(1);
    }

    // create a Command struct with the option
    Ok(Command::get_command(matches))
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

  // get the user config path defined in XDG_CONFIG_HOME, or use the default
  fn get_user_config_dir() -> String {
    let home_dir = home_dir().unwrap();
    env::var("XDG_CONFIG_HOME").unwrap_or_else(|_| {
      String::from(format!("{}/.config", home_dir.display()))
    })
  }

  // get a handle to the user's config file, or create one if it doesn't exist
  fn get_config_file_handle(config_path: &str, default_list: &str) -> fs::File {
    fs::File::open(format!("{}/.todo_notes/config.toml", config_path)).unwrap_or_else(|_| {
      create_config_file(&config_path, &default_list);
      fs::File::open(format!("{}/.todo_notes/config.toml", config_path)).unwrap_or_else(|err| {
        eprintln!("Error opening \"{}/.todo_notes\" directory: {}", &config_path, err);
        process::exit(1);
      })
    })
  }

  // create a config file in the users config file directory and add provided list
  fn create_config_file(config_dir: &str, default_list: &str) {
    println!("Creating directory: \"{}/.todo_notes\"", config_dir);
    fs::create_dir_all(format!("{}/.todo_notes", config_dir)).unwrap_or_else(|err| {
      eprintln!("Error creating \"{}/.todo_notes\" directory: {}", config_dir, err);
      process::exit(1);
    });
    add_list_to_config(config_dir, default_list);
  }

  pub fn get_list_name() -> Result<String, ()> {
    // attempt to find a config file in the users config file path,
    // if unsuccessful create the config and a default task list
    let list = String::from("DEFAULT");
    let config_path = get_user_config_dir();
    let mut config_file = get_config_file_handle(&config_path, &list);

    // if the user is in a git repo, create/use a task list for this
    // dir referenced by the uppercased repo name in their config,
    // otherwise, use the default list
    let list = match get_repo_name() {
      Some(repo) => repo,
      None => list
    };

    let mut buf = String::new();
    config_file.read_to_string(&mut buf).unwrap();

    // find the config line entry for the current list name 
    let mut list_name = String::new();
    for line in buf.lines() {
      if line.starts_with(&list) {
        list_name = line.split('=').nth(1).unwrap().to_string();
      }
    }

    // if the list doesn't exist, add it to their config and create the list
    if list_name.len() == 0 {
      list_name = add_list_to_config(&config_path, &list);
    }

    Ok(list_name)
  }
}
