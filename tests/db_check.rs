use persistent_db_core::storage::PageStore;
use std::fs::{self, OpenOptions};
use std::io::{Seek, SeekFrom, Write};
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const PAGE_SIZE: usize = 4096;
const DATA_PAGE_HEADER_SIZE: u64 = 16;
const RECORD_LENGTH_SIZE: usize = 4;
const MAX_RECORD_PAYLOAD_LEN: usize =
    PAGE_SIZE - DATA_PAGE_HEADER_SIZE as usize - RECORD_LENGTH_SIZE;
const FIRST_RECORD_LENGTH_OFFSET: u64 = PAGE_SIZE as u64 + DATA_PAGE_HEADER_SIZE;
const FIRST_RECORD_PAYLOAD_OFFSET: u64 = FIRST_RECORD_LENGTH_OFFSET + 4;
const FIRST_SQL_KIND_OFFSET: u64 = FIRST_RECORD_PAYLOAD_OFFSET + SQL_RECORD_PREFIX.len() as u64;
const SQL_RECORD_PREFIX: &[u8; 8] = b"PDBSQL1\0";
const WAL_MAGIC: &[u8; 8] = b"PDBWAL1\0";
const WAL_VERSION: u16 = 1;
const WAL_STATE_COMMITTED: u8 = 0x01;
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
        "persistent_db_core_db_check_{}_{}_{}",
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
    if path.is_dir() {
        let _ = fs::remove_dir_all(path);
        return;
    }
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

fn check_db(path: &Path) -> Output {
    db(&["check", path.to_str().expect("temp path should be UTF-8")])
}

fn assert_exec_ok(path: &Path, sql: &str) {
    let output = exec_sql(path, sql);
    assert_eq!(
        Some(0),
        output.status.code(),
        "exec failed; stdout={:?}; stderr={:?}",
        stdout(&output),
        stderr(&output)
    );
    assert_eq!("", stdout(&output));
    assert_eq!("", stderr(&output));
}

fn assert_check_ok(output: &Output) {
    assert_eq!(
        Some(0),
        output.status.code(),
        "valid db check should pass; stdout={:?}; stderr={:?}",
        stdout(output),
        stderr(output)
    );
    assert_eq!("ok: db check passed\n", stdout(output));
    assert_eq!("", stderr(output));
}

fn assert_check_failed(output: &Output, expected_label: &str) {
    assert_eq!(
        Some(1),
        output.status.code(),
        "db check should fail with exit 1; stdout={:?}; stderr={:?}",
        stdout(output),
        stderr(output)
    );
    assert_eq!("", stdout(output), "db check failure stdout must be empty");
    assert_eq!(
        format!("error: db check failed: {expected_label}\n"),
        stderr(output),
        "stderr should use exact check failure contract"
    );
}

fn assert_user_open_read_error(output: &Output, expected_path: &Path) {
    assert_eq!(
        Some(1),
        output.status.code(),
        "open/read error should fail with exit 1; stdout={:?}; stderr={:?}",
        stdout(output),
        stderr(output)
    );
    assert_eq!("", stdout(output), "open/read error stdout must be empty");
    assert_eq!(
        format!(
            "error: could not open or read database path: {}\n",
            expected_path.display()
        ),
        stderr(output),
        "stderr should match documented open/read failure shape"
    );
}

fn wal_path(path: &Path) -> PathBuf {
    PathBuf::from(format!(
        "{}.wal",
        path.to_str().expect("temp path should be UTF-8")
    ))
}

fn write_string_u16(record: &mut Vec<u8>, value: &str) {
    record.extend_from_slice(&(value.len() as u16).to_le_bytes());
    record.extend_from_slice(value.as_bytes());
}

