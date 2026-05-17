use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::{Mutex, Once};

const SQL_RECORD_PREFIX: &[u8; 8] = b"PDBSQL1\0";
const WAL_MAGIC: &[u8; 8] = b"PDBWAL1\0";
const WAL_VERSION: u16 = 1;
const WAL_STATE_COMMITTED: u8 = 0x01;
const WAL_STATE_ROLLED_BACK: u8 = 0x02;
const WAL_PAYLOAD_KIND_PAGE_APPEND: u8 = 0x01;
const WAL_HEADER_LEN: usize = 36;
const REPORT_DIR: &str = "target/crash_matrix";

static REPORT_INIT: Once = Once::new();
static REPORT_LOCK: Mutex<()> = Mutex::new(());

#[derive(Clone, Copy)]
struct CrashCase {
    case_id: &'static str,
    crash_point: &'static str,
    evidence_id: &'static str,
    test_name: &'static str,
    expected_rows: &'static str,
    wal_assertion: &'static str,
}

const CM_001: CrashCase = CrashCase {
    case_id: "CM-001",
    crash_point: "pre-wal-append",
    evidence_id: "crash-matrix-case-CM-001",
    test_name: "cm_001_pre_wal_append_seed_only_visible",
    expected_rows: "id|name\n1|seed\n",
    wal_assertion: "WAL sidecar absent or empty; file header/version and seed data unchanged",
};

const CM_002: CrashCase = CrashCase {
    case_id: "CM-002",
    crash_point: "partial-wal-frame",
    evidence_id: "crash-matrix-case-CM-002",
    test_name: "cm_002_partial_wal_frame_is_ignored",
    expected_rows: "id|name\n1|seed\n",
    wal_assertion: "incomplete WAL header or payload tail is ignored/truncated without panic",
};

const CM_003: CrashCase = CrashCase {
    case_id: "CM-003",
    crash_point: "wal-frame-without-commit-marker",
    evidence_id: "crash-matrix-case-CM-003",
    test_name: "cm_003_wal_frame_without_commit_marker_is_not_visible",
    expected_rows: "id|name\n1|seed\n",
    wal_assertion: "commit marker absent maps to WAL_STATE_ROLLED_BACK 0x02 and is not replayed",
};

const CM_004: CrashCase = CrashCase {
    case_id: "CM-004",
    crash_point: "committed-wal-before-data-apply",
    evidence_id: "crash-matrix-case-CM-004",
    test_name: "cm_004_committed_wal_before_data_apply_is_idempotent",
    expected_rows: "id|name\n1|seed\n2|committed_wal\n",
    wal_assertion: "committed WAL replay is idempotent across first and second reopen",
};

const CM_005: CrashCase = CrashCase {
    case_id: "CM-005",
    crash_point: "recovery-interrupted-after-first-apply",
    evidence_id: "crash-matrix-case-CM-005",
    test_name: "cm_005_recovery_interrupted_after_first_apply_replays_remaining_once",
    expected_rows: "id|name\n1|seed\n2|recover_a\n3|recover_b\n",
    wal_assertion: "interrupted recovery re-entry applies every committed frame exactly once",
};

const CM_006: CrashCase = CrashCase {
    case_id: "CM-006",
    crash_point: "corrupt-tail-after-committed-frame",
    evidence_id: "crash-matrix-case-CM-006",
    test_name: "cm_006_corrupt_tail_after_committed_frame_preserves_committed_prefix",
    expected_rows: "id|name\n1|seed\n2|committed_before_tail\n",
    wal_assertion: "committed WAL prefix is replayed and incomplete/invalid-length tail is ignored without CLI output change",
};

fn db(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_db"))
        .args(args)
        .output()
        .expect("db binary should run")
}

fn db_with_env(args: &[&str], envs: &[(&str, &str)]) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_db"));
    command.args(args);
    for (key, value) in envs {
        command.env(key, value);
    }
    command.output().expect("db binary should run")
}

