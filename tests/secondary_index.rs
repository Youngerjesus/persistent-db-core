use persistent_db_core::index::SecondaryIndex;
use persistent_db_core::sql::{plan_query_path_for_test, QueryPath};
use persistent_db_core::storage::PageStore;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const SQL_RECORD_PREFIX: &[u8; 8] = b"PDBSQL1\0";
const INVALID_SQL_STORAGE_STDERR: &str = "error: invalid SQL storage record: unknown record tag\nhint: run against a database file created by this SQL contract or restore from a valid backup.\n";
const UNSUPPORTED_SQL_HINT: &str =
    "hint: supported SQL subset is documented in docs/sql_subset.md.\n";
const WAL_MAGIC: &[u8; 8] = b"PDBWAL1\0";
const WAL_VERSION: u16 = 1;
const WAL_STATE_COMMITTED: u8 = 0x01;
const WAL_PAYLOAD_KIND_PAGE_APPEND: u8 = 0x01;
const WAL_HEADER_LEN: usize = 36;

#[derive(Clone, Copy)]
struct EmbeddedIndexEntry<'a> {
    build_id: u64,
    index_name: &'a str,
    indexed_key: i64,
    tie_break: i64,
    row_position: u64,
}

#[derive(Debug, PartialEq, Eq)]
struct DecodedSecondaryMetadata {
    build_id: u64,
    index_name: String,
    table_name: String,
    indexed_column: u16,
    tie_break_mode: u8,
}

#[derive(Debug, PartialEq, Eq)]
struct DecodedSecondaryEntry {
    build_id: u64,
    index_name: String,
    indexed_key: i64,
    tie_break: i64,
    row_position: u64,
}

type FixtureBuilder = Box<dyn Fn(&Path)>;

fn db(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_db"))
        .args(args)
        .output()
        .expect("db binary should run")
}