fn catalog_record(table: &str, columns: &[(&str, u8, bool)]) -> Vec<u8> {
    let mut record = Vec::new();
    record.extend_from_slice(SQL_RECORD_PREFIX);
    record.push(b'C');
    write_string_u16(&mut record, table);
    record.extend_from_slice(&(columns.len() as u16).to_le_bytes());
    let mut primary_key = None;
    for (index, (name, value_type, is_primary_key)) in columns.iter().enumerate() {
        write_string_u16(&mut record, name);
        record.push(*value_type);
        if *is_primary_key {
            primary_key = Some(index as u16);
        }
    }
    if let Some(primary_key) = primary_key {
        record.push(b'P');
        record.extend_from_slice(&primary_key.to_le_bytes());
    }
    record
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

fn append_raw_record(path: &Path, payload: &[u8]) {
    let mut store = PageStore::open(path).expect("fixture page store should open");
    store
        .append_record(payload)
        .expect("fixture record should append");
}

fn committed_wal_frame(frame_id: u64, record_count_before: u64, payload: &[u8]) -> Vec<u8> {
    let mut frame = Vec::with_capacity(WAL_HEADER_LEN + payload.len());
    frame.extend_from_slice(WAL_MAGIC);
    frame.extend_from_slice(&WAL_VERSION.to_le_bytes());
    frame.extend_from_slice(&frame_id.to_le_bytes());
    frame.extend_from_slice(&record_count_before.to_le_bytes());
    frame.push(WAL_STATE_COMMITTED);
    frame.push(WAL_PAYLOAD_KIND_PAGE_APPEND);
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
fn check_valid_database_passes_with_exact_output() {
    let path = temp_db_path("check_valid_database_passes_with_exact_output");

    assert_exec_ok(
        &path,
        "CREATE TABLE users (id INT PRIMARY KEY, name TEXT); INSERT INTO users VALUES (1, 'ada');",
    );

    let output = check_db(&path);
    assert_check_ok(&output);

    cleanup(&path);
}

#[test]
fn check_valid_chained_retained_wal_frames_pass() {
    let path = temp_db_path("check_valid_chained_retained_wal_frames_pass");

    assert_exec_ok(&path, "CREATE TABLE users (id INT, name TEXT);");

    let first_row = row_record("users", &[(b'I', "1"), (b'T', "ada")]);
    let second_row = row_record("users", &[(b'I', "2"), (b'T', "bea")]);
    let durable_record_count = 1u64;
    fs::write(
        wal_path(&path),
        [
            committed_wal_frame(1, durable_record_count, &first_row),
            committed_wal_frame(2, durable_record_count + 1, &second_row),
        ]
        .concat(),
    )
    .expect("valid chained retained WAL fixture should be written");

    let output = check_db(&path);
    assert_check_ok(&output);

    cleanup(&path);
}

#[test]
fn check_storage_readability_corruption_fails_with_stable_prefix() {
    let path = temp_db_path("check_storage_readability_corruption_fails_with_stable_prefix");

    assert_exec_ok(&path, "CREATE TABLE users (id INT, name TEXT);");
    let mut file = OpenOptions::new()
        .write(true)
        .open(&path)
        .expect("fixture db should open for corruption");
    file.seek(SeekFrom::Start(FIRST_RECORD_LENGTH_OFFSET))
        .expect("seek should succeed");
    file.write_all(&(PAGE_SIZE as u32).to_le_bytes())
        .expect("corrupt record length should be written");

    let output = check_db(&path);
    assert_check_failed(&output, "storage record readability");

    cleanup(&path);
}

#[test]
fn check_decode_impossible_sql_bytes_fail_as_storage_readability() {
    let path = temp_db_path("check_decode_impossible_sql_bytes_fail_as_storage_readability");

    assert_exec_ok(&path, "CREATE TABLE users (id INT, name TEXT);");
    let mut file = OpenOptions::new()
        .write(true)
        .open(&path)
        .expect("fixture db should open for SQL byte corruption");
    file.seek(SeekFrom::Start(FIRST_SQL_KIND_OFFSET))
        .expect("seek should succeed");
    file.write_all(b"X")
        .expect("corrupt SQL record kind should be written");

    let output = check_db(&path);
    assert_check_failed(&output, "storage record readability");

    cleanup(&path);
}

#[test]
fn check_catalog_record_invariant_corruption_fails_with_label() {
    let path = temp_db_path("check_catalog_record_invariant_corruption_fails_with_label");

    append_raw_record(&path, &row_record("ghosts", &[(b'I', "1")]));

    let output = check_db(&path);
    assert_check_failed(&output, "catalog/record invariant");

    cleanup(&path);
}

#[test]
fn check_primary_index_duplicate_key_corruption_fails_with_label() {
    let path = temp_db_path("check_primary_index_duplicate_key_corruption_fails_with_label");

    append_raw_record(
        &path,
        &catalog_record("users", &[("id", b'I', true), ("name", b'T', false)]),
    );
    append_raw_record(&path, &row_record("users", &[(b'I', "1"), (b'T', "ada")]));
    append_raw_record(&path, &row_record("users", &[(b'I', "1"), (b'T', "bea")]));

    let output = check_db(&path);
    assert_check_failed(&output, "primary index");

    cleanup(&path);
}

#[test]
fn check_wal_ahead_of_store_corruption_fails_with_label() {
    let path = temp_db_path("check_wal_ahead_of_store_corruption_fails_with_label");

    assert_exec_ok(&path, "CREATE TABLE users (id INT, name TEXT);");

    let sidecar = wal_path(&path);
    let ghost_row = row_record("users", &[(b'I', "9"), (b'T', "ghost")]);
    let durable_record_count = 1u64;
    let ahead_of_store_record_count_before = durable_record_count + 1;
    fs::write(
        &sidecar,
        committed_wal_frame(1, ahead_of_store_record_count_before, &ghost_row),
    )
    .expect("ahead-of-store WAL fixture should be written");

    let output = check_db(&path);
    assert_check_failed(&output, "wal replay consistency");

    cleanup(&path);
}

#[test]
fn check_wal_count_valid_unreplayable_payload_fails_with_label() {
    let path = temp_db_path("check_wal_count_valid_unreplayable_payload_fails_with_label");

    assert_exec_ok(&path, "CREATE TABLE users (id INT, name TEXT);");

    let sidecar = wal_path(&path);
    let durable_record_count = 1u64;
    let unreplayable_payload = vec![b'x'; MAX_RECORD_PAYLOAD_LEN + 1];
    fs::write(
        &sidecar,
        committed_wal_frame(1, durable_record_count, &unreplayable_payload),
    )
    .expect("count-valid unreplayable WAL fixture should be written");

    let output = check_db(&path);
    assert_check_failed(&output, "wal replay consistency");

    cleanup(&path);
}

#[test]
fn check_missing_path_fails_as_user_open_read_error() {
    let path = temp_db_path("check_missing_path_fails_as_user_open_read_error");
    cleanup(&path);

    let output = check_db(&path);
    assert_user_open_read_error(&output, &path);
}

#[test]
fn check_directory_path_fails_as_user_open_read_error() {
    let path = temp_db_path("check_directory_path_fails_as_user_open_read_error");
    let dir = path
        .parent()
        .expect("temp db path should have parent")
        .to_path_buf();

    let output = check_db(&dir);
    assert_user_open_read_error(&output, &dir);

    cleanup(&dir);
}

#[cfg(unix)]
#[test]
fn check_unreadable_regular_file_fails_as_user_open_read_error() {
    let path = temp_db_path("check_unreadable_regular_file_fails_as_user_open_read_error");

    assert_exec_ok(&path, "CREATE TABLE users (id INT, name TEXT);");
    let original_permissions = fs::metadata(&path)
        .expect("fixture metadata should be readable")
        .permissions();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o000))
        .expect("fixture permissions should be removable");

    let output = check_db(&path);

    fs::set_permissions(&path, original_permissions).expect("fixture permissions should restore");
    assert_user_open_read_error(&output, &path);

    cleanup(&path);
}