fn temp_db_path(test_name: &str) -> PathBuf {
    let mut dir = std::env::temp_dir();
    dir.push(format!(
        "persistent_db_core_crash_matrix_{}_{}_{}",
        test_name,
        std::process::id(),
        unique_suffix()
    ));
    fs::create_dir_all(&dir).expect("temp dir should be created");
    dir.push("test.pdb");
    dir
}

fn unique_suffix() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system clock should be after unix epoch")
        .as_nanos()
}

fn cleanup(path: &Path) {
    if let Some(parent) = path.parent() {
        let _ = fs::remove_dir_all(parent);
    }
}

fn stdout(output: &Output) -> String {
    String::from_utf8(output.stdout.clone()).expect("stdout should be UTF-8")
}

fn stderr(output: &Output) -> String {
    String::from_utf8(output.stderr.clone()).expect("stderr should be UTF-8")
}

fn exec_sql(path: &Path, sql: &str) -> Output {
    db(&[
        "exec",
        path.to_str().expect("temp path should be UTF-8"),
        sql,
    ])
}

fn exec_sql_with_env(path: &Path, sql: &str, envs: &[(&str, &str)]) -> Output {
    db_with_env(
        &[
            "exec",
            path.to_str().expect("temp path should be UTF-8"),
            sql,
        ],
        envs,
    )
}

fn assert_exec(path: &Path, sql: &str, code: i32, expected_stdout: &str, expected_stderr: &str) {
    let output = exec_sql(path, sql);
    assert_eq!(
        Some(code),
        output.status.code(),
        "unexpected exit; stdout={:?}; stderr={:?}",
        stdout(&output),
        stderr(&output)
    );
    assert_eq!(expected_stdout, stdout(&output));
    assert_eq!(expected_stderr, stderr(&output));
}

fn wal_path(path: &Path) -> PathBuf {
    PathBuf::from(format!(
        "{}.wal",
        path.to_str().expect("temp path should be UTF-8")
    ))
}

fn row_record(table: &str, values: &[(u8, &str)]) -> Vec<u8> {
    let mut record = Vec::new();
    record.extend_from_slice(SQL_RECORD_PREFIX);
    record.push(b'R');
    write_string_u16(&mut record, table);
    record.extend_from_slice(&(values.len() as u16).to_le_bytes());
    for (value_type, value) in values {
        record.push(*value_type);
        record.extend_from_slice(&(value.len() as u32).to_le_bytes());
        record.extend_from_slice(value.as_bytes());
    }
    record
}

fn write_string_u16(record: &mut Vec<u8>, value: &str) {
    record.extend_from_slice(&(value.len() as u16).to_le_bytes());
    record.extend_from_slice(value.as_bytes());
}

fn wal_frame(
    frame_id: u64,
    record_count_before: u64,
    state: u8,
    payload_kind: u8,
    payload: &[u8],
) -> Vec<u8> {
    let mut frame = Vec::with_capacity(WAL_HEADER_LEN + payload.len());
    frame.extend_from_slice(WAL_MAGIC);
    frame.extend_from_slice(&WAL_VERSION.to_le_bytes());
    frame.extend_from_slice(&frame_id.to_le_bytes());
    frame.extend_from_slice(&record_count_before.to_le_bytes());
    frame.push(state);
    frame.push(payload_kind);
    frame.extend_from_slice(&(payload.len() as u32).to_le_bytes());
    frame.extend_from_slice(&0u32.to_le_bytes());
    frame.extend_from_slice(payload);

    let checksum = checksum(&frame);
    frame[32..36].copy_from_slice(&checksum.to_le_bytes());
    frame
}

fn committed_wal_frame(frame_id: u64, record_count_before: u64, payload: &[u8]) -> Vec<u8> {
    wal_frame(
        frame_id,
        record_count_before,
        WAL_STATE_COMMITTED,
        WAL_PAYLOAD_KIND_PAGE_APPEND,
        payload,
    )
}

