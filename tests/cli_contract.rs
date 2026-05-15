use std::process::{Command, Output};

const REQUIRED_HELP_LINES: &[&str] = &[
    "db - deterministic single-process V1 database CLI",
    "Usage:",
    "  db --help",
    "  db help",
    "Supported commands:",
    "  help        Print this help text.",
    "Reserved future commands:",
    "  open <path>",
    "  exec <path> <sql>",
    "  check <path>",
    "  bench <path>",
    "V1 bootstrap scope:",
    "  This build only defines the CLI contract and smoke baseline.",
    "  Storage pages, SQL execution, indexes, transactions, WAL, and recovery are not implemented in this slice.",
    "Non-goals:",
    "  No network server, multi-process concurrency, or distributed storage.",
];

fn db(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_db"))
        .args(args)
        .output()
        .expect("db binary should run")
}

fn stdout(output: &Output) -> String {
    String::from_utf8(output.stdout.clone()).expect("stdout should be UTF-8")
}

fn stderr(output: &Output) -> String {
    String::from_utf8(output.stderr.clone()).expect("stderr should be UTF-8")
}

fn assert_help_contract(output: &Output) {
    assert!(
        output.status.success(),
        "expected exit 0, got {:?}; stderr={:?}",
        output.status.code(),
        stderr(output)
    );
    assert_eq!("", stderr(output), "help stderr must be empty");

    let out = stdout(output);
    let mut search_from = 0usize;
    for line in REQUIRED_HELP_LINES {
        let relative = out[search_from..].find(line).unwrap_or_else(|| {
            panic!("missing help line after byte {search_from}: {line:?}\nstdout:\n{out}")
        });
        search_from += relative + line.len();
    }
}

#[test]
fn help_flag_prints_required_contract() {
    let output = db(&["--help"]);

    assert_help_contract(&output);
}

#[test]
fn help_subcommand_matches_help_flag() {
    let help_flag = db(&["--help"]);
    let help_subcommand = db(&["help"]);

    assert_help_contract(&help_flag);
    assert_help_contract(&help_subcommand);
    assert_eq!(stdout(&help_flag), stdout(&help_subcommand));
}

#[test]
fn unsupported_argument_reports_first_token() {
    let output = db(&["--unknown"]);

    assert_eq!(Some(2), output.status.code());
    assert_eq!("", stdout(&output), "unsupported stdout must be empty");
    assert_eq!(
        "error: unsupported argument or command: --unknown\nhint: run 'db --help' for the supported V1 CLI contract.\n",
        stderr(&output)
    );
}

#[test]
fn reserved_future_command_remains_unsupported() {
    let output = db(&["open", "demo.db"]);

    assert_eq!(Some(2), output.status.code());
    assert_eq!("", stdout(&output), "unsupported stdout must be empty");
    assert_eq!(
        "error: unsupported argument or command: open\nhint: run 'db --help' for the supported V1 CLI contract.\n",
        stderr(&output)
    );
}
