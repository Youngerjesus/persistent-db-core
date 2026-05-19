use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::{Mutex, MutexGuard, OnceLock};
use std::thread;
use std::time::Duration;

const EVIDENCE_PATH: &str = "target/bench_acceptance/section14-benchmark-acceptance.json";
const TEST_LOCK_DIR: &str = "target/bench_acceptance/.section14.test.lock";
const DB_BENCH_SENTINEL: &str =
    "DB_BENCH: PASS evidence=target/bench_acceptance/section14-benchmark-acceptance.json\n";
const VERIFY_SENTINEL: &str =
    "BENCH_ACCEPTANCE: PASS evidence=target/bench_acceptance/section14-benchmark-acceptance.json";
static BENCH_TEST_LOCK: Mutex<()> = Mutex::new(());
static DB_BENCH_RESULT: OnceLock<CommandResult> = OnceLock::new();
static VERIFY_ROOT_RESULT: OnceLock<CommandResult> = OnceLock::new();
static VERIFY_OUTSIDE_RESULT: OnceLock<CommandResult> = OnceLock::new();

struct CommandResult {
    code: Option<i32>,
    stdout: String,
    stderr: String,
    evidence: String,
}

fn repo_path(relative: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(relative)
}

fn db(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_db"))
        .args(args)
        .output()
        .expect("db binary should run")
}

fn command_result(output: Output) -> CommandResult {
    let evidence = fs::read_to_string(repo_path(EVIDENCE_PATH)).unwrap_or_default();
    CommandResult {
        code: output.status.code(),
        stdout: stdout(&output),
        stderr: stderr(&output),
        evidence,
    }
}

fn stdout(output: &Output) -> String {
    String::from_utf8(output.stdout.clone()).expect("stdout should be UTF-8")
}

fn stderr(output: &Output) -> String {
    String::from_utf8(output.stderr.clone()).expect("stderr should be UTF-8")
}

fn assert_json_contains(json: &str, needle: &str) {
    let compact: String = json.chars().filter(|ch| !ch.is_whitespace()).collect();
    let compact_needle: String = needle.chars().filter(|ch| !ch.is_whitespace()).collect();
    assert!(
        compact.contains(&compact_needle),
        "evidence JSON missing {needle:?}\n{json}"
    );
}

struct BenchTestGuard {
    _thread_guard: MutexGuard<'static, ()>,
    _process_guard: ProcessTestLock,
}

struct ProcessTestLock {
    path: PathBuf,
}

impl Drop for ProcessTestLock {
    fn drop(&mut self) {
        let _ = fs::remove_dir(&self.path);
    }
}

fn bench_test_guard() -> BenchTestGuard {
    let thread_guard = bench_thread_guard();
    let process_guard = acquire_process_test_lock();
    clean_bench_test_artifacts();
    BenchTestGuard {
        _thread_guard: thread_guard,
        _process_guard: process_guard,
    }
}

fn bench_thread_guard() -> MutexGuard<'static, ()> {
    BENCH_TEST_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn acquire_process_test_lock() -> ProcessTestLock {
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

fn clean_bench_test_artifacts() {
    let _ = fs::remove_file(repo_path(EVIDENCE_PATH));
}

fn db_bench_result() -> &'static CommandResult {
    DB_BENCH_RESULT.get_or_init(|| {
        let _guard = bench_test_guard();
        command_result(db(&["bench"]))
    })
}

fn verify_root_result() -> &'static CommandResult {
    VERIFY_ROOT_RESULT.get_or_init(|| {
        let _guard = bench_thread_guard();
        let script = repo_path("scripts/verify_bench_acceptance");
        command_result(
            Command::new(script)
                .output()
                .expect("verify_bench_acceptance should run"),
        )
    })
}

fn verify_outside_result() -> &'static CommandResult {
    VERIFY_OUTSIDE_RESULT.get_or_init(|| {
        let _guard = bench_thread_guard();
        let script = repo_path("scripts/verify_bench_acceptance");
        let outside = std::env::temp_dir();
        command_result(
            Command::new(script)
                .current_dir(outside)
                .output()
                .expect("verify_bench_acceptance should run from outside the repo"),
        )
    })
}

