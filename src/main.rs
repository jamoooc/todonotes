use std::env;
use std::process;
use getopts::Options;
use env_logger::{Builder};
use log::{debug,LevelFilter};
use std::str::FromStr;

use todo_notes::cmd;
use todo_notes::{FLAG_CREATE,FLAG_DELETE,FLAG_SWITCH,FLAG_RESET,FLAG_PRINT,FLAG_LEVEL,FLAG_HELP};

fn print_usage(program: &str, opts: &Options) {
    let brief = format!("\nUsage: {} [options]", program);
    println!("{}", opts.usage(&brief));
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();

    let mut opts = Options::new();
    opts.optflag(FLAG_PRINT, "print", "Print all items");
    opts.optflag(FLAG_RESET, "reset", "Reset list state");
    opts.optopt(FLAG_CREATE, "create", "Create \"item\"", "-c \"item to create\"");
    opts.optmulti(FLAG_DELETE, "delete", "Delete list item n", "-d 5");

    opts.optopt(FLAG_SWITCH, "switch", "Switch to another list", "./todo_notes -s default"); // use progname?
    opts.optopt(FLAG_LEVEL, "level", "Set the log level", "INFO");
    opts.optflag(FLAG_HELP, "help", "Display usage info");

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
