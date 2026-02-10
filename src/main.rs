use std::env;
use std::process;
use getopts::Options;
use env_logger::{Builder};
use log::{debug,LevelFilter};
use std::str::FromStr;

use todo_notes::cmd;

fn print_usage(program: &str, opts: &Options) {
    let brief = format!("\nUsage: {} [options]", program);
    println!("{}", opts.usage(&brief));
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();

    let mut opts = Options::new();
    opts.optopt("a", "add", "Add \"item\"", "");
    opts.optopt("d", "delete", "Delete list item n, or items \"n n n\"", "");
    opts.optopt("s", "switch", "Switch to another list", "./todo_notes -s default"); // s switch
    opts.optopt("l", "level", "Set the log level", "INFO");

    opts.optflag("p", "print", "Print all items");
    opts.optflag("r", "reset", "Reset list state");
    opts.optflag("h", "help", "Display usage info");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => {
            eprintln!("Argument error: {}", f);
            print_usage(&program, &opts);
            process::exit(1);
        },
    };

    if matches.opt_present("h") {
        print_usage(&program, &opts);
        process::exit(0);
    }

    if !matches.free.is_empty() {
        print_usage(&program, &opts);
        process::exit(1);
    }

    let log_level = if matches.opt_present("l") {
        match matches.opt_str("l") {
            Some(arg) => LevelFilter::from_str(&arg).unwrap_or(LevelFilter::Off),
            None => LevelFilter::Off
        }
    } else {
        LevelFilter::Off
    };

    Builder::from_default_env().filter_level(log_level).init();

    let command = match cmd::Command::get_command(matches) {
        Ok(cmd) => cmd,
        Err(e) => {
            eprintln!("Error: {:?}", e);
            process::exit(1);
        }
    };

    debug!("Recieved command");
    if let Err(e) = cmd::Command::run(command) {
        eprintln!("Error: {e:?}");
        process::exit(1);
    };
}