fn rolled_back_wal_frame(frame_id: u64, record_count_before: u64, payload: &[u8]) -> Vec<u8> {
    wal_frame(
        frame_id,
        record_count_before,
        WAL_STATE_ROLLED_BACK,
        WAL_PAYLOAD_KIND_PAGE_APPEND,
        payload,
    )
}

fn checksum(frame_with_zero_checksum: &[u8]) -> u32 {
    frame_with_zero_checksum
        .iter()
        .enumerate()
        .filter(|(index, _)| !(32..36).contains(index))
        .fold(0u32, |sum, (_, byte)| sum.wrapping_add(*byte as u32))
}

fn seed_committed_one(path: &Path) {
    assert_exec(
        path,
        "CREATE TABLE items (id INT PRIMARY KEY, name TEXT); INSERT INTO items VALUES (1, 'seed');",
        0,
        "",
        "",
    );
}

fn select_items(path: &Path) -> Output {
    exec_sql(path, "SELECT * FROM items;")
}

fn row_payload(id: u64, name: &str) -> Vec<u8> {
    row_record("items", &[(b'I', &id.to_string()), (b'T', name)])
}

fn assert_select_case(case: CrashCase, output: &Output) {
    assert_eq!(
        Some(0),
        output.status.code(),
        "{} {} reopen command failed; stdout={:?}; stderr={:?}",
        case.case_id,
        case.crash_point,
        stdout(output),
        stderr(output)
    );
    assert_eq!(
        case.expected_rows,
        stdout(output),
        "{} {} actual visible rows differed",
        case.case_id,
        case.crash_point
    );
    assert_eq!(
        "",
        stderr(output),
        "{} {} changed user-facing CLI stderr",
        case.case_id,
        case.crash_point
    );
}

fn run_id() -> String {
    std::env::var("CRASH_MATRIX_RUN_ID")
        .unwrap_or_else(|_| format!("cargo-test-pid-{}", std::process::id()))
}

fn record_case(case: CrashCase, actual_rows: &str, exit_status: Option<i32>, details: &[String]) {
    let _guard = REPORT_LOCK
        .lock()
        .expect("report lock should not be poisoned");
    let report_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(REPORT_DIR);
    let run_id = run_id();
    REPORT_INIT.call_once(|| {
        let _ = fs::remove_dir_all(&report_dir);
        fs::create_dir_all(report_dir.join("cases")).expect("crash matrix report dir should exist");
    });

    let case_path = report_dir
        .join("cases")
        .join(format!("{}.md", case.case_id));
    fs::write(
        &case_path,
        case_report(case, actual_rows, exit_status, details),
    )
    .expect("case report should be written");

    let mut report = String::from("# Deterministic Crash Matrix Report\n\n");
    report.push_str(&format!(
        "- run id: {run_id}\n- command: cargo test --test crash_matrix\n\n"
    ));
    for matrix_case in [CM_001, CM_002, CM_003, CM_004, CM_005, CM_006] {
        let path = report_dir
            .join("cases")
            .join(format!("{}.md", matrix_case.case_id));
        if path.exists() {
            report.push_str(
                &fs::read_to_string(path).expect("case report should be readable for assembly"),
            );
            report.push('\n');
        }
    }
    fs::write(report_dir.join("crash_matrix_report.md"), report)
        .expect("crash matrix report should be written");
}

fn case_report(
    case: CrashCase,
    actual_rows: &str,
    exit_status: Option<i32>,
    details: &[String],
) -> String {
    let mut report = format!(
        "## {case_id} {crash_point}\n\
         - run id: {run_id}\n\
         - required evidence id: {evidence_id}\n\
         - reopen command: db exec <db_path> \"SELECT * FROM items;\"\n\
         - expected visible rows:\n\n```text\n{expected_rows}```\n\
         - actual visible rows:\n\n```text\n{actual_rows}```\n\
         - WAL/file-format assertion result: PASS - {wal_assertion}\n\
         - exit status: {exit_status:?}\n",
        case_id = case.case_id,
        crash_point = case.crash_point,
        run_id = run_id(),
        evidence_id = case.evidence_id,
        expected_rows = case.expected_rows,
        actual_rows = actual_rows,
        wal_assertion = case.wal_assertion,
        exit_status = exit_status
    );
    for detail in details {
        report.push_str("- detail: ");
        report.push_str(detail);
        report.push('\n');
    }
    report
}

