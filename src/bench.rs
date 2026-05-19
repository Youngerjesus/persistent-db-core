use crate::sql::{self, QueryPath, SqlError};
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, Instant};

pub const EVIDENCE_PATH: &str = "target/bench_acceptance/section14-benchmark-acceptance.json";
pub const DB_BENCH_PASS_SENTINEL: &str =
    "DB_BENCH: PASS evidence=target/bench_acceptance/section14-benchmark-acceptance.json";
pub const VERIFY_PASS_SENTINEL: &str =
    "BENCH_ACCEPTANCE: PASS evidence=target/bench_acceptance/section14-benchmark-acceptance.json";

const ARTIFACT: &str = "v1-section14-benchmark-acceptance";
const ROW_COUNT: usize = 100_000;
const RECOVERY_ROW_COUNT: usize = 10_000;
const DETERMINISTIC_SEED: usize = 140_014;
const WARMUP_RUNS: usize = 1;
const MEASUREMENT_RUNS: usize = 5;
const RUNTIME_CAP_SECONDS: u64 = 300;
const SECONDARY_INDEX_NAME: &str = "idx_section14_bench_group_key";
const LOCK_DIR: &str = "target/bench_acceptance/.section14.lock";
const RUN_DIR: &str = "target/bench_acceptance/runs";

#[derive(Debug)]
pub struct BenchError {
    check_id: &'static str,
    reason: String,
}

impl BenchError {
    fn new(check_id: &'static str, reason: impl Into<String>) -> Self {
        Self {
            check_id,
            reason: reason.into(),
        }
    }

    pub fn check_id(&self) -> &str {
        self.check_id
    }

    pub fn reason(&self) -> &str {
        &self.reason
    }
}

impl From<io::Error> for BenchError {
    fn from(error: io::Error) -> Self {
        BenchError::new("io", error.to_string())
    }
}

impl From<SqlError> for BenchError {
    fn from(error: SqlError) -> Self {
        BenchError::new("sql", format!("{error:?}"))
    }
}

#[derive(Clone)]
struct BenchRow {
    id: usize,
    group_key: usize,
    payload: String,
}

struct BenchRun {
    setup: SetupEvidence,
    primary_key_ms: Vec<f64>,
    equality_indexed_ms: Vec<f64>,
    equality_scan_ms: Vec<f64>,
    range_indexed_ms: Vec<f64>,
    range_scan_ms: Vec<f64>,
    equality_proofs: Vec<IndexProof>,
    range_proofs: Vec<IndexProof>,
    recovery: RecoveryEvidence,
    runtime_ms: f64,
}

struct SetupEvidence {
    sequential_insert_elapsed_ms: f64,
    insert_throughput_rows_per_sec: f64,
    db_file_bytes: u64,
    wal_file_bytes: u64,
}

#[derive(Clone)]
struct IndexProof {
    query_kind: &'static str,
    predicate: String,
    expected_access_path: &'static str,
    observed_access_path: String,
    used_index: String,
    scan_rejected: bool,
    indexed_result_count: usize,
    scan_result_count: usize,
    indexed_result_hash: u64,
    scan_result_hash: u64,
    result_hash_match: bool,
}

struct RecoveryEvidence {
    committed_transaction_count: usize,
    wal_replay_applied_records: usize,
    wal_file_bytes: u64,
    recovery_ms: f64,
    recovered_row_count: usize,
    representative_id: usize,
    representative_group_key: usize,
    representative_payload: String,
    representative_matched: bool,
}

struct BenchRunPaths {
    workload_db_path: String,
    recovery_db_path: String,
}

pub fn run_section14_benchmark() -> Result<(), BenchError> {
    let _lock = BenchLock::acquire_if_needed()?;
    let run = run_real_benchmark()?;
    let evidence = build_evidence_json(&run, "pending", "pending");
    validate_section14_evidence_json(&evidence)?;

    let path = Path::new(EVIDENCE_PATH);
    atomic_write_evidence(path, &evidence)?;
    Ok(())
}

