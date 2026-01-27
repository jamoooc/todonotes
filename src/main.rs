use std::env;
use std::process;
use getopts::Options;

use todo_notes::cmd;

fn print_usage(program: &str, opts: &Options) {
    let brief = format!("\nUsage: ./{} [options]", program);
    println!("{}", opts.usage(&brief));
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();

    let mut opts = Options::new();
    opts.optopt("a", "add", "Add \"item\"", "");
    opts.optopt("d", "delete", "Delete list item n, or items \"n n n\"", "");
    opts.optopt("t", "todo", "Use another list", "./todo_notes -t default"); // TODO: -l list

    opts.optflag("l", "list", "List all items"); // TODO: p print
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

    let command = match cmd::Command::get_command(matches) {
        Ok(cmd) => cmd,
        Err(e) => {
            eprintln!("Error: {:?}", e);
            process::exit(1);
        }
    };

    if let Err(e) = cmd::Command::run(command) {
        eprintln!("Error: {e:?}");
        process::exit(1);
    };
}