#[test]
fn cm_001_pre_wal_append_seed_only_visible() {
    let path = temp_db_path(CM_001.test_name);
    seed_committed_one(&path);
    let before = fs::read(&path).expect("seed database should be readable");
    let sidecar = wal_path(&path);
    let _ = fs::remove_file(&sidecar);

    let output = select_items(&path);
    assert_select_case(CM_001, &output);
    assert!(
        !sidecar.exists()
            || fs::metadata(&sidecar)
                .expect("WAL sidecar metadata should be readable")
                .len()
                == 0,
        "{} {} WAL sidecar should remain absent or empty",
        CM_001.case_id,
        CM_001.crash_point
    );
    assert_eq!(
        before,
        fs::read(&path).expect("database should be readable after reopen"),
        "{} {} file header/version and data file changed",
        CM_001.case_id,
        CM_001.crash_point
    );

    record_case(CM_001, &stdout(&output), output.status.code(), &[]);
    cleanup(&path);
}

#[test]
fn cm_002_partial_wal_frame_is_ignored() {
    let path = temp_db_path(CM_002.test_name);
    seed_committed_one(&path);
    let mut partial = committed_wal_frame(1, 2, &row_payload(2, "partial_wal"));
    partial.truncate(WAL_HEADER_LEN + 4);
    fs::write(wal_path(&path), partial).expect("partial WAL frame should be written");

    let output = select_items(&path);
    assert_select_case(CM_002, &output);
    assert_eq!(
        0,
        fs::metadata(wal_path(&path))
            .expect("WAL sidecar should remain after truncation")
            .len(),
        "{} {} incomplete tail should be truncated away",
        CM_002.case_id,
        CM_002.crash_point
    );

    record_case(CM_002, &stdout(&output), output.status.code(), &[]);
    cleanup(&path);
}

#[test]
fn cm_003_wal_frame_without_commit_marker_is_not_visible() {
    let path = temp_db_path(CM_003.test_name);
    seed_committed_one(&path);
    fs::write(
        wal_path(&path),
        rolled_back_wal_frame(1, 2, &row_payload(2, "uncommitted")),
    )
    .expect("rolled-back WAL frame should be written");

    let output = select_items(&path);
    assert_select_case(CM_003, &output);

    record_case(CM_003, &stdout(&output), output.status.code(), &[]);
    cleanup(&path);
}

#[test]
fn cm_004_committed_wal_before_data_apply_is_idempotent() {
    let path = temp_db_path(CM_004.test_name);
    seed_committed_one(&path);
    fs::write(
        wal_path(&path),
        committed_wal_frame(1, 2, &row_payload(2, "committed_wal")),
    )
    .expect("committed WAL frame should be written");

    let first = select_items(&path);
    assert_select_case(CM_004, &first);
    let second = select_items(&path);
    assert_select_case(CM_004, &second);

    record_case(
        CM_004,
        &stdout(&second),
        second.status.code(),
        &[
            format!("first reopen exit status: {:?}", first.status.code()),
            format!("first reopen actual visible rows: {:?}", stdout(&first)),
            format!("second reopen exit status: {:?}", second.status.code()),
            format!("second reopen actual visible rows: {:?}", stdout(&second)),
        ],
    );
    cleanup(&path);
}