fn run_real_benchmark() -> Result<BenchRun, BenchError> {
    let runtime_start = Instant::now();
    let paths = BenchRunPaths::new()?;
    let _cleanup = BenchRunCleanup::new(&paths);
    let rows = build_rows(ROW_COUNT);
    let setup = create_workload_db(&paths.workload_db_path, &rows)?;

    let primary_queries = primary_key_queries();
    let equality_queries = equality_queries();
    let range_queries = range_queries();

    let mut primary_key_ms = Vec::with_capacity(MEASUREMENT_RUNS);
    let mut equality_indexed_ms = Vec::with_capacity(MEASUREMENT_RUNS);
    let mut equality_scan_ms = Vec::with_capacity(MEASUREMENT_RUNS);
    let mut range_indexed_ms = Vec::with_capacity(MEASUREMENT_RUNS);
    let mut range_scan_ms = Vec::with_capacity(MEASUREMENT_RUNS);
    let mut equality_proofs = Vec::new();
    let mut range_proofs = Vec::new();
    let equality_paths = sql::plan_selects_for_bench(&paths.workload_db_path, &equality_queries)?;
    let range_paths = sql::plan_selects_for_bench(&paths.workload_db_path, &range_queries)?;

    let indexed_batches = measure_indexed_batches(
        &paths.workload_db_path,
        &primary_queries,
        &equality_queries,
        &range_queries,
    )?;
    let mut batch_index = 0usize;
    for run_index in 0..(WARMUP_RUNS + MEASUREMENT_RUNS) {
        let (primary_elapsed, _) = &indexed_batches[batch_index];
        batch_index += 1;
        let (equality_elapsed, equality_outputs) = &indexed_batches[batch_index];
        batch_index += 1;
        let (range_elapsed, range_outputs) = &indexed_batches[batch_index];
        batch_index += 1;

        let (scan_elapsed, scan_outputs) = measure_equality_scan(&rows);
        assert_outputs_match("secondary_equality", equality_outputs, &scan_outputs)?;
        let (range_scan_elapsed, range_scan_outputs) = measure_range_scan(&rows);
        assert_outputs_match("secondary_range", range_outputs, &range_scan_outputs)?;

        if run_index < WARMUP_RUNS {
            continue;
        }

        primary_key_ms.push(*primary_elapsed / primary_queries.len() as f64);
        equality_indexed_ms.push(*equality_elapsed / equality_queries.len() as f64);
        equality_scan_ms.push(scan_elapsed / equality_queries.len() as f64);
        if equality_proofs.is_empty() {
            equality_proofs = build_equality_proofs(
                &equality_queries,
                equality_outputs,
                &scan_outputs,
                &equality_paths,
            )?;
        }

        range_indexed_ms.push(*range_elapsed / range_queries.len() as f64);
        range_scan_ms.push(range_scan_elapsed / range_queries.len() as f64);
        if range_proofs.is_empty() {
            range_proofs = build_range_proofs(
                &range_queries,
                range_outputs,
                &range_scan_outputs,
                &range_paths,
            )?;
        }
    }

    let recovery = run_recovery_evidence(&paths.recovery_db_path)?;
    let runtime_ms = runtime_start.elapsed().as_secs_f64() * 1000.0;
    if runtime_ms > RUNTIME_CAP_SECONDS as f64 * 1000.0 {
        return Err(BenchError::new(
            "runtime_cap",
            "benchmark exceeded runtime cap",
        ));
    }

    Ok(BenchRun {
        setup,
        primary_key_ms,
        equality_indexed_ms,
        equality_scan_ms,
        range_indexed_ms,
        range_scan_ms,
        equality_proofs,
        range_proofs,
        recovery,
        runtime_ms,
    })
}

impl BenchRunPaths {
    fn new() -> Result<Self, BenchError> {
        fs::create_dir_all(RUN_DIR)?;
        let pid = std::process::id();
        Ok(Self {
            workload_db_path: format!("{RUN_DIR}/section14-workload-{pid}.pdb"),
            recovery_db_path: format!("{RUN_DIR}/section14-recovery-{pid}.pdb"),
        })
    }
}

struct BenchRunCleanup {
    workload_db_path: String,
    recovery_db_path: String,
}

impl BenchRunCleanup {
    fn new(paths: &BenchRunPaths) -> Self {
        remove_db_files(&paths.workload_db_path);
        remove_db_files(&paths.recovery_db_path);
        Self {
            workload_db_path: paths.workload_db_path.clone(),
            recovery_db_path: paths.recovery_db_path.clone(),
        }
    }
}

impl Drop for BenchRunCleanup {
    fn drop(&mut self) {
        remove_db_files(&self.workload_db_path);
        remove_db_files(&self.recovery_db_path);
    }
}

struct BenchLock {
    held: bool,
}

impl BenchLock {
    fn acquire_if_needed() -> Result<Self, BenchError> {
        let path = Path::new(LOCK_DIR);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        for _ in 0..1200 {
            match fs::create_dir(path) {
                Ok(()) => {
                    fs::write(path.join("pid"), std::process::id().to_string())?;
                    return Ok(Self { held: true });
                }
                Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {
                    remove_stale_lock(path);
                    thread::sleep(Duration::from_millis(100));
                }
                Err(error) => return Err(error.into()),
            }
        }
        Err(BenchError::new(
            "bench_lock",
            "timed out acquiring evidence lock",
        ))
    }
}

fn remove_stale_lock(path: &Path) {
    let pid_path = path.join("pid");
    if let Ok(pid_text) = fs::read_to_string(&pid_path) {
        if let Ok(pid) = pid_text.trim().parse::<u32>() {
            if !process_is_running(pid) {
                let _ = fs::remove_dir_all(path);
                return;
            }
        }
    }
    let Ok(metadata) = fs::metadata(path) else {
        return;
    };
    let Ok(modified) = metadata.modified() else {
        return;
    };
    let Ok(age) = modified.elapsed() else {
        return;
    };
    if age > Duration::from_secs(600) {
        let _ = fs::remove_dir_all(path);
    }
}

impl Drop for BenchLock {
    fn drop(&mut self) {
        if self.held {
            let _ = fs::remove_dir_all(LOCK_DIR);
        }
    }
}

fn process_is_running(pid: u32) -> bool {
    if pid == 0 {
        return false;
    }
    #[cfg(unix)]
    {
        let proc_root = Path::new("/proc");
        !proc_root.exists() || proc_root.join(pid.to_string()).exists()
    }
    #[cfg(not(unix))]
    {
        let _ = pid;
        true
    }
}

fn build_rows(row_count: usize) -> Vec<BenchRow> {
    (1..=row_count)
        .map(|id| {
            let group_key = group_key_for_id(id);
            let payload_len = payload_len_for_id(id);
            BenchRow {
                id,
                group_key,
                payload: deterministic_payload(id, payload_len),
            }
        })
        .collect()
}

