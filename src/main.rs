use std::env;
use std::process;

use todo_notes::cmd;
use todo_notes::config;

fn main() {
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();
    let command = match config::parse_args(&program, &args) {
        Ok(cmd) => cmd,
        Err(e) => {
            eprintln!("Error: {e}");
            process::exit(1);
        }
    };

    if let Err(e) = cmd::Command::run(command) {
        eprintln!("Application command error: {e:?}");
        process::exit(1);
    };
}
