use std::env;
use std::process;

use todo_notes::config::config;
use todo_notes::cmd::cmd;

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