#[test]
fn db_bench_generates_section14_evidence_schema() {
    let output = db_bench_result();

    assert_eq!(
        Some(0),
        output.code,
        "db bench should pass; stdout={:?}; stderr={:?}",
        output.stdout,
        output.stderr
    );
    assert_eq!("", output.stderr);
    assert_eq!(DB_BENCH_SENTINEL, output.stdout);

    let json = &output.evidence;
    for required in [
        "\"schema_version\"",
        "\"artifact\":\"v1-section14-benchmark-acceptance\"",
        "\"row_count\":100000",
        "\"primary_key_type\":\"INTEGER\"",
        "\"secondary_index_column\":\"group_key\"",
        "\"secondary_index_name\":\"idx_section14_bench_group_key\"",
        "\"text_bytes_min\":8",
        "\"text_bytes_max\":64",
        "\"deterministic_seed\":140014",
        "\"warmup_runs\":1",
        "\"measurement_runs\":5",
        "\"runtime_cap_seconds\"",
        "\"commands\"",
        "\"workload\"",
        "\"metrics\"",
        "\"recovery\"",
        "\"index_use_evidence\"",
        "\"hard_fail_checks\"",
        "\"result\"",
    ] {
        assert_json_contains(json, required);
    }
    assert_json_contains(json, "\"verify_bench_acceptance\":{\"status\":\"pending\"");
    assert_json_contains(json, "\"result\":\"pending\"");
}

#[test]
fn db_bench_records_required_metrics_recovery_and_index_proof() {
    let output = db_bench_result();
    assert_eq!(
        Some(0),
        output.code,
        "stdout={:?}; stderr={:?}",
        output.stdout,
        output.stderr
    );

    let json = &output.evidence;
    for required in [
        "\"sequential_insert_elapsed_ms\"",
        "\"insert_throughput_rows_per_sec\"",
        "\"primary_key_lookup_median_ms\"",
        "\"secondary_equality_indexed_median_ms\"",
        "\"secondary_equality_scan_median_ms\"",
        "\"range_indexed_median_ms\"",
        "\"range_scan_median_ms\"",
        "\"equality_index_speedup\"",
        "\"range_index_speedup\"",
        "\"db_file_bytes\"",
        "\"wal_file_bytes\"",
        "\"committed_transaction_count\":10000",
        "\"wal_replay_applied_records\":10000",
        "\"recovered_row_count\":10000",
        "\"recovery_ms\"",
        "\"representative_lookup_result\"",
        "\"query_kind\":\"secondary_equality\"",
        "\"query_kind\":\"secondary_range\"",
        "\"expected_access_path\"",
        "\"observed_access_path\"",
        "\"used_index\":\"idx_section14_bench_group_key\"",
        "\"scan_rejected\":true",
        "\"result_hash_match\":true",
        "\"primary_key_lookup\"",
        "\"keys\"",
        "\"secondary_equality\"",
        "\"secondary_range\"",
        "\"range_window_distinct_group_keys\":50",
        "\"selectivity_fraction\":0.005",
        "\"scan_comparison\"",
        "\"mode\":\"explicit_harness_full_scan\"",
        "\"measured_counts_by_run\"",
        "\"primary_key_lookup\":100",
        "\"secondary_equality_indexed\":100",
        "\"secondary_equality_scan\":100",
        "\"range_indexed\":50",
        "\"range_scan\":50",
        "\"check_id\":\"dataset_contract\",\"status\":\"pass\"",
        "\"check_id\":\"equality_speedup\",\"status\":\"pass\"",
        "\"check_id\":\"range_speedup\",\"status\":\"pass\"",
        "\"check_id\":\"indexed_equality_no_full_scan\",\"status\":\"pass\"",
        "\"check_id\":\"indexed_range_no_full_scan\",\"status\":\"pass\"",
        "\"check_id\":\"recovery_proportionality\",\"status\":\"pass\"",
        "\"check_id\":\"wal_replay_applied_records\",\"status\":\"pass\"",
        "\"check_id\":\"no_retry_required\",\"status\":\"pass\"",
    ] {
        assert_json_contains(json, required);
    }
}