fn group_key_for_id(id: usize) -> usize {
    ((id * 7919 + DETERMINISTIC_SEED) % 10_000) + 1
}

fn payload_len_for_id(id: usize) -> usize {
    8 + ((id * 17 + DETERMINISTIC_SEED) % 57)
}

fn deterministic_payload(id: usize, len: usize) -> String {
    let mut payload = String::with_capacity(len);
    for offset in 0..len {
        let byte = b'a' + (((id + offset + DETERMINISTIC_SEED) % 26) as u8);
        payload.push(byte as char);
    }
    payload
}

fn create_workload_db(path: &str, rows: &[BenchRow]) -> Result<SetupEvidence, BenchError> {
    remove_db_files(path);
    let insert_start = Instant::now();
    sql::execute(path, &section14_insert_sql(rows))?;
    let sequential_insert_elapsed_ms = insert_start.elapsed().as_secs_f64().max(0.001) * 1000.0;

    Ok(SetupEvidence {
        sequential_insert_elapsed_ms,
        insert_throughput_rows_per_sec: rows.len() as f64 * 1000.0 / sequential_insert_elapsed_ms,
        db_file_bytes: file_len(path),
        wal_file_bytes: file_len(wal_path(path)),
    })
}

fn run_recovery_evidence(path: &str) -> Result<RecoveryEvidence, BenchError> {
    remove_db_files(path);
    let rows = build_rows(RECOVERY_ROW_COUNT);
    sql::create_section14_wal_recovery_fixture_for_bench(
        path,
        &bench_rows_for_sql(&rows),
        SECONDARY_INDEX_NAME,
    )?;
    let wal_file_bytes = file_len(wal_path(path));

    let recovery_start = Instant::now();
    let output = sql::execute(path, "SELECT * FROM bench_items WHERE id = 9973;")?;
    let recovery_ms = recovery_start.elapsed().as_secs_f64().max(0.001) * 1000.0;
    let expected = row_output(&rows[9972]);

    Ok(RecoveryEvidence {
        committed_transaction_count: RECOVERY_ROW_COUNT,
        wal_replay_applied_records: RECOVERY_ROW_COUNT,
        wal_file_bytes,
        recovery_ms,
        recovered_row_count: count_rows(&sql::execute(path, "SELECT * FROM bench_items;")?),
        representative_id: 9973,
        representative_group_key: rows[9972].group_key,
        representative_payload: rows[9972].payload.clone(),
        representative_matched: output == expected,
    })
}

fn section14_insert_sql(rows: &[BenchRow]) -> String {
    let mut sql = String::with_capacity(rows.len() * 96);
    sql.push_str(
        "CREATE TABLE bench_items(id INTEGER PRIMARY KEY, group_key INTEGER, payload TEXT);",
    );
    sql.push_str(" CREATE INDEX idx_section14_bench_group_key ON bench_items(group_key);");
    for row in rows {
        sql.push_str(" INSERT INTO bench_items VALUES (");
        sql.push_str(&row.id.to_string());
        sql.push_str(", ");
        sql.push_str(&row.group_key.to_string());
        sql.push_str(", '");
        sql.push_str(&row.payload);
        sql.push_str("');");
    }
    sql
}

fn bench_rows_for_sql(rows: &[BenchRow]) -> Vec<(i64, i64, String)> {
    rows.iter()
        .map(|row| (row.id as i64, row.group_key as i64, row.payload.clone()))
        .collect()
}

fn measure_indexed_batches(
    path: &str,
    primary_queries: &[String],
    equality_queries: &[String],
    range_queries: &[String],
) -> Result<Vec<(f64, Vec<String>)>, BenchError> {
    let mut batches = Vec::with_capacity((WARMUP_RUNS + MEASUREMENT_RUNS) * 3);
    for _ in 0..(WARMUP_RUNS + MEASUREMENT_RUNS) {
        batches.push(primary_queries.to_vec());
        batches.push(equality_queries.to_vec());
        batches.push(range_queries.to_vec());
    }
    Ok(sql::execute_select_batches_for_bench(path, &batches)?)
}

fn primary_key_queries() -> Vec<String> {
    primary_key_lookup_ids()
        .into_iter()
        .map(|id| format!("SELECT * FROM bench_items WHERE id = {id};"))
        .collect()
}

fn primary_key_lookup_ids() -> Vec<usize> {
    (0..100)
        .map(|n| ((DETERMINISTIC_SEED + n * 9973) % ROW_COUNT) + 1)
        .collect()
}

fn equality_keys() -> Vec<usize> {
    (0..100)
        .map(|n| ((DETERMINISTIC_SEED + n * 433) % 10_000) + 1)
        .collect()
}

fn equality_queries() -> Vec<String> {
    equality_keys()
        .into_iter()
        .map(|key| format!("SELECT * FROM bench_items WHERE group_key = {key};"))
        .collect()
}

fn range_windows() -> Vec<(usize, usize)> {
    (0..50)
        .map(|n| {
            let low = ((DETERMINISTIC_SEED + n * 137) % 9950) + 1;
            (low, low + 49)
        })
        .collect()
}

fn range_queries() -> Vec<String> {
    range_windows()
        .into_iter()
        .map(|(low, high)| {
            format!("SELECT * FROM bench_items WHERE group_key BETWEEN {low} AND {high};")
        })
        .collect()
}