#[test]
fn cm_005_recovery_interrupted_after_first_apply_replays_remaining_once() {
    let path = temp_db_path(CM_005.test_name);
    seed_committed_one(&path);

    let mut wal_bytes = committed_wal_frame(1, 2, &row_payload(2, "recover_a"));
    wal_bytes.extend_from_slice(&committed_wal_frame(2, 3, &row_payload(3, "recover_b")));
    fs::write(wal_path(&path), wal_bytes)
        .expect("interrupted recovery WAL fixture should be written");

    let interrupted = exec_sql_with_env(
        &path,
        "SELECT * FROM items;",
        &[("PDB_CRASH_AFTER_WAL_REPLAY_APPLIES", "1")],
    );
    assert_eq!(
        Some(101),
        interrupted.status.code(),
        "{} {} crash injection should interrupt after first replay apply; stdout={:?}; stderr={:?}",
        CM_005.case_id,
        CM_005.crash_point,
        stdout(&interrupted),
        stderr(&interrupted)
    );

    let first = select_items(&path);
    assert_select_case(CM_005, &first);
    let second = select_items(&path);
    assert_select_case(CM_005, &second);

    record_case(
        CM_005,
        &stdout(&second),
        second.status.code(),
        &[
            "durable setup before crash injection: seed_committed_one only".to_string(),
            format!(
                "interrupted reopen exit status after first replay apply: {:?}",
                interrupted.status.code()
            ),
            format!(
                "first recovery reopen exit status: {:?}",
                first.status.code()
            ),
            format!("first recovery actual visible rows: {:?}", stdout(&first)),
            format!(
                "second recovery reopen exit status: {:?}",
                second.status.code()
            ),
            format!("second recovery actual visible rows: {:?}", stdout(&second)),
        ],
    );
    cleanup(&path);
}

#[test]
fn cm_006_corrupt_tail_after_committed_frame_preserves_committed_prefix() {
    let path = temp_db_path(CM_006.test_name);
    seed_committed_one(&path);

    let committed = committed_wal_frame(1, 2, &row_payload(2, "committed_before_tail"));
    let committed_len = committed.len() as u64;
    let mut corrupt_tail = committed_wal_frame(2, 3, &row_payload(3, "tail_garbage"));
    corrupt_tail.truncate(WAL_HEADER_LEN + 2);
    let mut wal_bytes = committed;
    wal_bytes.extend_from_slice(&corrupt_tail);
    fs::write(wal_path(&path), wal_bytes).expect("corrupt-tail WAL fixture should be written");

    let output = select_items(&path);
    assert_select_case(CM_006, &output);
    assert_eq!(
        committed_len,
        fs::metadata(wal_path(&path))
            .expect("WAL sidecar should remain after corrupt-tail cleanup")
            .len(),
        "{} {} corrupt tail should be truncated after committed prefix",
        CM_006.case_id,
        CM_006.crash_point
    );

    record_case(CM_006, &stdout(&output), output.status.code(), &[]);
    cleanup(&path);
}

#[test]
fn crash_matrix_contract_lists_all_cases_and_evidence_ids() {
    let cases = [CM_001, CM_002, CM_003, CM_004, CM_005, CM_006];
    assert_eq!(6, cases.len());
    for case in cases {
        assert!(
            case.evidence_id.starts_with("crash-matrix-case-CM-"),
            "{} {} has invalid evidence id {}",
            case.case_id,
            case.crash_point,
            case.evidence_id
        );
        assert!(
            case.expected_rows.starts_with("id|name\n"),
            "{} {} expected rows must include deterministic SELECT header",
            case.case_id,
            case.crash_point
        );
    }

    let _ = (
        SQL_RECORD_PREFIX,
        WAL_MAGIC,
        WAL_VERSION,
        WAL_STATE_COMMITTED,
        WAL_PAYLOAD_KIND_PAGE_APPEND,
        WAL_HEADER_LEN,
        db as fn(&[&str]) -> Output,
        db_with_env as fn(&[&str], &[(&str, &str)]) -> Output,
        stdout as fn(&Output) -> String,
        stderr as fn(&Output) -> String,
        exec_sql as fn(&Path, &str) -> Output,
        exec_sql_with_env as fn(&Path, &str, &[(&str, &str)]) -> Output,
        wal_path as fn(&Path) -> PathBuf,
        row_record as fn(&str, &[(u8, &str)]) -> Vec<u8>,
        wal_frame as fn(u64, u64, u8, u8, &[u8]) -> Vec<u8>,
    );
}
