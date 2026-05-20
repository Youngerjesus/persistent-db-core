use persistent_db_core::index::PrimaryIndex;
use persistent_db_core::storage::PageStore;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const SQL_RECORD_PREFIX: &[u8; 8] = b"PDBSQL1\0";
const DUPLICATE_PRIMARY_KEY_INVALID_STORAGE_STDERR: &str = "error: invalid SQL storage record: duplicate primary key for table users: 2\nhint: primary key values must be unique in persisted SQL storage.\n";

fn db(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_db"))
        .args(args)
        .output()
        .expect("db binary should run")
}

fn temp_db_path(test_name: &str) -> PathBuf {
    let mut dir = std::env::temp_dir();
    dir.push(format!(
        "persistent_db_core_primary_index_{}_{}_{}",
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

fn append_fixture_record(path: &Path, payload: &[u8]) {
    let mut store = PageStore::open(path).expect("fixture database should open");
    store
        .append_record(payload)
        .expect("fixture record should append");
}

fn catalog_record(table: &str, columns: &[(&str, u8)]) -> Vec<u8> {
    catalog_record_with_primary_key(table, columns, None)
}

fn catalog_record_with_primary_key(
    table: &str,
    columns: &[(&str, u8)],
    primary_key_column: Option<u16>,
) -> Vec<u8> {
    let mut record = Vec::new();
    record.extend_from_slice(SQL_RECORD_PREFIX);
    record.push(b'C');
    write_string_u16(&mut record, table);
    record.extend_from_slice(&(columns.len() as u16).to_le_bytes());
    for (name, column_type) in columns {
        write_string_u16(&mut record, name);
        record.push(*column_type);
    }
    if let Some(primary_key_column) = primary_key_column {
        record.push(b'P');
        record.extend_from_slice(&primary_key_column.to_le_bytes());
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

fn write_string_u16(record: &mut Vec<u8>, value: &str) {
    record.extend_from_slice(&(value.len() as u16).to_le_bytes());
    record.extend_from_slice(value.as_bytes());
}

#[test]
fn primary_index_insert_find_missing_duplicate_and_len() {
    let mut index = PrimaryIndex::new();

    assert!(index.is_empty());
    assert_eq!(None, index.get(9));

    index.insert(2, 0).expect("first key should insert");
    index.insert(1, 1).expect("second key should insert");

    assert_eq!(2, index.len());
    assert_eq!(Some(0), index.get(2));
    assert_eq!(Some(1), index.get(1));
    assert_eq!(None, index.get(3));
    assert!(index.insert(2, 99).is_err());
    assert_eq!(Some(0), index.get(2), "duplicate insert must not overwrite");
}

#[test]
fn primary_index_ordered_positions_are_ascending_by_key() {
    let mut index = PrimaryIndex::new();

    index.insert(30, 0).expect("key should insert");
    index.insert(-5, 1).expect("key should insert");
    index.insert(10, 2).expect("key should insert");

    assert_eq!(vec![1, 2, 0], index.ordered_positions());
}

#[test]
fn primary_index_empty_ordered_positions_are_empty() {
    let index = PrimaryIndex::new();

    assert_eq!(Vec::<usize>::new(), index.ordered_positions());
}

#[test]
fn primary_index_rebuild_from_persisted_rows_survives_reopen() {
    let path = temp_db_path("primary_index_rebuild_from_persisted_rows_survives_reopen");

    assert_exec(
        &path,
        "CREATE TABLE users (id INT PRIMARY KEY, name TEXT); INSERT INTO users VALUES (2, 'bea'); INSERT INTO users VALUES (1, 'ada'); INSERT INTO users VALUES (3, 'cal');",
        0,
        "",
        "",
    );
    assert_exec(
        &path,
        "SELECT * FROM users WHERE id = 2;",
        0,
        "id|name\n2|bea\n",
        "",
    );
    assert_exec(
        &path,
        "SELECT * FROM users;",
        0,
        "id|name\n1|ada\n2|bea\n3|cal\n",
        "",
    );

    cleanup(&path);
}

#[test]
fn primary_index_missing_key_and_empty_scan_are_deterministic() {
    let path = temp_db_path("primary_index_missing_key_and_empty_scan_are_deterministic");

    assert_exec(
        &path,
        "CREATE TABLE users (id INT PRIMARY KEY, name TEXT); SELECT * FROM users WHERE id = 9; SELECT * FROM users;",
        0,
        "id|name\nid|name\n",
        "",
    );

    cleanup(&path);
}

#[test]
fn primary_index_duplicate_persisted_key_fails_as_invalid_storage_record() {
    let path =
        temp_db_path("primary_index_duplicate_persisted_key_fails_as_invalid_storage_record");

    append_fixture_record(
        &path,
        &catalog_record_with_primary_key("users", &[("id", b'I'), ("name", b'T')], Some(0)),
    );
    append_fixture_record(&path, &row_record("users", &[(b'I', "2"), (b'T', "bea")]));
    append_fixture_record(&path, &row_record("users", &[(b'I', "2"), (b'T', "dupe")]));

    assert_exec(
        &path,
        "SELECT * FROM users;",
        1,
        "",
        DUPLICATE_PRIMARY_KEY_INVALID_STORAGE_STDERR,
    );

    cleanup(&path);
}

#[test]
fn primary_index_existing_row_only_catalog_remains_insert_order() {
    let path = temp_db_path("primary_index_existing_row_only_catalog_remains_insert_order");

    append_fixture_record(
        &path,
        &catalog_record("users", &[("id", b'I'), ("name", b'T')]),
    );
    append_fixture_record(&path, &row_record("users", &[(b'I', "2"), (b'T', "bea")]));
    append_fixture_record(&path, &row_record("users", &[(b'I', "1"), (b'T', "ada")]));

    assert_exec(
        &path,
        "SELECT * FROM users;",
        0,
        "id|name\n2|bea\n1|ada\n",
        "",
    );

    cleanup(&path);
}