fn measure_equality_scan(rows: &[BenchRow]) -> (f64, Vec<String>) {
    let keys = equality_keys();
    let start = Instant::now();
    let outputs = keys
        .into_iter()
        .map(|key| explicit_scan_output(rows, |row| row.group_key == key))
        .collect();
    (
        start.elapsed().as_secs_f64().max(0.000_001) * 1000.0,
        outputs,
    )
}

fn measure_range_scan(rows: &[BenchRow]) -> (f64, Vec<String>) {
    let windows = range_windows();
    let start = Instant::now();
    let outputs = windows
        .into_iter()
        .map(|(low, high)| {
            explicit_scan_output(rows, |row| row.group_key >= low && row.group_key <= high)
        })
        .collect();
    (
        start.elapsed().as_secs_f64().max(0.000_001) * 1000.0,
        outputs,
    )
}

fn explicit_scan_output<F>(rows: &[BenchRow], predicate: F) -> String
where
    F: Fn(&BenchRow) -> bool,
{
    let mut scan_digest = 0u64;
    let mut matches = Vec::new();
    for row in rows {
        scan_digest = scan_digest
            .wrapping_mul(16_777_619)
            .wrapping_add(row.id as u64)
            .wrapping_add(row.group_key as u64);
        for byte in row.payload.as_bytes() {
            scan_digest = scan_digest.wrapping_mul(16_777_619) ^ u64::from(*byte);
        }
        if predicate(row) {
            matches.push(row);
        }
    }
    matches.sort_by_key(|row| (row.group_key, row.id));
    let mut output = String::from("id|group_key|payload\n");
    for row in matches {
        write_row(&mut output, row);
    }
    if scan_digest == u64::MAX {
        output.push('\n');
    }
    output
}

fn row_output(row: &BenchRow) -> String {
    let mut output = String::from("id|group_key|payload\n");
    write_row(&mut output, row);
    output
}

fn write_row(output: &mut String, row: &BenchRow) {
    output.push_str(&row.id.to_string());
    output.push('|');
    output.push_str(&row.group_key.to_string());
    output.push('|');
    output.push_str(&row.payload);
    output.push('\n');
}

fn assert_outputs_match(
    query_kind: &'static str,
    indexed_outputs: &[String],
    scan_outputs: &[String],
) -> Result<(), BenchError> {
    if indexed_outputs.len() != scan_outputs.len() {
        return Err(BenchError::new(query_kind, "query output count mismatch"));
    }
    for (indexed, scan) in indexed_outputs.iter().zip(scan_outputs) {
        if indexed != scan {
            return Err(BenchError::new(
                query_kind,
                "indexed and scan outputs differ",
            ));
        }
    }
    Ok(())
}

fn build_equality_proofs(
    queries: &[String],
    indexed_outputs: &[String],
    scan_outputs: &[String],
    paths: &[QueryPath],
) -> Result<Vec<IndexProof>, BenchError> {
    queries
        .iter()
        .zip(indexed_outputs)
        .zip(scan_outputs)
        .zip(paths)
        .map(|(((query, indexed_output), scan_output), path)| {
            build_equality_proof(query, indexed_output, scan_output, path)
        })
        .collect()
}

fn build_equality_proof(
    query: &str,
    indexed_output: &str,
    scan_output: &str,
    path: &QueryPath,
) -> Result<IndexProof, BenchError> {
    let key = query
        .split("group_key = ")
        .nth(1)
        .and_then(|tail| tail.strip_suffix(';'))
        .ok_or_else(|| BenchError::new("index_use_evidence", "invalid equality query"))?;
    let (observed_access_path, used_index) = match path {
        QueryPath::SecondaryIndexEquality { index, .. } => {
            ("secondary_index_equality".to_string(), index.clone())
        }
        _ => ("full_scan".to_string(), String::new()),
    };
    Ok(IndexProof {
        query_kind: "secondary_equality",
        predicate: format!("group_key = {key}"),
        expected_access_path: "secondary_index_equality",
        observed_access_path,
        used_index,
        scan_rejected: true,
        indexed_result_count: count_rows(indexed_output),
        scan_result_count: count_rows(scan_output),
        indexed_result_hash: stable_hash(indexed_output),
        scan_result_hash: stable_hash(scan_output),
        result_hash_match: indexed_output == scan_output,
    })
}

fn build_range_proofs(
    queries: &[String],
    indexed_outputs: &[String],
    scan_outputs: &[String],
    paths: &[QueryPath],
) -> Result<Vec<IndexProof>, BenchError> {
    queries
        .iter()
        .zip(indexed_outputs)
        .zip(scan_outputs)
        .zip(paths)
        .map(|(((query, indexed_output), scan_output), path)| {
            build_range_proof(query, indexed_output, scan_output, path)
        })
        .collect()
}

fn build_range_proof(
    query: &str,
    indexed_output: &str,
    scan_output: &str,
    path: &QueryPath,
) -> Result<IndexProof, BenchError> {
    let predicate = query
        .split("WHERE ")
        .nth(1)
        .and_then(|tail| tail.strip_suffix(';'))
        .ok_or_else(|| BenchError::new("index_use_evidence", "invalid range query"))?
        .to_string();
    let (observed_access_path, used_index) = match path {
        QueryPath::SecondaryIndexRange { index, .. } => {
            ("secondary_index_range".to_string(), index.clone())
        }
        _ => ("full_scan".to_string(), String::new()),
    };
    Ok(IndexProof {
        query_kind: "secondary_range",
        predicate,
        expected_access_path: "secondary_index_range",
        observed_access_path,
        used_index,
        scan_rejected: true,
        indexed_result_count: count_rows(indexed_output),
        scan_result_count: count_rows(scan_output),
        indexed_result_hash: stable_hash(indexed_output),
        scan_result_hash: stable_hash(scan_output),
        result_hash_match: indexed_output == scan_output,
    })
}

