
use std::error::Error;
use std::{process, fs};
use std::io::{Read, Write};
use regex::Regex;

use crate::config;

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

  pub fn get_command(matches: getopts::Matches) -> Command {
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

    let path = config::get_list_name().unwrap();
    Command::new(cmd, arg, path)
  }


  fn add_list_item(command: Command) -> Result<(), std::io::Error>  {

    let mut file = match fs::File::options()
      .append(true)
      .read(true)
      .open(&command.path) {
        Ok(file) => file,
        Err(e) => return Err(e)
    };
  
    let mut buf = String::new();
    if let Err(e) = file.read_to_string(&mut buf) {
      return Err(e);
    };

    let nlines = buf.lines().count();
    let item = format!("\n{:0>2}. {}", nlines + 1, command.arg);

    match buf.lines().nth(0) {
      Some(_) => {
        // append item with a newline if the file is not empty
        match file.write(item.as_bytes()) {
          Ok(n) => n,
          Err(e) => return Err(e)
        };
      },
      None => {
        // trim the newline if we're adding the first item
        match file.write(item.trim_start().as_bytes()) {
          Ok(n) => n,
          Err(e) => return Err(e)
        };
      }
    };

    println!("Added new item: {}", item.trim_start());
    if let Err(e) = Self::print_list_items(command)  { 
      return Err(e);
    };
    Ok(())
  }

  fn delete_list_item(command: Command) -> Result<(), std::io::Error>  {

    // open the file for read/write
    let mut file = match fs::File::options()
      .write(true)
      .read(true)
      .open(&command.path)  {
        Ok(file) => file,
        Err(e) => return Err(e)
    };
  
    // read the file into a buffer
    let mut buf = String::new();
    if let Err(e) = file.read_to_string(&mut buf) {
      return Err(e);
    }
    
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

    let item_num_regex = match Regex::new(r"^(\d{1,2}\. )([^']+)") {
      Ok(re) => re,
      Err(e) => panic!("Error creating regular expression: {e}")
    };

    // step through creating new strings and increment the item number
    // if it's > than the deleted item. This will become our new file
    let mut new_items: Vec<String> = Vec::new();
    for item in &t {
      // split the item string into it's num and text with capture groups 
      let caps = match item_num_regex.captures(item) {
        Some(caps) => caps,
        None => panic!("Error processing list item")
      };
      
      let item_str = caps.get(2).unwrap().as_str();
      let item_num: usize = caps
        .get(1)
        .unwrap()
        .as_str()
        .trim_matches(|c| c == '.' || c == ' ')
        .parse()
        .unwrap();

      // if the item number is > that the removed item, decrement it's num
      let item_num = if item_num > nitem { item_num - 1 } else { item_num };
      new_items.push(format!("{:0>2}. {}", item_num, item_str));
    }

    // open the file, trucating to length 0 and write the updated item list
    let mut file = match fs::File::options()
      .write(true)
      .truncate(true)
      .open(&command.path) {
        Ok(file) => file,
        Err(e) => return Err(e)
    };

    if let Err(e) = file.write(&new_items.join("\n").as_bytes()) {
      return Err(e);
    }
  
    println!("Deleted list item: {}", nitem);
    if let Err(e) = Self::print_list_items(command)  { 
      return Err(e);
    };
    Ok(())
  }

  fn print_list_items(command: Command) -> Result<(), std::io::Error> {
    let contents = match fs::read_to_string(command.path) { 
      Ok(content) => content,
      Err(e) => return Err(e)
    };
    for line in contents.lines() {
      println!("{line}");
    }
    Ok(())
  }

  pub fn run(command: Command) -> Result<(), Box<dyn Error>> {
    match command.cmd {
      CommandType::Add => Self::add_list_item(command)?,
      CommandType::List => Self::print_list_items(command)?,
      CommandType::Delete => Self::delete_list_item(command)?,
    }
    Ok(())
  }
}
