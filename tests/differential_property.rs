use rusqlite::{params, Connection, ErrorCode};
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const DEFAULT_SEEDS: &[u64] = &[1, 42, 0x5eed_2026];
const MIN_OPERATIONS_PER_SEED: usize = 100;
const MIN_SUCCESSFUL_ROWS_PER_SEED: usize = 25;

#[derive(Debug, Clone, PartialEq, Eq)]
enum Operation {
    CreateTable,
    Insert { id: i64, value: String },
    DuplicateInsert { id: i64, value: String },
    SelectAll,
    SelectById { id: i64 },
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Row {
    id: i64,
    value: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Observation {
    Ok { rows: Vec<Row> },
    Err { kind: ErrorKind },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ErrorKind {
    DuplicatePrimaryKey,
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Mismatch {
    operation_index: usize,
    operation: Operation,
    expected: Observation,
    actual: Observation,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct MismatchSignature {
    operation_class: OperationClass,
    expected_kind: ObservationKind,
    actual_kind: ObservationKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OperationClass {
    CreateTable,
    Insert,
    DuplicateInsert,
    SelectAll,
    SelectById,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ObservationKind {
    OkRows,
    DuplicatePrimaryKeyError,
    OtherError,
}

impl Operation {
    fn class(&self) -> OperationClass {
        match self {
            Operation::CreateTable => OperationClass::CreateTable,
            Operation::Insert { .. } => OperationClass::Insert,
            Operation::DuplicateInsert { .. } => OperationClass::DuplicateInsert,
            Operation::SelectAll => OperationClass::SelectAll,
            Operation::SelectById { .. } => OperationClass::SelectById,
        }
    }
}

impl Observation {
    fn kind(&self) -> ObservationKind {
        match self {
            Observation::Ok { .. } => ObservationKind::OkRows,
            Observation::Err {
                kind: ErrorKind::DuplicatePrimaryKey,
            } => ObservationKind::DuplicatePrimaryKeyError,
            Observation::Err {
                kind: ErrorKind::Other,
            } => ObservationKind::OtherError,
        }
    }
}

impl Mismatch {
    fn signature(&self) -> MismatchSignature {
        MismatchSignature {
            operation_class: self.operation.class(),
            expected_kind: self.expected.kind(),
            actual_kind: self.actual.kind(),
        }
    }
}

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

fn exec_sql(path: &Path, sql: &str) -> Output {
    db(&[
        "exec",
        path.to_str().expect("temp path should be UTF-8"),
        sql,
    ])
}

fn temp_db_path(seed: u64, label: &str) -> PathBuf {
    let mut dir = std::env::temp_dir();
    dir.push(format!(
        "persistent_db_core_differential_property_{seed}_{label}_{}_{}",
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

#[test]
fn deterministic_sql_subset_matches_sqlite_oracle() {
    let seed_override = std::env::var("PDB_DIFF_SEED")
        .ok()
        .map(|value| value.parse::<u64>().expect("PDB_DIFF_SEED must be u64"));
    let prefix_override = std::env::var("PDB_DIFF_PREFIX").ok().map(|value| {
        value
            .parse::<usize>()
            .expect("PDB_DIFF_PREFIX must be usize")
    });

    let seeds: Vec<u64> = seed_override
        .map(|seed| vec![seed])
        .unwrap_or_else(|| DEFAULT_SEEDS.to_vec());

    for seed in seeds {
        let operations = generate_operations(seed);
        if prefix_override.is_none() {
            assert_generator_coverage(seed, &operations);
        }

        let prefix_len = prefix_override
            .unwrap_or(operations.len())
            .min(operations.len());
        let selected = &operations[..prefix_len];

        if let Err(mismatch) = run_sequence(seed, selected, "main") {
            report_failure(seed, selected, mismatch);
        }
    }
}

fn assert_generator_coverage(seed: u64, operations: &[Operation]) {
    assert!(
        operations.len() >= MIN_OPERATIONS_PER_SEED,
        "seed {seed} generated {} operations, expected at least {MIN_OPERATIONS_PER_SEED}",
        operations.len()
    );

    let successful_rows = operations
        .iter()
        .filter(|operation| matches!(operation, Operation::Insert { .. }))
        .count();
    assert!(
        successful_rows >= MIN_SUCCESSFUL_ROWS_PER_SEED,
        "seed {seed} generated {successful_rows} unique rows, expected at least {MIN_SUCCESSFUL_ROWS_PER_SEED}"
    );

    assert!(
        matches!(operations.first(), Some(Operation::CreateTable)),
        "seed {seed} must start with CREATE TABLE"
    );
    assert!(
        operations
            .iter()
            .any(|operation| matches!(operation, Operation::DuplicateInsert { .. })),
        "seed {seed} must include duplicate primary-key coverage"
    );
    assert!(
        operations
            .iter()
            .any(|operation| matches!(operation, Operation::SelectById { id } if *id == missing_key(seed))),
        "seed {seed} must include missing key lookup coverage"
    );
    assert!(
        operations
            .iter()
            .any(|operation| matches!(operation, Operation::SelectAll)),
        "seed {seed} must include ordered scan coverage"
    );
}

fn generate_operations(seed: u64) -> Vec<Operation> {
    let mut rng = DeterministicRng::new(seed);
    let mut operations = vec![
        Operation::CreateTable,
        Operation::SelectAll,
        Operation::SelectById {
            id: missing_key(seed),
        },
    ];
    let mut inserted_ids = Vec::new();
    let mut used_ids = BTreeSet::new();

    for row_index in 0..MIN_SUCCESSFUL_ROWS_PER_SEED {
        let id = unique_key(&mut rng, &mut used_ids);
        inserted_ids.push(id);
        operations.push(Operation::Insert {
            id,
            value: ascii_value(&mut rng, row_index),
        });

        if row_index == 0 {
            operations.push(Operation::DuplicateInsert {
                id,
                value: ascii_value(&mut rng, 10_000 + row_index),
            });
        }
        if row_index % 5 == 0 {
            operations.push(Operation::SelectById { id });
        }
        if row_index % 7 == 0 {
            operations.push(Operation::SelectAll);
        }
    }

    while operations.len() < MIN_OPERATIONS_PER_SEED {
        match rng.next_u64() % 6 {
            0 | 1 => {
                let id = unique_key(&mut rng, &mut used_ids);
                inserted_ids.push(id);
                operations.push(Operation::Insert {
                    id,
                    value: ascii_value(&mut rng, operations.len()),
                });
            }
            2 => {
                let id = inserted_ids[(rng.next_u64() as usize) % inserted_ids.len()];
                operations.push(Operation::DuplicateInsert {
                    id,
                    value: ascii_value(&mut rng, operations.len()),
                });
            }
            3 => {
                let id = inserted_ids[(rng.next_u64() as usize) % inserted_ids.len()];
                operations.push(Operation::SelectById { id });
            }
            4 => operations.push(Operation::SelectById {
                id: missing_key(seed),
            }),
            _ => operations.push(Operation::SelectAll),
        }
    }

    operations.push(Operation::SelectAll);
    operations
}

fn missing_key(seed: u64) -> i64 {
    i64::MIN + (seed % 1_000_000) as i64
}

fn unique_key(rng: &mut DeterministicRng, used_ids: &mut BTreeSet<i64>) -> i64 {
    loop {
        let candidate = ((rng.next_u64() % 2_000_000) as i64) - 1_000_000;
        if candidate != missing_key(rng.seed) && used_ids.insert(candidate) {
            return candidate;
        }
    }
}

fn ascii_value(rng: &mut DeterministicRng, index: usize) -> String {
    let len = 5 + (rng.next_u64() % 8) as usize;
    let mut value = format!("v{index}_");
    for _ in 0..len {
        let ch = (b'a' + (rng.next_u64() % 26) as u8) as char;
        value.push(ch);
    }
    value
}

fn run_sequence(seed: u64, operations: &[Operation], label: &str) -> Result<(), Mismatch> {
    let db_path = temp_db_path(seed, label);
    let sqlite = Connection::open_in_memory().expect("in-memory SQLite oracle should open");

    for (index, operation) in operations.iter().enumerate() {
        let expected = execute_sqlite(&sqlite, operation);
        let actual = execute_db(&db_path, operation);
        if matches!(operation, Operation::DuplicateInsert { .. }) {
            assert_eq!(
                ObservationKind::DuplicatePrimaryKeyError,
                expected.kind(),
                "SQLite duplicate insert must fail with duplicate primary-key class"
            );
            assert_eq!(
                ObservationKind::DuplicatePrimaryKeyError,
                actual.kind(),
                "db duplicate insert must fail with duplicate primary-key class"
            );
        }
        if expected != actual {
            cleanup(&db_path);
            return Err(Mismatch {
                operation_index: index,
                operation: operation.clone(),
                expected,
                actual,
            });
        }
    }

    cleanup(&db_path);
    Ok(())
}

fn execute_sqlite(sqlite: &Connection, operation: &Operation) -> Observation {
    match operation {
        Operation::CreateTable => sqlite
            .execute("CREATE TABLE kv (id INTEGER PRIMARY KEY, value TEXT)", [])
            .map(|_| Observation::Ok { rows: Vec::new() })
            .unwrap_or(Observation::Err {
                kind: ErrorKind::Other,
            }),
        Operation::Insert { id, value } | Operation::DuplicateInsert { id, value } => sqlite
            .execute(
                "INSERT INTO kv (id, value) VALUES (?, ?)",
                params![id, value],
            )
            .map(|_| Observation::Ok { rows: Vec::new() })
            .unwrap_or_else(sqlite_error_observation),
        Operation::SelectAll => {
            let mut statement = sqlite
                .prepare("SELECT id, value FROM kv ORDER BY id")
                .expect("SQLite SELECT * oracle should prepare");
            let rows = statement
                .query_map([], |row| {
                    Ok(Row {
                        id: row.get(0)?,
                        value: row.get(1)?,
                    })
                })
                .expect("SQLite SELECT * oracle should run")
                .collect::<Result<Vec<_>, _>>()
                .expect("SQLite rows should decode");
            Observation::Ok { rows }
        }
        Operation::SelectById { id } => {
            let mut statement = sqlite
                .prepare("SELECT id, value FROM kv WHERE id = ? ORDER BY id")
                .expect("SQLite primary-key lookup oracle should prepare");
            let rows = statement
                .query_map(params![id], |row| {
                    Ok(Row {
                        id: row.get(0)?,
                        value: row.get(1)?,
                    })
                })
                .expect("SQLite primary-key lookup oracle should run")
                .collect::<Result<Vec<_>, _>>()
                .expect("SQLite rows should decode");
            Observation::Ok { rows }
        }
    }
}

fn sqlite_error_observation(error: rusqlite::Error) -> Observation {
    let kind = match error {
        rusqlite::Error::SqliteFailure(ref failure, _)
            if failure.code == ErrorCode::ConstraintViolation =>
        {
            ErrorKind::DuplicatePrimaryKey
        }
        _ => ErrorKind::Other,
    };
    Observation::Err { kind }
}

fn execute_db(path: &Path, operation: &Operation) -> Observation {
    let sql = match operation {
        Operation::CreateTable => "CREATE TABLE kv (id INT PRIMARY KEY, value TEXT);".to_string(),
        Operation::Insert { id, value } | Operation::DuplicateInsert { id, value } => {
            format!("INSERT INTO kv VALUES ({id}, '{}');", value)
        }
        Operation::SelectAll => "SELECT * FROM kv;".to_string(),
        Operation::SelectById { id } => format!("SELECT * FROM kv WHERE id = {id};"),
    };

    let output = exec_sql(path, &sql);
    if !output.status.success() {
        assert_eq!("", stdout(&output), "db error stdout must be empty");
        let err = stderr(&output);
        assert!(
            err.starts_with("error: "),
            "db error stderr must be deterministic, got {:?}",
            err
        );
        let kind = if err.starts_with("error: SQL semantic error: duplicate primary key ") {
            ErrorKind::DuplicatePrimaryKey
        } else {
            ErrorKind::Other
        };
        return Observation::Err { kind };
    }

    let out = stdout(&output);
    match operation {
        Operation::SelectAll | Operation::SelectById { .. } => Observation::Ok {
            rows: parse_db_rows(&out),
        },
        _ => {
            assert_eq!("", out, "mutation stdout must be empty");
            Observation::Ok { rows: Vec::new() }
        }
    }
}

fn parse_db_rows(output: &str) -> Vec<Row> {
    let mut lines = output.lines();
    assert_eq!(
        Some("id|value"),
        lines.next(),
        "db SELECT output must start with the kv header"
    );
    lines
        .map(|line| {
            let (id, value) = line
                .split_once('|')
                .expect("db row must contain id and value separated by |");
            Row {
                id: id.parse().expect("db id column must be i64"),
                value: value.to_string(),
            }
        })
        .collect()
}

fn report_failure(seed: u64, operations: &[Operation], mismatch: Mismatch) -> ! {
    let (minimal_prefix, reproduced_mismatch) = minimal_failing_prefix(seed, operations, mismatch);
    let artifact_path =
        write_failure_artifact(seed, operations, minimal_prefix, &reproduced_mismatch);
    let replay_command =
        format!("PDB_DIFF_SEED={seed} PDB_DIFF_PREFIX={minimal_prefix} cargo test --test differential_property -- --nocapture");

    println!("differential/property failure");
    println!("seed: {seed}");
    println!(
        "failing operation index: {}",
        reproduced_mismatch.operation_index
    );
    println!("failing operation: {:?}", reproduced_mismatch.operation);
    println!("mismatch signature: {:?}", reproduced_mismatch.signature());
    println!("minimal reproducible operation prefix: {minimal_prefix}");
    println!("reproducible sequence:");
    for (index, operation) in operations.iter().take(minimal_prefix).enumerate() {
        println!("  {index}: {operation:?}");
    }
    println!("SQLite expected rows: {:?}", reproduced_mismatch.expected);
    println!("db actual rows: {:?}", reproduced_mismatch.actual);
    println!("failure artifact: {}", artifact_path.display());
    println!("rerun command: {replay_command}");

    panic!("SQLite differential/property mismatch for seed {seed}");
}

fn minimal_failing_prefix(
    seed: u64,
    operations: &[Operation],
    mismatch: Mismatch,
) -> (usize, Mismatch) {
    let upper_bound = mismatch.operation_index + 1;
    for prefix in 1..=upper_bound {
        if let Err(replayed) = run_sequence(seed, &operations[..prefix], &format!("min_{prefix}")) {
            if replayed.operation_index + 1 == prefix
                && replayed.operation == mismatch.operation
                && replayed.expected == mismatch.expected
                && replayed.actual == mismatch.actual
            {
                return (prefix, replayed);
            }
        }
    }
    (upper_bound, mismatch)
}

fn write_failure_artifact(
    seed: u64,
    operations: &[Operation],
    minimal_prefix: usize,
    mismatch: &Mismatch,
) -> PathBuf {
    let path = PathBuf::from("target")
        .join("differential_property")
        .join("failures")
        .join(format!("{seed}.json"));
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("failure artifact dir should be created");
    }

    let mut json = String::new();
    json.push_str("{\n");
    json.push_str(&format!("  \"seed\": {seed},\n"));
    json.push_str(&format!(
        "  \"failing_operation_index\": {},\n",
        mismatch.operation_index
    ));
    json.push_str(&format!("  \"minimal_prefix\": {minimal_prefix},\n"));
    json.push_str(&format!(
        "  \"mismatch_signature\": \"{}\",\n",
        json_escape(&format!("{:?}", mismatch.signature()))
    ));
    json.push_str(&format!(
        "  \"rerun_command\": \"{}\",\n",
        json_escape(&format!(
            "PDB_DIFF_SEED={seed} PDB_DIFF_PREFIX={minimal_prefix} cargo test --test differential_property -- --nocapture"
        ))
    ));
    json.push_str("  \"operations\": [\n");
    for (index, operation) in operations.iter().take(minimal_prefix).enumerate() {
        let comma = if index + 1 == minimal_prefix { "" } else { "," };
        json.push_str(&format!(
            "    \"{}\"{comma}\n",
            json_escape(&format!("{operation:?}"))
        ));
    }
    json.push_str("  ],\n");
    json.push_str(&format!(
        "  \"sqlite_expected_rows\": \"{}\",\n",
        json_escape(&format!("{:?}", mismatch.expected))
    ));
    json.push_str(&format!(
        "  \"db_actual_rows\": \"{}\"\n",
        json_escape(&format!("{:?}", mismatch.actual))
    ));
    json.push_str("}\n");
    fs::write(&path, json).expect("failure artifact should be written");
    path
}

fn json_escape(value: &str) -> String {
    value
        .chars()
        .flat_map(|ch| match ch {
            '"' => "\\\"".chars().collect::<Vec<_>>(),
            '\\' => "\\\\".chars().collect(),
            '\n' => "\\n".chars().collect(),
            '\r' => "\\r".chars().collect(),
            '\t' => "\\t".chars().collect(),
            _ => vec![ch],
        })
        .collect()
}

struct DeterministicRng {
    seed: u64,
    state: u64,
}

impl DeterministicRng {
    fn new(seed: u64) -> Self {
        Self {
            seed,
            state: seed ^ 0x9e37_79b9_7f4a_7c15,
        }
    }

    fn next_u64(&mut self) -> u64 {
        self.state = self
            .state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        self.state
    }
}