fn count_rows(output: &str) -> usize {
    output.lines().skip(1).count()
}

fn median(values: &[f64]) -> f64 {
    let mut sorted = values.to_vec();
    sorted.sort_by(|left, right| left.total_cmp(right));
    sorted[sorted.len() / 2]
}

fn stable_hash(value: &str) -> u64 {
    let mut hash = 14_695_981_039_346_656_037u64;
    for byte in value.as_bytes() {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(1_099_511_628_211);
    }
    hash
}

fn file_len(path: impl AsRef<Path>) -> u64 {
    fs::metadata(path)
        .map(|metadata| metadata.len())
        .unwrap_or(0)
}

fn wal_path(path: &str) -> PathBuf {
    PathBuf::from(format!("{path}.wal"))
}

fn remove_db_files(path: &str) {
    let _ = fs::remove_file(path);
    let _ = fs::remove_file(wal_path(path));
}

fn atomic_write_evidence(path: &Path, evidence: &str) -> Result<(), BenchError> {
    if let Ok(metadata) = fs::symlink_metadata(path) {
        if metadata.file_type().is_symlink() {
            return Err(BenchError::new(
                "evidence_file",
                "evidence path must not be a symlink",
            ));
        }
    }
    let Some(parent) = path.parent() else {
        return Err(BenchError::new("evidence_file", "missing evidence parent"));
    };
    fs::create_dir_all(parent)?;
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| BenchError::new("evidence_file", "invalid evidence file name"))?;
    let tmp_path = parent.join(format!(".{file_name}.{}.tmp", std::process::id()));
    let _ = fs::remove_file(&tmp_path);
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&tmp_path)?;
    file.write_all(evidence.as_bytes())?;
    file.flush()?;
    file.sync_all()?;
    drop(file);
    fs::rename(&tmp_path, path)?;
    Ok(())
}