fn temp_db_path(test_name: &str) -> PathBuf {
    let mut dir = std::env::temp_dir();
    dir.push(format!(
        "persistent_db_core_secondary_index_{}_{}_{}",
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

fn check_db(path: &Path) -> Output {
    db(&["check", path.to_str().expect("temp path should be UTF-8")])
}

fn wal_path(path: &Path) -> PathBuf {
    PathBuf::from(format!(
        "{}.wal",
        path.to_str().expect("temp path should be UTF-8")
    ))
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

fn assert_check_ok(path: &Path) {
    let output = check_db(path);
    assert_eq!(
        Some(0),
        output.status.code(),
        "db check should pass; stdout={:?}; stderr={:?}",
        stdout(&output),
        stderr(&output)
    );
    assert_eq!("ok: db check passed\n", stdout(&output));
    assert_eq!("", stderr(&output));
}

fn assert_check_secondary_index_failure(path: &Path) {
    let output = check_db(path);
    assert_eq!(
        Some(1),
        output.status.code(),
        "db check should fail; stdout={:?}; stderr={:?}",
        stdout(&output),
        stderr(&output)
    );
    assert_eq!("", stdout(&output));
    assert_eq!("error: db check failed: secondary index\n", stderr(&output));
}

fn append_fixture_record(path: &Path, payload: &[u8]) {
    let mut store = PageStore::open(path).expect("fixture database should open");
    store
        .append_record(payload)
        .expect("fixture record should append");
}

fn append_committed_wal_frame(
    path: &Path,
    frame_id: u64,
    record_count_before: u64,
    payload: &[u8],
) {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(wal_path(path))
        .expect("WAL sidecar should open for append");
    file.write_all(&committed_wal_frame(frame_id, record_count_before, payload))
        .expect("WAL frame should append");
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

    let checksum = wal_checksum(&frame);
    frame[32..36].copy_from_slice(&checksum.to_le_bytes());
    frame
}

fn wal_checksum(frame_with_zero_checksum: &[u8]) -> u32 {
    frame_with_zero_checksum
        .iter()
        .enumerate()
        .filter(|(index, _)| !(32..36).contains(index))
        .fold(0u32, |sum, (_, byte)| sum.wrapping_add(*byte as u32))
}

fn read_fixture_records(path: &Path) -> Vec<Vec<u8>> {
    PageStore::open(path)
        .expect("fixture database should open")
        .read_records()
        .expect("fixture records should read")
}

fn sql_record_kind(record: &[u8]) -> u8 {
    assert!(
        record.starts_with(SQL_RECORD_PREFIX),
        "fixture record should be a SQL logical record"
    );
    record[SQL_RECORD_PREFIX.len()]
}

fn count_record_kind(records: &[Vec<u8>], kind: u8) -> usize {
    records
        .iter()
        .filter(|record| sql_record_kind(record) == kind)
        .count()
}

fn indexed_row_embedded_entry_count(record: &[u8]) -> u16 {
    assert_eq!(b'I', sql_record_kind(record));
    let mut offset = SQL_RECORD_PREFIX.len() + 1;
    let table_len = read_u16(record, &mut offset) as usize;
    offset += table_len;
    let value_count = read_u16(record, &mut offset);
    for _ in 0..value_count {
        offset += 1;
        let value_len = read_u32(record, &mut offset) as usize;
        offset += value_len;
    }
    read_u16(record, &mut offset)
}

fn decode_secondary_metadata(record: &[u8]) -> DecodedSecondaryMetadata {
    assert_eq!(b'X', sql_record_kind(record));
    let mut offset = SQL_RECORD_PREFIX.len() + 1;
    let build_id = read_u64(record, &mut offset);
    let index_name = read_string_u16(record, &mut offset);
    let table_name = read_string_u16(record, &mut offset);
    let indexed_column = read_u16(record, &mut offset);
    let tie_break_mode = record[offset];
    DecodedSecondaryMetadata {
        build_id,
        index_name,
        table_name,
        indexed_column,
        tie_break_mode,
    }
}

fn decode_secondary_entry(record: &[u8]) -> DecodedSecondaryEntry {
    assert_eq!(b'E', sql_record_kind(record));
    let mut offset = SQL_RECORD_PREFIX.len() + 1;
    let build_id = read_u64(record, &mut offset);
    let index_name = read_string_u16(record, &mut offset);
    let indexed_key = read_i64(record, &mut offset);
    let tie_break = read_i64(record, &mut offset);
    let row_position = read_u64(record, &mut offset);
    DecodedSecondaryEntry {
        build_id,
        index_name,
        indexed_key,
        tie_break,
        row_position,
    }
}

fn read_u16(record: &[u8], offset: &mut usize) -> u16 {
    let end = *offset + 2;
    let value = u16::from_le_bytes(
        record[*offset..end]
            .try_into()
            .expect("fixture should have u16 bytes"),
    );
    *offset = end;
    value
}

fn read_u64(record: &[u8], offset: &mut usize) -> u64 {
    let end = *offset + 8;
    let value = u64::from_le_bytes(
        record[*offset..end]
            .try_into()
            .expect("fixture should have u64 bytes"),
    );
    *offset = end;
    value
}

fn read_i64(record: &[u8], offset: &mut usize) -> i64 {
    let end = *offset + 8;
    let value = i64::from_le_bytes(
        record[*offset..end]
            .try_into()
            .expect("fixture should have i64 bytes"),
    );
    *offset = end;
    value
}

fn read_string_u16(record: &[u8], offset: &mut usize) -> String {
    let len = read_u16(record, offset) as usize;
    let end = *offset + len;
    let value =
        String::from_utf8(record[*offset..end].to_vec()).expect("fixture string should be UTF-8");
    *offset = end;
    value
}

fn read_u32(record: &[u8], offset: &mut usize) -> u32 {
    let end = *offset + 4;
    let value = u32::from_le_bytes(
        record[*offset..end]
            .try_into()
            .expect("fixture should have u32 bytes"),
    );
    *offset = end;
    value
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
    for (index, (name, column_type, is_primary_key)) in columns.iter().enumerate() {
        write_string_u16(&mut record, name);
        record.push(*column_type);
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

fn secondary_index_metadata_record(
    build_id: u64,
    index_name: &str,
    table_name: &str,
    indexed_column: u16,
    tie_break_mode: u8,
) -> Vec<u8> {
    let mut record = Vec::new();
    record.extend_from_slice(SQL_RECORD_PREFIX);
    record.push(b'X');
    record.extend_from_slice(&build_id.to_le_bytes());
    write_string_u16(&mut record, index_name);
    write_string_u16(&mut record, table_name);
    record.extend_from_slice(&indexed_column.to_le_bytes());
    record.push(tie_break_mode);
    record
}

fn secondary_index_entry_record(
    build_id: u64,
    index_name: &str,
    indexed_key: i64,
    tie_break: i64,
    row_position: u64,
) -> Vec<u8> {
    let mut record = Vec::new();
    record.extend_from_slice(SQL_RECORD_PREFIX);
    record.push(b'E');
    record.extend_from_slice(&build_id.to_le_bytes());
    write_string_u16(&mut record, index_name);
    record.extend_from_slice(&indexed_key.to_le_bytes());
    record.extend_from_slice(&tie_break.to_le_bytes());
    record.extend_from_slice(&row_position.to_le_bytes());
    record
}

fn indexed_row_record(
    table: &str,
    values: &[(u8, &str)],
    embedded_entries: &[EmbeddedIndexEntry<'_>],
) -> Vec<u8> {
    let mut record = Vec::new();
    record.extend_from_slice(SQL_RECORD_PREFIX);
    record.push(b'I');
    write_string_u16(&mut record, table);
    record.extend_from_slice(&(values.len() as u16).to_le_bytes());
    for (value_type, value) in values {
        record.push(*value_type);
        record.extend_from_slice(&(value.len() as u32).to_le_bytes());
        record.extend_from_slice(value.as_bytes());
    }
    record.extend_from_slice(&(embedded_entries.len() as u16).to_le_bytes());
    for entry in embedded_entries {
        record.extend_from_slice(&entry.build_id.to_le_bytes());
        write_string_u16(&mut record, entry.index_name);
        record.extend_from_slice(&entry.indexed_key.to_le_bytes());
        record.extend_from_slice(&entry.tie_break.to_le_bytes());
        record.extend_from_slice(&entry.row_position.to_le_bytes());
    }
    record
}

fn append_users_catalog_with_one_row(path: &Path) {
    append_fixture_record(
        path,
        &catalog_record(
            "users",
            &[
                ("id", b'I', true),
                ("age", b'I', false),
                ("score", b'I', false),
                ("name", b'T', false),
            ],
        ),
    );
    append_fixture_record(
        path,
        &row_record(
            "users",
            &[(b'I', "1"), (b'I', "10"), (b'I', "100"), (b'T', "ada")],
        ),
    );
}

#[test]
fn secondary_index_primitive_orders_equality_range_and_rejects_duplicate_entries() {
    let mut index = SecondaryIndex::new();

    index.insert(20, 3, 0).expect("entry should insert");
    index.insert(10, 1, 1).expect("entry should insert");
    index.insert(20, 2, 2).expect("entry should insert");

    assert_eq!(vec![2, 0], index.equality_positions(20));
    assert_eq!(vec![1, 2, 0], index.range_positions(10, 20));
    assert_eq!(Vec::<usize>::new(), index.equality_positions(99));
    assert_eq!(Vec::<usize>::new(), index.range_positions(30, 20));
    assert!(index.insert(20, 2, 99).is_err());
    assert_eq!(
        vec![2, 0],
        index.equality_positions(20),
        "duplicate insert must not overwrite the original row position"
    );
}

#[test]
fn planner_path_marker_reports_secondary_equality_and_range_after_create_index() {
    let path = temp_db_path("planner_path_marker_reports_secondary_equality_and_range");

    assert_exec(
        &path,
        "CREATE TABLE users (id INT PRIMARY KEY, age INT, name TEXT); INSERT INTO users VALUES (1, 20, 'ada'); CREATE INDEX idx_users_age ON users(age);",
        0,
        "",
        "",
    );

    assert_eq!(
        QueryPath::SecondaryIndexEquality {
            table: "users".to_string(),
            index: "idx_users_age".to_string(),
            column: "age".to_string(),
        },
        plan_query_path_for_test(&path, "SELECT * FROM users WHERE age = 20;")
            .expect("secondary equality should plan")
    );
    assert_eq!(
        QueryPath::SecondaryIndexRange {
            table: "users".to_string(),
            index: "idx_users_age".to_string(),
            column: "age".to_string(),
        },
        plan_query_path_for_test(&path, "SELECT * FROM users WHERE age BETWEEN 10 AND 20;")
            .expect("secondary range should plan")
    );

    cleanup(&path);
}

#[test]
fn primary_key_column_with_secondary_index_uses_secondary_equality_and_range_paths() {
    let path = temp_db_path("primary_key_column_with_secondary_index_uses_secondary_paths");

    assert_exec(
        &path,
        "CREATE TABLE users (id INT PRIMARY KEY, age INT, name TEXT); INSERT INTO users VALUES (2, 20, 'bea'); INSERT INTO users VALUES (1, 10, 'ada'); CREATE INDEX idx_users_id ON users(id);",
        0,
        "",
        "",
    );

    assert_eq!(
        QueryPath::SecondaryIndexEquality {
            table: "users".to_string(),
            index: "idx_users_id".to_string(),
            column: "id".to_string(),
        },
        plan_query_path_for_test(&path, "SELECT * FROM users WHERE id = 2;")
            .expect("primary-key column equality should use explicit secondary index")
    );
    assert_eq!(
        QueryPath::SecondaryIndexRange {
            table: "users".to_string(),
            index: "idx_users_id".to_string(),
            column: "id".to_string(),
        },
        plan_query_path_for_test(&path, "SELECT * FROM users WHERE id BETWEEN 1 AND 2;")
            .expect("primary-key column range should use explicit secondary index")
    );
    assert_exec(
        &path,
        "SELECT * FROM users WHERE id BETWEEN 1 AND 2;",
        0,
        "id|age|name\n1|10|ada\n2|20|bea\n",
        "",
    );

    cleanup(&path);
}

#[test]
fn create_index_success_is_silent_and_required_equality_example_uses_index_order() {
    let path = temp_db_path("create_index_success_is_silent_and_required_equality_example");

    assert_exec(
        &path,
        "CREATE TABLE users (id INT PRIMARY KEY, age INT, name TEXT); INSERT INTO users VALUES (3, 20, 'cal'); INSERT INTO users VALUES (1, 10, 'ada'); INSERT INTO users VALUES (2, 20, 'bea');",
        0,
        "",
        "",
    );
    assert_exec(
        &path,
        "CREATE INDEX idx_users_age ON users(age);",
        0,
        "",
        "",
    );
    assert_exec(
        &path,
        "SELECT * FROM users WHERE age = 20;",
        0,
        "id|age|name\n2|20|bea\n3|20|cal\n",
        "",
    );

    cleanup(&path);
}

#[test]
fn between_range_scan_is_inclusive_and_ordered_by_secondary_key_then_primary_key() {
    let path = temp_db_path("between_range_scan_is_inclusive_and_ordered");

    assert_exec(
        &path,
        "CREATE TABLE users (id INT PRIMARY KEY, age INT, name TEXT); INSERT INTO users VALUES (3, 20, 'cal'); INSERT INTO users VALUES (1, 10, 'ada'); INSERT INTO users VALUES (2, 20, 'bea'); CREATE INDEX idx_users_age ON users(age); SELECT * FROM users WHERE age BETWEEN 10 AND 20;",
        0,
        "id|age|name\n1|10|ada\n2|20|bea\n3|20|cal\n",
        "",
    );

    cleanup(&path);
}

#[test]
fn create_index_semantic_errors_match_contract_exactly() {
    let cases = [
        (
            "missing_table",
            "CREATE INDEX idx_users_age ON missing(age);",
            "error: SQL semantic error: table not found: missing\nhint: create the table before INSERT, SELECT, or CREATE INDEX.\n",
        ),
        (
            "missing_column",
            "CREATE TABLE users (id INT); CREATE INDEX idx_users_age ON users(age);",
            "error: SQL semantic error: column not found for index idx_users_age: age\nhint: create the index on an existing table column.\n",
        ),
        (
            "unsupported_type",
            "CREATE TABLE users (id INT, name TEXT); CREATE INDEX idx_users_name ON users(name);",
            "error: SQL semantic error: secondary index column must be INT: name\nhint: this SQL slice supports secondary indexes only on INT columns.\n",
        ),
        (
            "duplicate_index",
            "CREATE TABLE users (id INT, age INT); CREATE INDEX idx_users_age ON users(age); CREATE INDEX idx_users_age ON users(age);",
            "error: SQL semantic error: index already exists: idx_users_age\nhint: use a new index name for CREATE INDEX in this database.\n",
        ),
    ];

    for (name, sql, expected_stderr) in cases {
        let path = temp_db_path(name);
        assert_exec(&path, sql, 2, "", expected_stderr);
        cleanup(&path);
    }
}

#[test]
fn non_primary_key_predicate_before_create_index_is_unsupported_not_full_scan() {
    let path = temp_db_path("non_primary_key_predicate_before_create_index_is_unsupported");

    assert_exec(
        &path,
        "CREATE TABLE users (id INT PRIMARY KEY, age INT, name TEXT); INSERT INTO users VALUES (1, 20, 'ada');",
        0,
        "",
        "",
    );
    assert_exec(
        &path,
        "SELECT * FROM users WHERE age = 20;",
        2,
        "",
        &format!(
            "error: unsupported SQL statement: SELECT * FROM users WHERE age = 20;\n{}",
            UNSUPPORTED_SQL_HINT
        ),
    );

    cleanup(&path);
}

#[test]
fn range_predicate_before_create_index_is_unsupported_not_full_scan() {
    let path = temp_db_path("range_predicate_before_create_index_is_unsupported_not_full_scan");

    assert_exec(
        &path,
        "CREATE TABLE users (id INT PRIMARY KEY, age INT, name TEXT); INSERT INTO users VALUES (1, 20, 'ada');",
        0,
        "",
        "",
    );
    assert_exec(
        &path,
        "SELECT * FROM users WHERE age BETWEEN 10 AND 20;",
        2,
        "",
        &format!(
            "error: unsupported SQL statement: SELECT * FROM users WHERE age BETWEEN 10 AND 20;\n{}",
            UNSUPPORTED_SQL_HINT
        ),
    );

    cleanup(&path);
}

#[test]
fn indexed_range_with_low_greater_than_high_returns_header_only_through_range_path() {
    let path = temp_db_path("indexed_range_with_low_greater_than_high_returns_header_only");

    assert_exec(
        &path,
        "CREATE TABLE users (id INT PRIMARY KEY, age INT, name TEXT); INSERT INTO users VALUES (1, 10, 'ada'); INSERT INTO users VALUES (2, 20, 'bea'); CREATE INDEX idx_users_age ON users(age);",
        0,
        "",
        "",
    );
    assert_eq!(
        QueryPath::SecondaryIndexRange {
            table: "users".to_string(),
            index: "idx_users_age".to_string(),
            column: "age".to_string(),
        },
        plan_query_path_for_test(&path, "SELECT * FROM users WHERE age BETWEEN 30 AND 20;")
            .expect("low-greater-than-high range should still plan through the secondary index")
    );
    assert_exec(
        &path,
        "SELECT * FROM users WHERE age BETWEEN 30 AND 20;",
        0,
        "id|age|name\n",
        "",
    );

    cleanup(&path);
}

#[test]
fn no_primary_key_duplicate_secondary_key_tie_breaks_by_insertion_order() {
    let path = temp_db_path("no_primary_key_duplicate_secondary_key_tie_breaks");

    assert_exec(
        &path,
        "CREATE TABLE users (age INT, name TEXT); INSERT INTO users VALUES (20, 'cal'); INSERT INTO users VALUES (10, 'ada'); INSERT INTO users VALUES (20, 'bea'); CREATE INDEX idx_users_age ON users(age); SELECT * FROM users WHERE age = 20;",
        0,
        "age|name\n20|cal\n20|bea\n",
        "",
    );

    cleanup(&path);
}

#[test]
fn old_no_index_database_reopens_then_backfills_and_post_index_insert_persists() {
    let path = temp_db_path("old_no_index_database_reopens_then_backfills");

    append_fixture_record(
        &path,
        &catalog_record(
            "users",
            &[
                ("id", b'I', true),
                ("age", b'I', false),
                ("name", b'T', false),
            ],
        ),
    );
    append_fixture_record(
        &path,
        &row_record("users", &[(b'I', "1"), (b'I', "10"), (b'T', "ada")]),
    );
    append_fixture_record(
        &path,
        &row_record("users", &[(b'I', "2"), (b'I', "20"), (b'T', "bea")]),
    );

    assert_exec(
        &path,
        "SELECT * FROM users;",
        0,
        "id|age|name\n1|10|ada\n2|20|bea\n",
        "",
    );
    assert_exec(
        &path,
        "CREATE INDEX idx_users_age ON users(age);",
        0,
        "",
        "",
    );
    assert_exec(&path, "INSERT INTO users VALUES (3, 20, 'cal');", 0, "", "");
    assert_exec(
        &path,
        "SELECT * FROM users WHERE age BETWEEN 10 AND 20;",
        0,
        "id|age|name\n1|10|ada\n2|20|bea\n3|20|cal\n",
        "",
    );
    assert_exec(
        &path,
        "SELECT * FROM users WHERE age = 20;",
        0,
        "id|age|name\n2|20|bea\n3|20|cal\n",
        "",
    );

    cleanup(&path);
}

#[test]
fn committed_secondary_index_metadata_and_entries_survive_reopen() {
    let path = temp_db_path("committed_secondary_index_metadata_and_entries_survive_reopen");

    assert_exec(
        &path,
        "CREATE TABLE users (id INT PRIMARY KEY, age INT, name TEXT); INSERT INTO users VALUES (1, 10, 'ada'); INSERT INTO users VALUES (2, 20, 'bea'); CREATE INDEX idx_users_age ON users(age);",
        0,
        "",
        "",
    );
    assert_exec(
        &path,
        "SELECT * FROM users WHERE age = 20;",
        0,
        "id|age|name\n2|20|bea\n",
        "",
    );
    assert_check_ok(&path);

    cleanup(&path);
}

#[test]
fn committed_wal_replay_applies_secondary_backfill_entries_and_metadata() {
    let path = temp_db_path("committed_wal_replay_applies_secondary_backfill");

    assert_exec(
        &path,
        "CREATE TABLE users (id INT PRIMARY KEY, age INT, name TEXT); INSERT INTO users VALUES (1, 10, 'ada');",
        0,
        "",
        "",
    );
    append_committed_wal_frame(
        &path,
        3,
        2,
        &secondary_index_entry_record(2, "idx_users_age", 10, 1, 0),
    );
    append_committed_wal_frame(
        &path,
        4,
        3,
        &secondary_index_metadata_record(2, "idx_users_age", "users", 1, b'P'),
    );

    assert_exec(
        &path,
        "SELECT * FROM users WHERE age BETWEEN 10 AND 10;",
        0,
        "id|age|name\n1|10|ada\n",
        "",
    );
    assert_check_ok(&path);

    cleanup(&path);
}

#[test]
fn committed_wal_replay_applies_atomic_indexed_row_record() {
    let path = temp_db_path("committed_wal_replay_applies_atomic_indexed_row");

    assert_exec(
        &path,
        "CREATE TABLE users (id INT PRIMARY KEY, age INT); CREATE INDEX idx_users_age ON users(age);",
        0,
        "",
        "",
    );
    append_committed_wal_frame(
        &path,
        3,
        2,
        &indexed_row_record(
            "users",
            &[(b'I', "1"), (b'I', "10")],
            &[EmbeddedIndexEntry {
                build_id: 1,
                index_name: "idx_users_age",
                indexed_key: 10,
                tie_break: 1,
                row_position: 0,
            }],
        ),
    );

    assert_exec(
        &path,
        "SELECT * FROM users WHERE age = 10;",
        0,
        "id|age\n1|10\n",
        "",
    );
    assert_check_ok(&path);

    cleanup(&path);
}

#[test]
fn stale_orphan_backfill_entries_are_ignored_and_retry_commits_fresh_build_id() {
    let path = temp_db_path("stale_orphan_backfill_entries_are_ignored");

    append_fixture_record(
        &path,
        &catalog_record(
            "users",
            &[
                ("id", b'I', true),
                ("age", b'I', false),
                ("name", b'T', false),
            ],
        ),
    );
    append_fixture_record(
        &path,
        &row_record("users", &[(b'I', "1"), (b'I', "10"), (b'T', "ada")]),
    );
    append_fixture_record(
        &path,
        &secondary_index_entry_record(2, "idx_users_age", 99, 9, 0),
    );

    assert_check_ok(&path);
    assert_exec(
        &path,
        "CREATE INDEX idx_users_age ON users(age);",
        0,
        "",
        "",
    );
    assert_exec(
        &path,
        "SELECT * FROM users WHERE age = 10;",
        0,
        "id|age|name\n1|10|ada\n",
        "",
    );
    assert_exec(
        &path,
        "SELECT * FROM users WHERE age BETWEEN 10 AND 10;",
        0,
        "id|age|name\n1|10|ada\n",
        "",
    );
    assert_check_ok(&path);

    let records = read_fixture_records(&path);
    let stale_entries: Vec<DecodedSecondaryEntry> = records
        .iter()
        .filter(|record| sql_record_kind(record) == b'E')
        .map(|record| decode_secondary_entry(record))
        .filter(|entry| entry.build_id == 2)
        .collect();
    assert_eq!(
        vec![DecodedSecondaryEntry {
            build_id: 2,
            index_name: "idx_users_age".to_string(),
            indexed_key: 99,
            tie_break: 9,
            row_position: 0,
        }],
        stale_entries,
        "fixture must retain a distinguishable stale orphan entry"
    );

    let retry_metadata: Vec<DecodedSecondaryMetadata> = records
        .iter()
        .filter(|record| sql_record_kind(record) == b'X')
        .map(|record| decode_secondary_metadata(record))
        .collect();
    assert_eq!(
        vec![DecodedSecondaryMetadata {
            build_id: 3,
            index_name: "idx_users_age".to_string(),
            table_name: "users".to_string(),
            indexed_column: 1,
            tie_break_mode: b'P',
        }],
        retry_metadata,
        "retry must commit a fresh build id instead of attaching stale orphan entries"
    );

    let retry_entries: Vec<DecodedSecondaryEntry> = records
        .iter()
        .filter(|record| sql_record_kind(record) == b'E')
        .map(|record| decode_secondary_entry(record))
        .filter(|entry| entry.build_id == 3)
        .collect();
    assert_eq!(
        vec![DecodedSecondaryEntry {
            build_id: 3,
            index_name: "idx_users_age".to_string(),
            indexed_key: 10,
            tie_break: 1,
            row_position: 0,
        }],
        retry_entries,
        "retry must write a fresh valid backfill entry set"
    );

    let last_two_kinds: Vec<u8> = records
        .iter()
        .rev()
        .take(2)
        .map(|record| sql_record_kind(record))
        .collect();
    assert_eq!(
        vec![b'X', b'E'],
        last_two_kinds,
        "retry must append fresh E records before the final X commit marker"
    );

    cleanup(&path);
}

#[test]
fn db_check_reports_secondary_index_for_committed_entry_mismatch() {
    let path = temp_db_path("db_check_reports_secondary_index_for_committed_entry_mismatch");

    append_fixture_record(
        &path,
        &catalog_record(
            "users",
            &[
                ("id", b'I', true),
                ("age", b'I', false),
                ("name", b'T', false),
            ],
        ),
    );
    append_fixture_record(
        &path,
        &row_record("users", &[(b'I', "1"), (b'I', "10"), (b'T', "ada")]),
    );
    append_fixture_record(
        &path,
        &secondary_index_entry_record(2, "idx_users_age", 99, 1, 0),
    );
    append_fixture_record(
        &path,
        &secondary_index_metadata_record(2, "idx_users_age", "users", 1, b'P'),
    );

    assert_check_secondary_index_failure(&path);
    assert_exec(
        &path,
        "SELECT * FROM users;",
        1,
        "",
        INVALID_SQL_STORAGE_STDERR,
    );

    cleanup(&path);
}

#[test]
fn db_check_secondary_index_corruption_matrix_reports_secondary_index() {
    let cases: &[(&str, FixtureBuilder)] = &[
        (
            "missing_entry",
            Box::new(|path| {
                append_users_catalog_with_one_row(path);
                append_fixture_record(
                    path,
                    &secondary_index_metadata_record(2, "idx_users_age", "users", 1, b'P'),
                );
            }),
        ),
        (
            "wrong_key",
            Box::new(|path| {
                append_users_catalog_with_one_row(path);
                append_fixture_record(
                    path,
                    &secondary_index_entry_record(2, "idx_users_age", 99, 1, 0),
                );
                append_fixture_record(
                    path,
                    &secondary_index_metadata_record(2, "idx_users_age", "users", 1, b'P'),
                );
            }),
        ),
        (
            "wrong_tie_break",
            Box::new(|path| {
                append_users_catalog_with_one_row(path);
                append_fixture_record(
                    path,
                    &secondary_index_entry_record(2, "idx_users_age", 10, 9, 0),
                );
                append_fixture_record(
                    path,
                    &secondary_index_metadata_record(2, "idx_users_age", "users", 1, b'P'),
                );
            }),
        ),
        (
            "invalid_row_position",
            Box::new(|path| {
                append_users_catalog_with_one_row(path);
                append_fixture_record(
                    path,
                    &secondary_index_entry_record(2, "idx_users_age", 10, 1, 9),
                );
                append_fixture_record(
                    path,
                    &secondary_index_metadata_record(2, "idx_users_age", "users", 1, b'P'),
                );
            }),
        ),
        (
            "duplicate_entry",
            Box::new(|path| {
                append_users_catalog_with_one_row(path);
                append_fixture_record(
                    path,
                    &secondary_index_entry_record(2, "idx_users_age", 10, 1, 0),
                );
                append_fixture_record(
                    path,
                    &secondary_index_entry_record(2, "idx_users_age", 10, 1, 0),
                );
                append_fixture_record(
                    path,
                    &secondary_index_metadata_record(2, "idx_users_age", "users", 1, b'P'),
                );
            }),
        ),
        (
            "missing_table_metadata",
            Box::new(|path| {
                append_users_catalog_with_one_row(path);
                append_fixture_record(
                    path,
                    &secondary_index_entry_record(2, "idx_ghost_age", 10, 1, 0),
                );
                append_fixture_record(
                    path,
                    &secondary_index_metadata_record(2, "idx_ghost_age", "ghosts", 1, b'P'),
                );
            }),
        ),
        (
            "missing_column_metadata",
            Box::new(|path| {
                append_users_catalog_with_one_row(path);
                append_fixture_record(
                    path,
                    &secondary_index_entry_record(2, "idx_users_missing", 10, 1, 0),
                );
                append_fixture_record(
                    path,
                    &secondary_index_metadata_record(2, "idx_users_missing", "users", 99, b'P'),
                );
            }),
        ),
        (
            "non_int_column_metadata",
            Box::new(|path| {
                append_users_catalog_with_one_row(path);
                append_fixture_record(
                    path,
                    &secondary_index_metadata_record(2, "idx_users_name", "users", 3, b'P'),
                );
            }),
        ),
        (
            "duplicate_index_metadata",
            Box::new(|path| {
                append_fixture_record(
                    path,
                    &catalog_record(
                        "users",
                        &[
                            ("id", b'I', true),
                            ("age", b'I', false),
                            ("name", b'T', false),
                        ],
                    ),
                );
                append_fixture_record(
                    path,
                    &secondary_index_metadata_record(1, "idx_users_age", "users", 1, b'P'),
                );
                append_fixture_record(
                    path,
                    &secondary_index_metadata_record(2, "IDX_USERS_AGE", "users", 1, b'P'),
                );
            }),
        ),
    ];

    for (name, build_fixture) in cases {
        let path = temp_db_path(name);
        build_fixture(&path);
        assert_check_secondary_index_failure(&path);
        cleanup(&path);
    }
}

#[test]
fn db_check_reports_secondary_index_for_matching_entry_appended_after_commit() {
    let path = temp_db_path("db_check_reports_secondary_index_for_post_commit_entry");

    append_users_catalog_with_one_row(&path);
    append_fixture_record(
        &path,
        &secondary_index_entry_record(2, "idx_users_age", 10, 1, 0),
    );
    append_fixture_record(
        &path,
        &secondary_index_metadata_record(2, "idx_users_age", "users", 1, b'P'),
    );
    append_fixture_record(
        &path,
        &secondary_index_entry_record(2, "idx_users_age", 10, 1, 0),
    );

    assert_check_secondary_index_failure(&path);
    cleanup(&path);
}

#[test]
fn indexed_row_corruption_matrix_reports_secondary_index() {
    let cases: &[(&str, Vec<EmbeddedIndexEntry<'_>>)] = &[
        (
            "missing_embedded_entry",
            vec![EmbeddedIndexEntry {
                build_id: 1,
                index_name: "idx_users_age",
                indexed_key: 10,
                tie_break: 1,
                row_position: 0,
            }],
        ),
        (
            "extra_embedded_entry",
            vec![
                EmbeddedIndexEntry {
                    build_id: 1,
                    index_name: "idx_users_age",
                    indexed_key: 10,
                    tie_break: 1,
                    row_position: 0,
                },
                EmbeddedIndexEntry {
                    build_id: 2,
                    index_name: "idx_users_score",
                    indexed_key: 100,
                    tie_break: 1,
                    row_position: 0,
                },
                EmbeddedIndexEntry {
                    build_id: 99,
                    index_name: "idx_users_extra",
                    indexed_key: 777,
                    tie_break: 1,
                    row_position: 0,
                },
            ],
        ),
        (
            "wrong_index",
            vec![
                EmbeddedIndexEntry {
                    build_id: 1,
                    index_name: "idx_users_age_typo",
                    indexed_key: 10,
                    tie_break: 1,
                    row_position: 0,
                },
                EmbeddedIndexEntry {
                    build_id: 2,
                    index_name: "idx_users_score",
                    indexed_key: 100,
                    tie_break: 1,
                    row_position: 0,
                },
            ],
        ),
        (
            "wrong_key",
            vec![
                EmbeddedIndexEntry {
                    build_id: 1,
                    index_name: "idx_users_age",
                    indexed_key: 99,
                    tie_break: 1,
                    row_position: 0,
                },
                EmbeddedIndexEntry {
                    build_id: 2,
                    index_name: "idx_users_score",
                    indexed_key: 100,
                    tie_break: 1,
                    row_position: 0,
                },
            ],
        ),
        (
            "wrong_tie_break",
            vec![
                EmbeddedIndexEntry {
                    build_id: 1,
                    index_name: "idx_users_age",
                    indexed_key: 10,
                    tie_break: 9,
                    row_position: 0,
                },
                EmbeddedIndexEntry {
                    build_id: 2,
                    index_name: "idx_users_score",
                    indexed_key: 100,
                    tie_break: 1,
                    row_position: 0,
                },
            ],
        ),
        (
            "wrong_row_position",
            vec![
                EmbeddedIndexEntry {
                    build_id: 1,
                    index_name: "idx_users_age",
                    indexed_key: 10,
                    tie_break: 1,
                    row_position: 9,
                },
                EmbeddedIndexEntry {
                    build_id: 2,
                    index_name: "idx_users_score",
                    indexed_key: 100,
                    tie_break: 1,
                    row_position: 0,
                },
            ],
        ),
    ];

    for (name, entries) in cases {
        let path = temp_db_path(name);
        append_fixture_record(
            &path,
            &catalog_record(
                "users",
                &[
                    ("id", b'I', true),
                    ("age", b'I', false),
                    ("score", b'I', false),
                ],
            ),
        );
        append_fixture_record(
            &path,
            &secondary_index_metadata_record(1, "idx_users_age", "users", 1, b'P'),
        );
        append_fixture_record(
            &path,
            &secondary_index_metadata_record(2, "idx_users_score", "users", 2, b'P'),
        );
        append_fixture_record(
            &path,
            &indexed_row_record(
                "users",
                &[(b'I', "1"), (b'I', "10"), (b'I', "100")],
                entries,
            ),
        );

        assert_check_secondary_index_failure(&path);
        cleanup(&path);
    }
}

#[test]
fn post_index_insert_persists_one_atomic_indexed_row_record_for_all_indexes() {
    let path = temp_db_path("post_index_insert_persists_one_atomic_indexed_row_record");

    assert_exec(
        &path,
        "CREATE TABLE users (id INT PRIMARY KEY, age INT, score INT); CREATE INDEX idx_users_age ON users(age); CREATE INDEX idx_users_score ON users(score); INSERT INTO users VALUES (1, 10, 100);",
        0,
        "",
        "",
    );

    let records = read_fixture_records(&path);
    assert_eq!(1, count_record_kind(&records, b'C'));
    assert_eq!(2, count_record_kind(&records, b'X'));
    assert_eq!(1, count_record_kind(&records, b'I'));
    assert_eq!(
        0,
        count_record_kind(&records, b'R'),
        "post-index inserts must not append row-only R records"
    );
    assert_eq!(
        0,
        count_record_kind(&records, b'E'),
        "post-index inserts must not append standalone E records"
    );

    let indexed_row = records
        .iter()
        .find(|record| sql_record_kind(record) == b'I')
        .expect("post-index insert should append one indexed row record");
    assert_eq!(
        2,
        indexed_row_embedded_entry_count(indexed_row),
        "one I record must embed one entry for each committed index"
    );

    cleanup(&path);
}

#[test]
fn interrupted_post_index_insert_with_no_indexed_row_record_can_be_retried() {
    let path =
        temp_db_path("interrupted_post_index_insert_with_no_indexed_row_record_can_be_retried");

    append_fixture_record(
        &path,
        &catalog_record(
            "users",
            &[
                ("id", b'I', true),
                ("age", b'I', false),
                ("score", b'I', false),
            ],
        ),
    );
    append_fixture_record(
        &path,
        &secondary_index_metadata_record(1, "idx_users_age", "users", 1, b'P'),
    );
    append_fixture_record(
        &path,
        &secondary_index_metadata_record(2, "idx_users_score", "users", 2, b'P'),
    );

    assert_check_ok(&path);
    assert_exec(&path, "INSERT INTO users VALUES (1, 10, 100);", 0, "", "");
    assert_exec(
        &path,
        "SELECT * FROM users WHERE age = 10;",
        0,
        "id|age|score\n1|10|100\n",
        "",
    );

    let records = read_fixture_records(&path);
    assert_eq!(1, count_record_kind(&records, b'I'));
    assert_eq!(0, count_record_kind(&records, b'R'));
    assert_eq!(0, count_record_kind(&records, b'E'));

    cleanup(&path);
}
