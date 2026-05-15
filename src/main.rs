use std::env;
use std::process;

const HELP: &str = "\
db - deterministic single-process V1 database CLI
Usage:
  db --help
  db help
Supported commands:
  help        Print this help text.
Reserved future commands:
  open <path>
  exec <path> <sql>
  check <path>
  bench <path>
V1 bootstrap scope:
  This build only defines the CLI contract and smoke baseline.
  Storage pages, SQL execution, indexes, transactions, WAL, and recovery are not implemented in this slice.
Non-goals:
  No network server, multi-process concurrency, or distributed storage.
";

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    match args.as_slice() {
        [arg] if arg == "--help" || arg == "help" => {
            print!("{HELP}");
        }
        [token, ..] => {
            eprintln!("error: unsupported argument or command: {token}");
            eprintln!("hint: run 'db --help' for the supported V1 CLI contract.");
            process::exit(2);
        }
        [] => {
            eprintln!("error: unsupported argument or command: <none>");
            eprintln!("hint: run 'db --help' for the supported V1 CLI contract.");
            process::exit(2);
        }
    }
}
