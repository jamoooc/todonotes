use regex::Regex;
use std::error::Error;
use std::io::{Read, Write};
use std::{fs, process};

use crate::config;

// #[derive(Debug)]
// enum CommandType {
//     Add,
//     List,
//     Delete,
//     Reset,
// }

// #[derive(Debug)]
// pub struct Command {
//     cmd: CommandType,
//     arg: String,
//     path: String,
// }

#[derive(Debug)]
pub enum Command {
    Add     { path: String, arg: String },
    List    { path: String },
    Reset   { path: String },
    Delete  { path: String, arg: String },
    // Create  { path: String, arg: Vec<String> }
}

impl Command {
    // TODO: this should return an error
    pub fn get_command(matches: getopts::Matches) -> Command {
        let arg: String; // TODO: this might be a Vec<String>

        let mut provided_path = String::new();
        if matches.opt_present("t") {
            provided_path = String::from(matches.opt_str("t").unwrap());
        }
        let path = config::get_list_name(&provided_path).unwrap(); // todo no unwrap

        if matches.opt_present("a") {
            match matches.opt_str("a") {
                Some(arg) => return Command::Add{ arg, path },
                None => {
                    eprintln!("Missing argument for -a command");
                    process::exit(1)
                }
            }
        } else if matches.opt_present("d") {
            arg = String::from(matches.opt_str("d").unwrap());
            return Command::Delete{ arg, path }
        } else if matches.opt_present("l") {
            return Command::List{ path }
        } else if matches.opt_present("r") {
            return Command::Reset{ path }
        } else {
            process::exit(1);
        }
    }

    fn add_item(arg: String, path: String) -> Result<(), std::io::Error> {
        println!("path: {:?}", path);
        let mut file = match fs::File::options()
            .append(true)
            .read(true)
            .open(&path)
        {
            Ok(file) => file,
            Err(e) => return Err(e),
        };

        let mut buf = String::new();
        if let Err(e) = file.read_to_string(&mut buf) {
            return Err(e);
        };

        let nlines = buf.lines().count();
        let item = format!("\n{:0>2}. {}", nlines + 1, arg);

        match buf.lines().nth(0) {
            Some(_) => {
                // append item with a newline if the file is not empty
                match file.write(item.as_bytes()) {
                    Ok(n) => n,
                    Err(e) => return Err(e),
                };
            }
            None => {
                // trim the newline if we're adding the first item
                match file.write(item.trim_start().as_bytes()) {
                    Ok(n) => n,
                    Err(e) => return Err(e),
                };
            }
        };

        // println!("Added new item: {}", item.trim_start());
        // if let Err(e) = Self::print_list_items(command) {
        //     return Err(e);
        // };
        Ok(())
    }

    fn delete_item(arg: String, path: String) -> Result<(), std::io::Error> {
        // open the file for read/write
        let mut file = match fs::File::options()
            .write(true)
            .read(true)
            .open(&path)
        {
            Ok(file) => file,
            Err(e) => return Err(e),
        };

        // read the file into a buffer
        let mut buf = String::new();
        if let Err(e) = file.read_to_string(&mut buf) {
            return Err(e);
        }

        // collect the list items to delete in a vector
        let mut item_numbers: Vec<usize> = arg
            .split_whitespace()
            .map(|x| match x.parse::<usize>() {
                Ok(x) => x,
                Err(e) => panic!("Invalid item number format. {e}"),
            })
            .collect();

        // sort items in reverse so we don't affect indexing by
        // removing earlier items, and remove any duplicates
        item_numbers.sort_by(|a, b| b.cmp(a));
        item_numbers.dedup();

        // get the max list num and check it doesn't exceed the total items
        let nlines: usize = buf.lines().count();
        let max_item = match item_numbers.iter().max() {
            Some(max) => max,
            None => panic!("Unable to determine maximum list item"),
        };

        if *max_item > nlines {
            println!("List item number exceeds list length.");
            process::exit(1);
        }

        // split current list into a vector of list items (lines),
        // remove each given item and store a reference to print
        // the removed items
        let mut list_items: Vec<&str> = buf.lines().collect();
        let mut removed_items: Vec<&str> = Vec::new();
        for n in item_numbers.iter() {
            removed_items.push(list_items.remove(n - 1));
        }

        let item_num_regex = match Regex::new(r"^(\d{1,2}\. )([^']+)") {
            Ok(re) => re,
            Err(e) => panic!("Error creating regular expression: {e}"),
        };

        // step through creating new strings and incrementing the item
        // number, this will become our new file
        let mut new_items: Vec<String> = Vec::new();
        for (i, item) in list_items.iter().enumerate() {
            let caps = match item_num_regex.captures(item) {
                Some(caps) => caps,
                None => panic!("Error processing list item"),
            };

            let item_str = caps.get(2).unwrap().as_str();
            new_items.push(format!("{:0>2}. {}", i + 1, item_str));
        }

        // open the file, trucate and write the updated item list
        let mut file = match fs::File::options()
            .write(true)
            .truncate(true)
            .open(&path)
        {
            Ok(file) => file,
            Err(e) => return Err(e),
        };

        if let Err(e) = file.write(&new_items.join("\n").as_bytes()) {
            return Err(e);
        }

        removed_items.reverse(); // order the items by item num
        println!("Deleted {} list items:", item_numbers.len());
        for item in removed_items.iter() {
            println!("\t{}", item);
        }

        // if let Err(e) = Self::print_items(command) {
        //     return Err(e);
        // };

        Ok(())
    }

    fn reset_list(path: String) -> Result<(), std::io::Error> {
        // strip the list name from the path
        let list_name_regex = match Regex::new(r"/([\w_]+).txt$") {
            Ok(re) => re,
            Err(e) => panic!("Error creating regular expression: {e}"),
        };

        let caps = match list_name_regex.captures(&path) {
            Some(caps) => caps,
            None => panic!("Error getting list name"),
        };

        let list_name = caps.get(1).unwrap().as_str();

        // opening the file with the truncate option will set its length to 0
        match fs::File::options()
            .write(true)
            .truncate(true)
            .open(&path)
        {
            Ok(file) => {
                println!("List \"{}\" state reset.", list_name);
                file
            }
            Err(e) => {
                println!("Error resetting list: {}", list_name);
                return Err(e);
            }
        };

        // if let Err(e) = Self::print_items(command) {
        //     return Err(e);
        // };
        Ok(())
    }

    fn print_items(path: String) -> Result<(), std::io::Error> {
        let contents = match fs::read_to_string(path) {
            Ok(content) => content,
            Err(e) => return Err(e),
        };
        for line in contents.lines() {
            println!("{line}");
        }
        Ok(())
    }

    pub fn run(command: Command) -> Result<(), Box<dyn Error>> {
        match command {
            Command::Add    { arg, path } => Self::add_item(arg, path)?,
            Command::List   { path } => Self::print_items(path)?,
            Command::Delete { arg, path } => Self::delete_item(arg, path)?,
            Command::Reset  { path } => Self::reset_list(path)?,
            // Command::Create { arg, path } => {
            //     todo!("todo")
            // }
        }
        Ok(())
    }
}
