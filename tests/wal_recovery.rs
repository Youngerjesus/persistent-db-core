use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const SQL_RECORD_PREFIX: &[u8; 8] = b"PDBSQL1\0";
const WAL_MAGIC: &[u8; 8] = b"PDBWAL1\0";
const WAL_VERSION: u16 = 1;
const WAL_STATE_COMMITTED: u8 = 0x01;
const WAL_STATE_ROLLED_BACK: u8 = 0x02;
const WAL_PAYLOAD_KIND_PAGE_APPEND: u8 = 0x01;
const WAL_HEADER_LEN: usize = 36;

fn db(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_db"))
        .args(args)
        .output()
        .expect("db binary should run")
}

fn temp_db_path(test_name: &str) -> PathBuf {
    let mut dir = std::env::temp_dir();
    dir.push(format!(
        "persistent_db_core_wal_recovery_{}_{}_{}",
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

fn checksum(frame_with_zero_checksum: &[u8]) -> u32 {
    frame_with_zero_checksum
        .iter()
        .enumerate()
        .filter(|(index, _)| !(32..36).contains(index))
        .fold(0u32, |sum, (_, byte)| sum.wrapping_add(*byte as u32))
}

#[test]
fn committed_wal_replay_survives_reopen_via_cli() {
    let path = temp_db_path("committed_wal_replay_survives_reopen_via_cli");

    assert_exec(
        &path,
        "CREATE TABLE users (id INT, name TEXT); INSERT INTO users VALUES (1, 'ada'); INSERT INTO users VALUES (2, 'bea');",
        0,
        "",
        "",
    );

    let sidecar = wal_path(&path);
    assert!(
        sidecar.exists(),
        "committed mutations must create retained WAL sidecar at {:?}",
        sidecar
    );

    assert_exec(
        &path,
        "SELECT * FROM users;",
        0,
        "id|name\n1|ada\n2|bea\n",
        "",
    );

    cleanup(&path);
}

#[test]
fn rolled_back_wal_frame_is_not_replayed_as_uncommitted_change() {
    let path = temp_db_path("rolled_back_wal_frame_is_not_replayed_as_uncommitted_change");

    assert_exec(&path, "CREATE TABLE users (id INT, name TEXT);", 0, "", "");

    let committed = committed_wal_frame(1, 1, &row_record("users", &[(b'I', "1"), (b'T', "ada")]));
    let rolled_back_ghost =
        rolled_back_wal_frame(2, 2, &row_record("users", &[(b'I', "9"), (b'T', "ghost")]));

    let sidecar = wal_path(&path);
    let mut wal_bytes = committed;
    wal_bytes.extend_from_slice(&rolled_back_ghost);
    fs::write(&sidecar, wal_bytes).expect("WAL fixture should be written");

    assert_exec(&path, "SELECT * FROM users;", 0, "id|name\n1|ada\n", "");
    assert_exec(&path, "SELECT * FROM users;", 0, "id|name\n1|ada\n", "");

    cleanup(&path);
}

#[test]
fn incomplete_wal_entry_is_not_replayed_without_public_rollback_cli() {
    let path = temp_db_path("incomplete_wal_entry_is_not_replayed_without_public_rollback_cli");

    assert_exec(&path, "CREATE TABLE users (id INT, name TEXT);", 0, "", "");

    let committed = committed_wal_frame(1, 1, &row_record("users", &[(b'I', "1"), (b'T', "ada")]));
    let mut incomplete_ghost =
        committed_wal_frame(2, 2, &row_record("users", &[(b'I', "9"), (b'T', "ghost")]));
    incomplete_ghost.truncate(WAL_HEADER_LEN + 4);

    let sidecar = wal_path(&path);
    let mut wal_bytes = committed;
    wal_bytes.extend_from_slice(&incomplete_ghost);
    fs::write(&sidecar, wal_bytes).expect("WAL fixture should be written");

    // V1 intentionally has no public rollback or incomplete transaction command,
    // so this fixture authors WAL bytes directly and verifies replay through CLI.
    assert_exec(&path, "SELECT * FROM users;", 0, "id|name\n1|ada\n", "");
    assert_exec(&path, "SELECT * FROM users;", 0, "id|name\n1|ada\n", "");

    let wal_len = fs::metadata(&sidecar)
        .expect("retained WAL sidecar should exist")
        .len();
    assert!(
        wal_len > WAL_HEADER_LEN as u64,
        "WAL sidecar should retain committed bytes after incomplete-tail cleanup"
    );

    cleanup(&path);
}

#[test]
fn committed_frame_after_incomplete_tail_cleanup_remains_replayable() {
    let path = temp_db_path("committed_frame_after_incomplete_tail_cleanup_remains_replayable");

    assert_exec(&path, "CREATE TABLE users (id INT, name TEXT);", 0, "", "");

    let committed_ada =
        committed_wal_frame(1, 1, &row_record("users", &[(b'I', "1"), (b'T', "ada")]));
    let mut incomplete_ghost =
        committed_wal_frame(2, 2, &row_record("users", &[(b'I', "9"), (b'T', "ghost")]));
    incomplete_ghost.truncate(WAL_HEADER_LEN + 4);

    let sidecar = wal_path(&path);
    let committed_ada_len = committed_ada.len() as u64;
    let mut wal_bytes = committed_ada;
    wal_bytes.extend_from_slice(&incomplete_ghost);
    fs::write(&sidecar, wal_bytes).expect("WAL fixture should be written");

    assert_exec(&path, "SELECT * FROM users;", 0, "id|name\n1|ada\n", "");
    assert_eq!(
        committed_ada_len,
        fs::metadata(&sidecar)
            .expect("WAL sidecar should remain readable after cleanup")
            .len(),
        "incomplete tail bytes should be removed before future WAL appends"
    );

    let committed_bea =
        committed_wal_frame(2, 2, &row_record("users", &[(b'I', "2"), (b'T', "bea")]));
    OpenOptions::new()
        .append(true)
        .open(&sidecar)
        .expect("WAL sidecar should open for post-cleanup frame")
        .write_all(&committed_bea)
        .expect("post-cleanup committed frame should be written");

    assert_exec(
        &path,
        "SELECT * FROM users;",
        0,
        "id|name\n1|ada\n2|bea\n",
        "",
    );

    cleanup(&path);
}

#[test]
fn committed_wal_frame_ahead_of_page_store_fails_deterministically() {
    let path = temp_db_path("committed_wal_frame_ahead_of_page_store_fails_deterministically");

    assert_exec(&path, "CREATE TABLE users (id INT, name TEXT);", 0, "", "");

    let sidecar = wal_path(&path);
    fs::write(
        &sidecar,
        committed_wal_frame(1, 2, &row_record("users", &[(b'I', "9"), (b'T', "ghost")])),
    )
    .expect("ahead-of-store WAL fixture should be written");

    let output = exec_sql(&path, "SELECT * FROM users;");
    assert_eq!(
        Some(1),
        output.status.code(),
        "ahead-of-store WAL should fail open; stdout={:?}; stderr={:?}",
        stdout(&output),
        stderr(&output)
    );
    assert_eq!("", stdout(&output));
    assert_eq!(
        "error: storage error: CorruptRecordLength\nhint: database file must use the documented V1 page format.\n",
        stderr(&output)
    );

    cleanup(&path);
}
