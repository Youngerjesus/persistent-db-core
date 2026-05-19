use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::thread;
use std::time::Duration;

const TEST_LOCK_DIR: &str = "target/bench_acceptance/.section14.test.lock";

const REQUIRED_HELP_LINES: &[&str] = &[
    "db - deterministic single-process V1 database CLI",
    "Usage:",
    "  db --help",
    "  db help",
    "  db exec <path> <sql>",
    "  db check <path>",
    "  db bench",
    "Supported commands:",
    "  help        Print this help text.",
    "  exec <path> <sql>",
    "  check <path>",
    "  bench       Run the fixed Section 14 benchmark acceptance workload.",
    "Reserved future commands:",
    "  open <path>",
    "V1 scope:",
    "  This build supports the CLI contract, page storage, and the documented minimal SQL subset.",
    "Non-goals:",
    "  No network server, multi-process concurrency, or distributed storage.",
];

fn db(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_db"))
        .args(args)
        .output()
        .expect("db binary should run")
}

struct ProcessTestLock {
    path: PathBuf,
}

impl Drop for ProcessTestLock {
    fn drop(&mut self) {
        let _ = fs::remove_dir(&self.path);
    }
}

fn repo_path(relative: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(relative)
}

fn acquire_benchmark_test_lock() -> ProcessTestLock {
    let path = repo_path(TEST_LOCK_DIR);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("benchmark test lock parent should be created");
    }
    for _ in 0..3600 {
        match fs::create_dir(&path) {
            Ok(()) => return ProcessTestLock { path },
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                thread::sleep(Duration::from_millis(100));
            }
            Err(error) => panic!("benchmark test lock failed: {error}"),
        }
    }
    panic!("timed out acquiring benchmark test lock");
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

#[test]
fn bench_rejects_extra_path_argument() {
    let output = db(&["bench", "demo.db"]);

    assert_eq!(Some(2), output.status.code());
    assert_eq!("", stdout(&output), "unsupported stdout must be empty");
    assert_eq!(
        "error: unsupported argument or command: bench\nhint: run 'db --help' for the supported V1 CLI contract.\n",
        stderr(&output)
    );
}

#[test]
fn bench_public_command_writes_section14_evidence_contract() {
    let _guard = acquire_benchmark_test_lock();
    let output = db(&["bench"]);

    assert_eq!(
        Some(0),
        output.status.code(),
        "db bench should pass; stdout={:?}; stderr={:?}",
        stdout(&output),
        stderr(&output)
    );
    assert_eq!(
        "",
        stderr(&output),
        "db bench stderr must be empty on success"
    );
    assert_eq!(
        "DB_BENCH: PASS evidence=target/bench_acceptance/section14-benchmark-acceptance.json\n",
        stdout(&output)
    );

    let evidence_path = repo_path("target/bench_acceptance/section14-benchmark-acceptance.json");
    assert!(
        evidence_path.exists(),
        "db bench must create {}",
        evidence_path.display()
    );
}

#[test]
fn exec_requires_path_and_single_sql_argument() {
    let output = db(&["exec", "demo.db"]);

    assert_eq!(Some(2), output.status.code());
    assert_eq!("", stdout(&output), "unsupported stdout must be empty");
    assert_eq!(
        "error: unsupported argument or command: exec\nhint: run 'db --help' for the supported V1 CLI contract.\n",
        stderr(&output)
    );
}

#[test]
fn check_requires_path_argument() {
    let output = db(&["check"]);

    assert_eq!(Some(2), output.status.code());
    assert_eq!("", stdout(&output), "unsupported stdout must be empty");
    assert_eq!(
        "error: unsupported argument or command: check\nhint: run 'db --help' for the supported V1 CLI contract.\n",
        stderr(&output)
    );
}
