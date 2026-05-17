use persistent_db_core::storage::PageStore;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const SQL_RECORD_PREFIX: &[u8; 8] = b"PDBSQL1\0";
const INVALID_SQL_STORAGE_STDERR: &str = "error: invalid SQL storage record: unknown record tag\nhint: run against a database file created by this SQL contract or restore from a valid backup.\n";

fn db(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_db"))
        .args(args)
        .output()
        .expect("db binary should run")
}

fn temp_db_path(test_name: &str) -> PathBuf {
    let mut dir = std::env::temp_dir();
    dir.push(format!(
        "persistent_db_core_sql_exec_{}_{}_{}",
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

fn assert_rejected_without_stdout(path: &Path, sql: &str) {
    let output = exec_sql(path, sql);
    assert_eq!(
        Some(2),
        output.status.code(),
        "unexpected exit; stdout={:?}; stderr={:?}",
        stdout(&output),
        stderr(&output)
    );
    assert_eq!("", stdout(&output));
    assert!(
        stderr(&output).starts_with("error: "),
        "stderr should contain a deterministic user-facing error, got {:?}",
        stderr(&output)
    );
}

fn append_fixture_record(path: &Path, payload: &[u8]) {
    let mut store = PageStore::open(path).expect("fixture database should open");
    store
        .append_record(payload)
        .expect("fixture record should append");
}

fn catalog_record(table: &str, columns: &[(&str, u8)]) -> Vec<u8> {
    let mut record = Vec::new();
    record.extend_from_slice(SQL_RECORD_PREFIX);
    record.push(b'C');
    write_string_u16(&mut record, table);
    record.extend_from_slice(&(columns.len() as u16).to_le_bytes());
    for (name, column_type) in columns {
        write_string_u16(&mut record, name);
        record.push(*column_type);
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
fn happy_path_creates_inserts_and_selects_rows_in_insert_order() {
    let path = temp_db_path("happy_path_creates_inserts_and_selects_rows_in_insert_order");

    assert_exec(
        &path,
        "CREATE TABLE users (id INT, name TEXT); INSERT INTO users VALUES (1, 'ada'); INSERT INTO users VALUES (2, 'bea'); SELECT * FROM users;",
        0,
        "id|name\n1|ada\n2|bea\n",
        "",
    );

    cleanup(&path);
}

#[test]
fn signed_decimal_int_literals_accept_noncanonical_zero_spelling() {
    let path = temp_db_path("signed_decimal_int_literals_accept_noncanonical_zero_spelling");

    assert_exec(
        &path,
        "CREATE TABLE nums (n INT); INSERT INTO nums VALUES (-0); SELECT * FROM nums;",
        0,
        "n\n0\n",
        "",
    );

    cleanup(&path);
}

#[test]
fn restart_persists_catalog_and_rows() {
    let path = temp_db_path("restart_persists_catalog_and_rows");

    assert_exec(
        &path,
        "CREATE TABLE users (id INT, name TEXT); INSERT INTO users VALUES (1, 'ada');",
        0,
        "",
        "",
    );
    assert_exec(&path, "SELECT * FROM users;", 0, "id|name\n1|ada\n", "");

    cleanup(&path);
}

#[test]
fn mid_command_failure_keeps_prior_successes_and_skips_later_statements() {
    let path = temp_db_path("mid_command_failure_keeps_prior_successes_and_skips_later_statements");

    assert_exec(
        &path,
        "CREATE TABLE users (id INT, name TEXT); INSERT INTO users VALUES (1, 'ada'); INSERT INTO users VALUES ('bad', 'type'); INSERT INTO users VALUES (2, 'bea');",
        2,
        "",
        "error: SQL semantic error: type mismatch for column id: expected INT, got TEXT\nhint: INSERT values must match the declared column types.\n",
    );
    assert_exec(&path, "SELECT * FROM users;", 0, "id|name\n1|ada\n", "");

    cleanup(&path);
}

#[test]
fn multiple_selects_repeat_headers_without_separators() {
    let path = temp_db_path("multiple_selects_repeat_headers_without_separators");

    assert_exec(
        &path,
        "CREATE TABLE users (id INT, name TEXT); SELECT * FROM users; INSERT INTO users VALUES (1, 'ada'); SELECT * FROM users;",
        0,
        "id|name\nid|name\n1|ada\n",
        "",
    );

    cleanup(&path);
}

#[test]
fn empty_table_select_outputs_header_only() {
    let path = temp_db_path("empty_table_select_outputs_header_only");

    assert_exec(
        &path,
        "CREATE TABLE users (id INT, name TEXT); SELECT * FROM users;",
        0,
        "id|name\n",
        "",
    );

    cleanup(&path);
}

#[test]
fn failed_command_suppresses_partial_select_stdout() {
    let path = temp_db_path("failed_command_suppresses_partial_select_stdout");

    assert_exec(
        &path,
        "CREATE TABLE users (id INT); INSERT INTO users VALUES (1); SELECT * FROM users; INSERT INTO users VALUES ('bad');",
        2,
        "",
        "error: SQL semantic error: type mismatch for column id: expected INT, got TEXT\nhint: INSERT values must match the declared column types.\n",
    );

    cleanup(&path);
}

#[test]
fn identifiers_compare_case_insensitively_but_preserve_catalog_spelling() {
    let path = temp_db_path("identifiers_compare_case_insensitively_but_preserve_catalog_spelling");

    assert_exec(
        &path,
        "CREATE TABLE Users (ID INT, Name TEXT); INSERT INTO users VALUES (1, 'ada'); SELECT * FROM users;",
        0,
        "ID|Name\n1|ada\n",
        "",
    );

    cleanup(&path);
}

#[test]
fn unsupported_sql_reports_exact_statement() {
    let path = temp_db_path("unsupported_sql_reports_exact_statement");

    assert_exec(
        &path,
        "SELECT id FROM users;",
        2,
        "",
        "error: unsupported SQL statement: SELECT id FROM users;\nhint: supported SQL subset: CREATE TABLE, INSERT INTO ... VALUES, SELECT * FROM ..., SELECT * FROM ... WHERE <primary_key> = <int>;\n",
    );

    cleanup(&path);
}

#[test]
fn primary_key_where_on_missing_table_reports_table_not_found() {
    let path = temp_db_path("primary_key_where_on_missing_table_reports_table_not_found");

    assert_exec(
        &path,
        "SELECT * FROM users WHERE id = 1;",
        2,
        "",
        "error: SQL semantic error: table not found: users\nhint: create the table before INSERT or SELECT.\n",
    );

    cleanup(&path);
}

#[test]
fn malformed_sql_reports_exact_statement() {
    let path = temp_db_path("malformed_sql_reports_exact_statement");

    assert_exec(
        &path,
        "CREATE TABLE users id INT);",
        2,
        "",
        "error: malformed SQL statement: CREATE TABLE users id INT);\nhint: terminate each statement with ';' and use the documented SQL subset.\n",
    );

    cleanup(&path);
}

#[test]
fn malformed_select_shape_reports_exact_statement() {
    let path = temp_db_path("malformed_select_shape_reports_exact_statement");

    assert_exec(
        &path,
        "SELECT * users;",
        2,
        "",
        "error: malformed SQL statement: SELECT * users;\nhint: terminate each statement with ';' and use the documented SQL subset.\n",
    );

    cleanup(&path);
}

#[test]
fn malformed_select_trailing_token_reports_exact_statement() {
    let path = temp_db_path("malformed_select_trailing_token_reports_exact_statement");

    assert_exec(
        &path,
        "SELECT * FROM users extra;",
        2,
        "",
        "error: malformed SQL statement: SELECT * FROM users extra;\nhint: terminate each statement with ';' and use the documented SQL subset.\n",
    );

    cleanup(&path);
}

#[test]
fn semantic_failure_matrix_reports_exact_errors() {
    let cases = [
        (
            "duplicate_table",
            "CREATE TABLE users (id INT); CREATE TABLE users (name TEXT);",
            "error: SQL semantic error: table already exists: users\nhint: use a new table name for CREATE TABLE in this database.\n",
        ),
        (
            "duplicate_table_case_variant",
            "CREATE TABLE users (id INT); CREATE TABLE Users (name TEXT);",
            "error: SQL semantic error: table already exists: Users\nhint: use a new table name for CREATE TABLE in this database.\n",
        ),
        (
            "missing_table_insert",
            "INSERT INTO missing VALUES (1);",
            "error: SQL semantic error: table not found: missing\nhint: create the table before INSERT or SELECT.\n",
        ),
        (
            "missing_table_select",
            "SELECT * FROM missing;",
            "error: SQL semantic error: table not found: missing\nhint: create the table before INSERT or SELECT.\n",
        ),
        (
            "duplicate_column",
            "CREATE TABLE users (id INT, id TEXT);",
            "error: SQL semantic error: duplicate column: id\nhint: column names in a table must be unique.\n",
        ),
        (
            "duplicate_column_case_variant",
            "CREATE TABLE users (id INT, ID TEXT);",
            "error: SQL semantic error: duplicate column: ID\nhint: column names in a table must be unique.\n",
        ),
        (
            "column_count_mismatch",
            "CREATE TABLE users (id INT, name TEXT); INSERT INTO users VALUES (1);",
            "error: SQL semantic error: column count mismatch for table users: expected 2 values, got 1\nhint: INSERT values must match the table schema exactly.\n",
        ),
        (
            "type_mismatch",
            "CREATE TABLE users (id INT); INSERT INTO users VALUES ('ada');",
            "error: SQL semantic error: type mismatch for column id: expected INT, got TEXT\nhint: INSERT values must match the declared column types.\n",
        ),
    ];

    for (name, sql, expected_stderr) in cases {
        let path = temp_db_path(name);
        assert_exec(&path, sql, 2, "", expected_stderr);
        cleanup(&path);
    }
}

#[test]
fn unknown_sql_storage_record_fails_deterministically() {
    let path = temp_db_path("unknown_sql_storage_record_fails_deterministically");
    append_fixture_record(&path, b"legacy");

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
fn sql_prefixed_duplicate_column_catalog_record_fails_deterministically() {
    let path = temp_db_path("sql_prefixed_duplicate_column_catalog_record_fails_deterministically");
    append_fixture_record(
        &path,
        &catalog_record("users", &[("id", b'I'), ("ID", b'T')]),
    );

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
fn sql_prefixed_output_breaking_text_row_record_fails_deterministically() {
    let path = temp_db_path("sql_prefixed_output_breaking_text_row_record_fails_deterministically");
    append_fixture_record(&path, &catalog_record("users", &[("name", b'T')]));
    append_fixture_record(&path, &row_record("users", &[(b'T', "a|b\n")]));

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
fn sql_prefixed_noncanonical_int_row_record_fails_deterministically() {
    let path = temp_db_path("sql_prefixed_noncanonical_int_row_record_fails_deterministically");
    append_fixture_record(&path, &catalog_record("nums", &[("id", b'I')]));
    append_fixture_record(&path, &row_record("nums", &[(b'I', "01")]));

    assert_exec(
        &path,
        "SELECT * FROM nums;",
        1,
        "",
        INVALID_SQL_STORAGE_STDERR,
    );

    cleanup(&path);
}

#[test]
fn primary_key_exact_lookup_outputs_matching_row_after_reopen() {
    let path = temp_db_path("primary_key_exact_lookup_outputs_matching_row_after_reopen");

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

    cleanup(&path);
}

#[test]
fn primary_key_select_all_scans_in_key_order_not_insert_order() {
    let path = temp_db_path("primary_key_select_all_scans_in_key_order_not_insert_order");

    assert_exec(
        &path,
        "CREATE TABLE users (id INT PRIMARY KEY, name TEXT); INSERT INTO users VALUES (2, 'bea'); INSERT INTO users VALUES (1, 'ada'); INSERT INTO users VALUES (3, 'cal'); SELECT * FROM users;",
        0,
        "id|name\n1|ada\n2|bea\n3|cal\n",
        "",
    );

    cleanup(&path);
}

#[test]
fn primary_key_missing_lookup_outputs_header_only() {
    let path = temp_db_path("primary_key_missing_lookup_outputs_header_only");

    assert_exec(
        &path,
        "CREATE TABLE users (id INT PRIMARY KEY, name TEXT); INSERT INTO users VALUES (2, 'bea'); SELECT * FROM users WHERE id = 9;",
        0,
        "id|name\n",
        "",
    );

    cleanup(&path);
}

#[test]
fn primary_key_duplicate_insert_fails_before_appending_row() {
    let path = temp_db_path("primary_key_duplicate_insert_fails_before_appending_row");

    assert_exec(
        &path,
        "CREATE TABLE users (id INT PRIMARY KEY, name TEXT); INSERT INTO users VALUES (2, 'bea'); INSERT INTO users VALUES (2, 'dupe');",
        2,
        "",
        "error: SQL semantic error: duplicate primary key for table users: 2\nhint: primary key values must be unique.\n",
    );
    assert_exec(&path, "SELECT * FROM users;", 0, "id|name\n2|bea\n", "");

    cleanup(&path);
}

#[test]
fn primary_key_empty_table_scan_outputs_header_only() {
    let path = temp_db_path("primary_key_empty_table_scan_outputs_header_only");

    assert_exec(
        &path,
        "CREATE TABLE empty_users (id INT PRIMARY KEY, name TEXT); SELECT * FROM empty_users;",
        0,
        "id|name\n",
        "",
    );

    cleanup(&path);
}

#[test]
fn primary_key_non_pk_table_preserves_insert_order() {
    let path = temp_db_path("primary_key_non_pk_table_preserves_insert_order");

    assert_exec(
        &path,
        "CREATE TABLE users (id INT, name TEXT); INSERT INTO users VALUES (2, 'bea'); INSERT INTO users VALUES (1, 'ada'); SELECT * FROM users;",
        0,
        "id|name\n2|bea\n1|ada\n",
        "",
    );

    cleanup(&path);
}

#[test]
fn primary_key_rejects_text_primary_key_declaration() {
    let path = temp_db_path("primary_key_rejects_text_primary_key_declaration");

    assert_exec(
        &path,
        "CREATE TABLE users (id TEXT PRIMARY KEY, name TEXT);",
        2,
        "",
        "error: SQL semantic error: primary key column must be INT: id\nhint: this SQL slice supports one INT PRIMARY KEY column per table.\n",
    );

    cleanup(&path);
}

#[test]
fn primary_key_rejects_non_primary_key_where_without_full_scan() {
    let path = temp_db_path("primary_key_rejects_non_primary_key_where_without_full_scan");

    assert_exec(
        &path,
        "CREATE TABLE users (id INT PRIMARY KEY, name TEXT); INSERT INTO users VALUES (1, 'ada'); INSERT INTO users VALUES (2, 'bea');",
        0,
        "",
        "",
    );
    assert_rejected_without_stdout(&path, "SELECT * FROM users WHERE name = 'ada';");

    cleanup(&path);
}

#[test]
fn primary_key_rejects_range_predicate_without_full_scan() {
    let path = temp_db_path("primary_key_rejects_range_predicate_without_full_scan");

    assert_exec(
        &path,
        "CREATE TABLE users (id INT PRIMARY KEY, name TEXT); INSERT INTO users VALUES (1, 'ada'); INSERT INTO users VALUES (2, 'bea');",
        0,
        "",
        "",
    );
    assert_rejected_without_stdout(&path, "SELECT * FROM users WHERE id > 1;");

    cleanup(&path);
}

#[test]
fn primary_key_rejects_order_by_without_full_scan() {
    let path = temp_db_path("primary_key_rejects_order_by_without_full_scan");

    assert_exec(
        &path,
        "CREATE TABLE users (id INT PRIMARY KEY, name TEXT); INSERT INTO users VALUES (1, 'ada'); INSERT INTO users VALUES (2, 'bea');",
        0,
        "",
        "",
    );
    assert_rejected_without_stdout(&path, "SELECT * FROM users ORDER BY id;");

    cleanup(&path);
}
