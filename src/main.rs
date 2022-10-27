use std::env;
use std::process;
use todo_notes::Command;

fn main() {
  let args: Vec<String> = env::args().collect();
  let program = args[0].clone();
  let command = match Command::parse_args(&program, &args) {
    Ok(cmd) => cmd,
    Err(e) => {
      eprintln!("Error: {e}");
      process::exit(1);
    }
  };

  Command::run(command);
}