#[test]
fn hard_fail_validator_rejects_partial_equality_proof_coverage() {
    let range_entries = (0..50)
        .map(|n| {
            let low = ((140014 + n * 137) % 9950) + 1;
            let high = low + 49;
            format!(
                r#"{{"query_kind":"secondary_range","predicate":"group_key BETWEEN {low} AND {high}","expected_access_path":"secondary_index_range","observed_access_path":"secondary_index_range","used_index":"idx_section14_bench_group_key","scan_rejected":true,"indexed_result_count":500,"scan_result_count":500,"indexed_result_hash":1,"scan_result_hash":1,"result_hash_match":true}}"#
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    let invalid = format!(
        r#"{{"artifact":"v1-section14-benchmark-acceptance","index_use_evidence":[{range_entries}]}}"#
    );

    let error = persistent_db_core::bench::validate_index_use_evidence_json_for_test(&invalid)
        .expect_err("missing equality proof coverage must be rejected");
    assert_eq!("indexed_equality_no_full_scan", error.check_id());
}

#[test]
fn hard_fail_validator_rejects_partial_range_proof_coverage() {
    let equality_entries = (0..100)
        .map(|n| {
            let key = ((140014 + n * 433) % 10000) + 1;
            format!(
                r#"{{"query_kind":"secondary_equality","predicate":"group_key = {key}","expected_access_path":"secondary_index_equality","observed_access_path":"secondary_index_equality","used_index":"idx_section14_bench_group_key","scan_rejected":true,"indexed_result_count":10,"scan_result_count":10,"indexed_result_hash":1,"scan_result_hash":1,"result_hash_match":true}}"#
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    let invalid = format!(
        r#"{{"artifact":"v1-section14-benchmark-acceptance","index_use_evidence":[{equality_entries}]}}"#
    );

    let error = persistent_db_core::bench::validate_index_use_evidence_json_for_test(&invalid)
        .expect_err("missing range proof coverage must be rejected");
    assert_eq!("indexed_range_no_full_scan", error.check_id());
}

#[test]
fn verify_bench_acceptance_runs_public_db_bench_and_finalizes_same_evidence() {
    let output = verify_root_result();

    assert_eq!(
        Some(0),
        output.code,
        "stdout={:?}; stderr={:?}",
        output.stdout,
        output.stderr
    );
    assert_eq!("", output.stderr);
    assert_eq!(
        format!("{VERIFY_SENTINEL}\n"),
        output.stdout,
        "script stdout must match the verifier sentinel exactly"
    );

    let json = &output.evidence;
    assert_json_contains(json, "\"db_bench\"");
    assert_json_contains(json, "\"verify_bench_acceptance\"");
    assert_json_contains(json, "\"status\":\"pass\"");
    assert_json_contains(
        json,
        "\"stdout_sentinel\":\"DB_BENCH: PASS evidence=target/bench_acceptance/section14-benchmark-acceptance.json\"",
    );
    assert_json_contains(
        json,
        "\"stdout_sentinel\":\"BENCH_ACCEPTANCE: PASS evidence=target/bench_acceptance/section14-benchmark-acceptance.json\"",
    );
    assert_json_contains(json, "\"result\":\"pass\"");
}

#[test]
fn verify_bench_acceptance_works_from_outside_repo_with_absolute_path() {
    let output = verify_outside_result();

    assert_eq!(
        Some(0),
        output.code,
        "stdout={:?}; stderr={:?}",
        output.stdout,
        output.stderr
    );
    assert_eq!("", output.stderr);
    assert_eq!(
        format!("{VERIFY_SENTINEL}\n"),
        output.stdout,
        "script stdout must match the verifier sentinel exactly"
    );
    assert!(
        repo_path(EVIDENCE_PATH).exists(),
        "script must write the repo-relative evidence path"
    );
}

#[test]
fn verify_bench_acceptance_failure_uses_contract_sentinel_and_nonzero_exit() {
    let _guard = bench_thread_guard();
    let script = repo_path("scripts/verify_bench_acceptance");
    let output = Command::new(script)
        .env("PATH", "/bin:/usr/bin")
        .output()
        .expect("verify_bench_acceptance should run far enough to report missing cargo");

    assert_ne!(Some(0), output.status.code());
    assert_eq!("", stderr(&output));
    assert_eq!(
        "BENCH_ACCEPTANCE: FAIL check=required_tool reason=cargo-not-found\n",
        stdout(&output)
    );
}

#[test]
fn hard_fail_validator_rejects_indexed_equality_full_scan_evidence() {
    let invalid = r#"{
        "index_use_evidence": [
            {
                "query_kind": "secondary_equality",
                "predicate": "group_key = 42",
                "expected_access_path": "secondary_index_equality",
                "observed_access_path": "full_scan",
                "used_index": "",
                "scan_rejected": false,
                "indexed_result_count": 10,
                "scan_result_count": 10,
                "result_hash_match": true
            }
        ]
    }"#;

    let error = persistent_db_core::bench::validate_index_use_evidence_json_for_test(invalid)
        .expect_err("eligible equality full scan must be rejected");
    assert_eq!("indexed_equality_no_full_scan", error.check_id());
}

#[test]
fn hard_fail_validator_rejects_indexed_range_full_scan_evidence() {
    let invalid = r#"{
        "index_use_evidence": [
            {
                "query_kind": "secondary_range",
                "predicate": "group_key BETWEEN 42 AND 91",
                "expected_access_path": "secondary_index_range",
                "observed_access_path": "full_scan",
                "used_index": "",
                "scan_rejected": false,
                "indexed_result_count": 500,
                "scan_result_count": 500,
                "result_hash_match": true
            }
        ]
    }"#;

    let error = persistent_db_core::bench::validate_index_use_evidence_json_for_test(invalid)
        .expect_err("eligible range full scan must be rejected");
    assert_eq!("indexed_range_no_full_scan", error.check_id());
}

#[test]
fn hard_fail_validator_rejects_below_threshold_equality_speedup() {
    let invalid = r#"{
        "row_count": 100000,
        "metrics": {
            "secondary_equality_indexed_median_ms": 10.0,
            "secondary_equality_scan_median_ms": 49.0,
            "equality_index_speedup": 4.9,
            "range_index_speedup": 3.1
        },
        "recovery": {
            "wal_file_bytes": 4096,
            "recovery_ms": 1,
            "committed_transaction_count": 10000,
            "recovered_row_count": 10000,
            "representative_lookup_result": {"matched": true}
        }
    }"#;

    let error = persistent_db_core::bench::validate_section14_evidence_json_for_test(invalid)
        .expect_err("equality speedup below 5.0 must be rejected");
    assert_eq!("equality_speedup", error.check_id());
}

#[test]
fn hard_fail_validator_rejects_below_threshold_range_speedup() {
    let invalid = r#"{
        "row_count": 100000,
        "metrics": {
            "secondary_equality_indexed_median_ms": 10.0,
            "secondary_equality_scan_median_ms": 50.0,
            "equality_index_speedup": 5.0,
            "range_indexed_median_ms": 10.0,
            "range_scan_median_ms": 29.0,
            "range_index_speedup": 2.9
        },
        "recovery": {
            "wal_file_bytes": 4096,
            "recovery_ms": 1,
            "committed_transaction_count": 10000,
            "recovered_row_count": 10000,
            "representative_lookup_result": {"matched": true}
        }
    }"#;

    let error = persistent_db_core::bench::validate_section14_evidence_json_for_test(invalid)
        .expect_err("range speedup below 3.0 must be rejected");
    assert_eq!("range_speedup", error.check_id());
}

#[test]
fn hard_fail_validator_rejects_recovery_proportionality_violation() {
    let invalid = r#"{
        "row_count": 100000,
        "metrics": {
            "secondary_equality_indexed_median_ms": 10.0,
            "secondary_equality_scan_median_ms": 50.0,
            "equality_index_speedup": 5.0,
            "range_indexed_median_ms": 10.0,
            "range_scan_median_ms": 30.0,
            "range_index_speedup": 3.0
        },
        "recovery": {
            "wal_file_bytes": 4096,
            "wal_replay_applied_records": 10000,
            "recovery_ms": 2001,
            "committed_transaction_count": 10000,
            "recovered_row_count": 10000,
            "representative_lookup_result": {"matched": true}
        }
    }"#;

    let error = persistent_db_core::bench::validate_section14_evidence_json_for_test(invalid)
        .expect_err("recovery_ms above max(2000, wal_file_bytes / 4096) must be rejected");
    assert_eq!("recovery_proportionality", error.check_id());
}

#[test]
fn hard_fail_validator_rejects_missing_wal_replay_apply_count() {
    let invalid = r#"{
        "row_count": 100000,
        "metrics": {
            "secondary_equality_indexed_median_ms": 10.0,
            "secondary_equality_scan_median_ms": 50.0,
            "equality_index_speedup": 5.0,
            "range_indexed_median_ms": 10.0,
            "range_scan_median_ms": 30.0,
            "range_index_speedup": 3.0
        },
        "recovery": {
            "wal_file_bytes": 4096,
            "recovery_ms": 1,
            "committed_transaction_count": 10000,
            "recovered_row_count": 10000,
            "representative_lookup_result": {"matched": true}
        }
    }"#;

    let error = persistent_db_core::bench::validate_section14_evidence_json_for_test(invalid)
        .expect_err("recovery evidence must prove 10000 WAL records were applied");
    assert_eq!("recovery_contract", error.check_id());
}

#[test]
fn hard_fail_validator_uses_recovery_wal_file_bytes_for_recovery_bound() {
    let invalid = r#"{
        "row_count": 100000,
        "metrics": {
            "secondary_equality_indexed_median_ms": 10.0,
            "secondary_equality_scan_median_ms": 50.0,
            "equality_index_speedup": 5.0,
            "range_indexed_median_ms": 10.0,
            "range_scan_median_ms": 30.0,
            "range_index_speedup": 3.0,
            "wal_file_bytes": 18478090
        },
        "recovery": {
            "wal_file_bytes": 4096,
            "wal_replay_applied_records": 10000,
            "recovery_ms": 2001,
            "committed_transaction_count": 10000,
            "recovered_row_count": 10000,
            "representative_lookup_result": {"matched": true}
        }
    }"#;

    let error = persistent_db_core::bench::validate_section14_evidence_json_for_test(invalid)
        .expect_err("recovery bound must use recovery.wal_file_bytes, not metrics.wal_file_bytes");
    assert_eq!("recovery_proportionality", error.check_id());
}