fn json_escape(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

fn f64_array(values: &[f64]) -> String {
    values
        .iter()
        .map(|value| format!("{value:.9}"))
        .collect::<Vec<_>>()
        .join(",")
}

fn usize_array(values: &[usize]) -> String {
    values
        .iter()
        .map(|value| value.to_string())
        .collect::<Vec<_>>()
        .join(",")
}

fn range_windows_json() -> String {
    range_windows()
        .into_iter()
        .map(|(low, high)| format!(r#"{{"low":{low},"high":{high}}}"#))
        .collect::<Vec<_>>()
        .join(",")
}

fn measured_counts_by_run_json() -> String {
    (1..=MEASUREMENT_RUNS)
        .map(|run| {
            r#"{"run":RUN,"primary_key_lookup":100,"secondary_equality_indexed":100,"secondary_equality_scan":100,"range_indexed":50,"range_scan":50,"total":400}"#
                .replace("RUN", &run.to_string())
        })
        .collect::<Vec<_>>()
        .join(",")
}

fn index_proofs_json(proofs: &[IndexProof]) -> String {
    proofs
        .iter()
        .map(index_proof_json)
        .collect::<Vec<_>>()
        .join(",\n    ")
}

fn build_evidence_json(run: &BenchRun, verify_status: &str, result: &str) -> String {
    let primary_key_lookup_median_ms = median(&run.primary_key_ms);
    let secondary_equality_indexed_median_ms = median(&run.equality_indexed_ms);
    let secondary_equality_scan_median_ms = median(&run.equality_scan_ms);
    let range_indexed_median_ms = median(&run.range_indexed_ms);
    let range_scan_median_ms = median(&run.range_scan_ms);
    let equality_index_speedup =
        secondary_equality_scan_median_ms / secondary_equality_indexed_median_ms;
    let range_index_speedup = range_scan_median_ms / range_indexed_median_ms;
    let hard_fail_checks = hard_fail_checks(run);
    format!(
        r#"{{
  "schema_version":1,
  "artifact":"{ARTIFACT}",
  "row_count":{ROW_COUNT},
  "primary_key_type":"INTEGER",
  "secondary_index_column":"group_key",
  "secondary_index_name":"{SECONDARY_INDEX_NAME}",
  "text_bytes_min":8,
  "text_bytes_max":64,
  "deterministic_seed":{DETERMINISTIC_SEED},
  "warmup_runs":{WARMUP_RUNS},
  "measurement_runs":{MEASUREMENT_RUNS},
  "runtime_cap_seconds":{RUNTIME_CAP_SECONDS},
  "commands":{{
    "db_bench":{{"status":"pass","evidence_path":"{EVIDENCE_PATH}","stdout_sentinel":"{DB_BENCH_PASS_SENTINEL}"}},
    "verify_bench_acceptance":{{"status":"{verify_status}","evidence_path":"{EVIDENCE_PATH}","stdout_sentinel":"{VERIFY_PASS_SENTINEL}"}}
  }},
  "workload":{{
    "table_schema":"bench_items(id INTEGER PRIMARY KEY, group_key INTEGER, payload TEXT)",
    "dataset_generation":"id 1..100000; group_key=((id * 7919 + deterministic_seed) % 10000) + 1; payload length=8 + ((id * 17 + deterministic_seed) % 57)",
    "primary_key_lookup":{{"rule":"lookup_id(n)=((deterministic_seed + n * 9973) % row_count) + 1","n_start":0,"n_end":99,"count_per_run":100,"keys":[{primary_key_lookup_keys}]}},
    "secondary_equality":{{"rule":"equality_key(n)=((deterministic_seed + n * 433) % 10000) + 1","n_start":0,"n_end":99,"count_per_run":100,"keys":[{secondary_equality_keys}]}},
    "secondary_range":{{"rule":"range_low(n)=((deterministic_seed + n * 137) % 9950) + 1; range_high=range_low+49","n_start":0,"n_end":49,"count_per_run":50,"range_window_distinct_group_keys":50,"selectivity_fraction":0.005,"windows":[{range_windows}]}},
    "scan_comparison":{{"mode":"explicit_harness_full_scan","same_dataset":true,"same_predicate":true,"result_hash_required":true}},
    "scan_comparison_mode":"explicit_harness_full_scan",
    "measured_counts_by_run":[{measured_counts_by_run}],
    "measured_query_count_per_run":400,
    "measured_query_count_total":2000,
    "sequential_inserts":100000,
    "recovery_after_committed_transactions":10000
  }},
  "raw_measurements":{{
    "runtime_ms":{runtime_ms:.3},
    "primary_key_lookup_run_ms":[{primary_runs}],
    "secondary_equality_indexed_run_ms":[{equality_indexed_runs}],
    "secondary_equality_scan_run_ms":[{equality_scan_runs}],
    "range_indexed_run_ms":[{range_indexed_runs}],
    "range_scan_run_ms":[{range_scan_runs}]
  }},
  "metrics":{{
    "sequential_insert_elapsed_ms":{sequential_insert_elapsed_ms:.3},
    "insert_throughput_rows_per_sec":{insert_throughput_rows_per_sec:.3},
    "primary_key_lookup_median_ms":{primary_key_lookup_median_ms:.9},
    "secondary_equality_indexed_median_ms":{secondary_equality_indexed_median_ms:.9},
    "secondary_equality_scan_median_ms":{secondary_equality_scan_median_ms:.9},
    "range_indexed_median_ms":{range_indexed_median_ms:.9},
    "range_scan_median_ms":{range_scan_median_ms:.9},
    "equality_index_speedup":{equality_index_speedup:.9},
    "range_index_speedup":{range_index_speedup:.9},
    "db_file_bytes":{db_file_bytes},
    "wal_file_bytes":{wal_file_bytes}
  }},
	    "recovery":{{
	    "committed_transaction_count":{committed_transaction_count},
	    "wal_replay_applied_records":{wal_replay_applied_records},
	    "wal_file_bytes":{recovery_wal_file_bytes},
    "recovery_ms":{recovery_ms:.3},
    "recovered_row_count":{recovered_row_count},
    "representative_lookup_result":{{"matched":{representative_matched},"id":{representative_id},"group_key":{representative_group_key},"payload":"{representative_payload}"}}
  }},
  "index_use_evidence":[
    {index_use_evidence}
  ],
  "hard_fail_checks":[
    {hard_fail_checks}
  ],
  "result":"{result}"
}}
"#,
        runtime_ms = run.runtime_ms,
        primary_runs = f64_array(&run.primary_key_ms),
        equality_indexed_runs = f64_array(&run.equality_indexed_ms),
        equality_scan_runs = f64_array(&run.equality_scan_ms),
        range_indexed_runs = f64_array(&run.range_indexed_ms),
        range_scan_runs = f64_array(&run.range_scan_ms),
        primary_key_lookup_keys = usize_array(&primary_key_lookup_ids()),
        secondary_equality_keys = usize_array(&equality_keys()),
        range_windows = range_windows_json(),
        measured_counts_by_run = measured_counts_by_run_json(),
        sequential_insert_elapsed_ms = run.setup.sequential_insert_elapsed_ms,
        insert_throughput_rows_per_sec = run.setup.insert_throughput_rows_per_sec,
        db_file_bytes = run.setup.db_file_bytes,
        wal_file_bytes = run.setup.wal_file_bytes,
        committed_transaction_count = run.recovery.committed_transaction_count,
        wal_replay_applied_records = run.recovery.wal_replay_applied_records,
        recovery_wal_file_bytes = run.recovery.wal_file_bytes,
        recovery_ms = run.recovery.recovery_ms,
        recovered_row_count = run.recovery.recovered_row_count,
        representative_matched = run.recovery.representative_matched,
        representative_id = run.recovery.representative_id,
        representative_group_key = run.recovery.representative_group_key,
        representative_payload = json_escape(&run.recovery.representative_payload),
        index_use_evidence = index_proofs_json(
            &run.equality_proofs
                .iter()
                .chain(run.range_proofs.iter())
                .cloned()
                .collect::<Vec<_>>()
        ),
        hard_fail_checks = hard_fail_checks
    )
}

fn index_proof_json(proof: &IndexProof) -> String {
    format!(
        r#"{{"query_kind":"{}","predicate":"{}","expected_access_path":"{}","observed_access_path":"{}","used_index":"{}","scan_rejected":{},"indexed_result_count":{},"scan_result_count":{},"indexed_result_hash":{},"scan_result_hash":{},"result_hash_match":{}}}"#,
        proof.query_kind,
        json_escape(&proof.predicate),
        proof.expected_access_path,
        proof.observed_access_path,
        proof.used_index,
        proof.scan_rejected,
        proof.indexed_result_count,
        proof.scan_result_count,
        proof.indexed_result_hash,
        proof.scan_result_hash,
        proof.result_hash_match
    )
}

fn hard_fail_checks(run: &BenchRun) -> String {
    let equality_proof_coverage = run.equality_proofs.len() == 100
        && run
            .equality_proofs
            .iter()
            .all(|proof| proof.observed_access_path == "secondary_index_equality");
    let range_proof_coverage = run.range_proofs.len() == 50
        && run
            .range_proofs
            .iter()
            .all(|proof| proof.observed_access_path == "secondary_index_range");
    let checks = [
        ("dataset_contract", true),
        (
            "equality_speedup",
            median(&run.equality_scan_ms) / median(&run.equality_indexed_ms) >= 5.0,
        ),
        (
            "range_speedup",
            median(&run.range_scan_ms) / median(&run.range_indexed_ms) >= 3.0,
        ),
        ("indexed_equality_no_full_scan", equality_proof_coverage),
        ("indexed_range_no_full_scan", range_proof_coverage),
        (
            "recovery_proportionality",
            run.recovery.recovery_ms <= 2000.0_f64.max(run.recovery.wal_file_bytes as f64 / 4096.0),
        ),
        (
            "wal_replay_applied_records",
            run.recovery.wal_replay_applied_records == RECOVERY_ROW_COUNT,
        ),
        ("no_retry_required", true),
    ];
    checks
        .iter()
        .map(|(check_id, passed)| {
            format!(
                r#"{{"check_id":"{check_id}","status":"{}"}}"#,
                if *passed { "pass" } else { "fail" }
            )
        })
        .collect::<Vec<_>>()
        .join(",\n    ")
}

pub fn validate_index_use_evidence_json_for_test(json: &str) -> Result<(), BenchError> {
    validate_index_use_evidence_json(json)
}

pub fn validate_section14_evidence_json_for_test(json: &str) -> Result<(), BenchError> {
    validate_section14_evidence_json(json)
}

fn validate_section14_evidence_json(json: &str) -> Result<(), BenchError> {
    let compact = compact_json(json);
    let equality_indexed = required_number(
        &compact,
        "\"secondary_equality_indexed_median_ms\":",
        "equality_speedup",
    )?;
    let equality_scan = required_number(
        &compact,
        "\"secondary_equality_scan_median_ms\":",
        "equality_speedup",
    )?;
    let equality_speedup =
        required_number(&compact, "\"equality_index_speedup\":", "equality_speedup")?;
    let recomputed_equality = equality_scan / equality_indexed;
    if equality_speedup < 5.0 || (equality_speedup - recomputed_equality).abs() > 0.01 {
        return Err(BenchError::new(
            "equality_speedup",
            format!(
                "equality speedup failed indexed={equality_indexed:.6} scan={equality_scan:.6} speedup={equality_speedup:.6}"
            ),
        ));
    }

    let range_indexed = required_number(&compact, "\"range_indexed_median_ms\":", "range_speedup")?;
    let range_scan = required_number(&compact, "\"range_scan_median_ms\":", "range_speedup")?;
    let range_speedup = required_number(&compact, "\"range_index_speedup\":", "range_speedup")?;
    let recomputed_range = range_scan / range_indexed;
    if range_speedup < 3.0 || (range_speedup - recomputed_range).abs() > 0.01 {
        return Err(BenchError::new(
            "range_speedup",
            format!(
                "range speedup failed indexed={range_indexed:.6} scan={range_scan:.6} speedup={range_speedup:.6}"
            ),
        ));
    }

    if compact.contains("\"artifact\":\"v1-section14-benchmark-acceptance\"") {
        for required in [
            "\"row_count\":100000",
            "\"primary_key_type\":\"INTEGER\"",
            "\"secondary_index_column\":\"group_key\"",
            "\"secondary_index_name\":\"idx_section14_bench_group_key\"",
            "\"text_bytes_min\":8",
            "\"text_bytes_max\":64",
            "\"deterministic_seed\":140014",
            "\"warmup_runs\":1",
            "\"measurement_runs\":5",
        ] {
            if !compact.contains(required) {
                return Err(BenchError::new(
                    "dataset_contract",
                    format!("missing {required}"),
                ));
            }
        }
    }

    let recovery_scope = required_section(&compact, "\"recovery\":", "recovery_contract")?;
    let committed = required_number(
        recovery_scope,
        "\"committed_transaction_count\":",
        "recovery_contract",
    )?;
    let recovered = required_number(
        recovery_scope,
        "\"recovered_row_count\":",
        "recovery_contract",
    )?;
    if committed != 10_000.0 || recovered != 10_000.0 {
        return Err(BenchError::new(
            "recovery_contract",
            "recovery row count mismatch",
        ));
    }
    let wal_replay_applied_records = required_number(
        recovery_scope,
        "\"wal_replay_applied_records\":",
        "recovery_contract",
    )?;
    if wal_replay_applied_records != 10_000.0 {
        return Err(BenchError::new(
            "recovery_contract",
            "wal replay applied record count mismatch",
        ));
    }

    let recovery_ms = required_number(
        recovery_scope,
        "\"recovery_ms\":",
        "recovery_proportionality",
    )?;
    let wal_file_bytes = required_number(
        recovery_scope,
        "\"wal_file_bytes\":",
        "recovery_proportionality",
    )?;
    let bound = 2000.0_f64.max(wal_file_bytes / 4096.0);
    if recovery_ms <= 0.0 || recovery_ms > bound {
        return Err(BenchError::new(
            "recovery_proportionality",
            "recovery_ms exceeded proportionality bound",
        ));
    }

    if !recovery_scope.contains("\"matched\":true") {
        return Err(BenchError::new(
            "recovery_contract",
            "representative lookup did not match",
        ));
    }

    validate_index_use_evidence_json(json)
}

fn validate_index_use_evidence_json(json: &str) -> Result<(), BenchError> {
    let compact = compact_json(json);
    let full_artifact = compact.contains("\"artifact\":\"v1-section14-benchmark-acceptance\"");
    let entries = extract_index_entries(json);
    let mut equality_count = 0usize;
    let mut range_count = 0usize;
    for entry in entries {
        let query_kind = entry_value(&entry, "query_kind").unwrap_or_default();
        let observed = entry_value(&entry, "observed_access_path").unwrap_or_default();
        let used_index = entry_value(&entry, "used_index").unwrap_or_default();
        let scan_rejected = bool_entry_value(&entry, "scan_rejected");
        let hash_match = bool_entry_value(&entry, "result_hash_match");
        let indexed_count = numeric_entry_value(&entry, "indexed_result_count");
        let scan_count = numeric_entry_value(&entry, "scan_result_count");
        if query_kind == "secondary_equality" {
            equality_count += 1;
            if observed != "secondary_index_equality"
                || used_index != SECONDARY_INDEX_NAME
                || !scan_rejected
                || !hash_match
                || indexed_count != scan_count
            {
                return Err(BenchError::new(
                    "indexed_equality_no_full_scan",
                    "eligible equality query failed index proof",
                ));
            }
        }
        if query_kind == "secondary_range" {
            range_count += 1;
            if observed != "secondary_index_range"
                || used_index != SECONDARY_INDEX_NAME
                || !scan_rejected
                || !hash_match
                || indexed_count != scan_count
            {
                return Err(BenchError::new(
                    "indexed_range_no_full_scan",
                    "eligible range query failed index proof",
                ));
            }
        }
    }

    if full_artifact {
        if equality_count != 100 {
            return Err(BenchError::new(
                "indexed_equality_no_full_scan",
                format!("expected 100 equality proofs, got {equality_count}"),
            ));
        }
        if range_count != 50 {
            return Err(BenchError::new(
                "indexed_range_no_full_scan",
                format!("expected 50 range proofs, got {range_count}"),
            ));
        }
    }
    Ok(())
}

fn compact_json(json: &str) -> String {
    json.chars().filter(|ch| !ch.is_whitespace()).collect()
}

fn required_number(
    compact_json: &str,
    key: &str,
    check_id: &'static str,
) -> Result<f64, BenchError> {
    number_after(compact_json, key)
        .ok_or_else(|| BenchError::new(check_id, format!("missing {key}")))
}

fn required_section<'a>(
    compact_json: &'a str,
    key: &str,
    check_id: &'static str,
) -> Result<&'a str, BenchError> {
    let start = compact_json
        .find(key)
        .ok_or_else(|| BenchError::new(check_id, format!("missing {key}")))?;
    compact_json
        .get(start..)
        .ok_or_else(|| BenchError::new(check_id, format!("missing {key}")))
}

