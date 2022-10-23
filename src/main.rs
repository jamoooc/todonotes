use std::env;
use todo_notes::Command;

fn main() {
  let args: Vec<String> = env::args().collect();
  let program = args[0].clone();
  let command = Command::parse_args(&program, &args);

  Command::run(command);
}
