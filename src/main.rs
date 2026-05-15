use std::env;
use std::process;

const HELP: &str = "\
db 0.1.0

Usage:
  db --help
  db help

Options:
  -h, --help    Print this help message.

V1 persistent-db-core is currently bootstrapped as a CLI skeleton. Future gaps will add page storage, SQL execution, indexes, transactions, WAL recovery, crash testing, differential tests, invariant checks, and benchmarks.
";

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    if args.is_empty() || args.iter().any(|arg| arg == "-h" || arg == "--help") || args == ["help"] {
        print!("{HELP}");
        return;
    }

    eprintln!("db: unsupported arguments: {}", args.join(" "));
    eprintln!("Run `db --help` for usage.");
    process::exit(2);
}