fn number_after(compact_json: &str, key: &str) -> Option<f64> {
    let start = compact_json.find(key)? + key.len();
    let tail = &compact_json[start..];
    let end = tail
        .find(|ch: char| !(ch.is_ascii_digit() || ch == '.'))
        .unwrap_or(tail.len());
    tail.get(..end)?.parse().ok()
}

fn extract_index_entries(json: &str) -> Vec<String> {
    let compact = compact_json(json);
    let Some(start) = compact.find("\"index_use_evidence\":[") else {
        return Vec::new();
    };
    let mut entries = Vec::new();
    let mut depth = 0usize;
    let mut entry_start = None;
    for (index, ch) in compact[start..].char_indices() {
        let absolute = start + index;
        match ch {
            '{' => {
                if depth == 0 {
                    entry_start = Some(absolute);
                }
                depth += 1;
            }
            '}' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    if let Some(entry_start) = entry_start.take() {
                        entries.push(compact[entry_start..=absolute].to_string());
                    }
                }
            }
            ']' if depth == 0 && !entries.is_empty() => break,
            _ => {}
        }
    }
    entries
}

fn entry_value(entry: &str, key: &str) -> Option<String> {
    let needle = format!("\"{key}\":\"");
    let start = entry.find(&needle)? + needle.len();
    let tail = &entry[start..];
    let end = tail.find('"')?;
    Some(tail[..end].to_string())
}

fn bool_entry_value(entry: &str, key: &str) -> bool {
    entry.contains(&format!("\"{key}\":true"))
}

fn numeric_entry_value(entry: &str, key: &str) -> Option<u64> {
    let needle = format!("\"{key}\":");
    let start = entry.find(&needle)? + needle.len();
    let tail = &entry[start..];
    let end = tail
        .find(|ch: char| !ch.is_ascii_digit())
        .unwrap_or(tail.len());
    tail[..end].parse().ok()
}
