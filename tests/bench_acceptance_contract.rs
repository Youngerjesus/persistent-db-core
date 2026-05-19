use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

fn repo_path(relative: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(relative)
}

fn read_repo_file(relative: &str) -> String {
    fs::read_to_string(repo_path(relative))
        .unwrap_or_else(|err| panic!("expected {relative} to be readable: {err}"))
}

#[test]
fn benchmark_acceptance_script_contract_is_pinned() {
    let script_path = repo_path("scripts/verify_bench_acceptance");
    let metadata = fs::metadata(&script_path)
        .unwrap_or_else(|err| panic!("missing benchmark acceptance script: {err}"));
    assert!(
        metadata.permissions().mode() & 0o111 != 0,
        "scripts/verify_bench_acceptance must be executable"
    );

    let script = read_repo_file("scripts/verify_bench_acceptance");
    assert!(
        script.starts_with("#!/usr/bin/env bash\nset -euo pipefail\n"),
        "script must be a strict bash verification entrypoint"
    );
    assert!(
        script.contains("cargo run --bin db -- bench")
            || script.contains("cargo run --quiet --bin db -- bench"),
        "benchmark acceptance must invoke the public db bench command"
    );
    assert!(
        !script.contains("DB_BENCH_LOCK_HELD=1"),
        "script must not bypass the public db bench lock owner"
    );
    assert!(
        script.contains("tempfile.mkstemp") && script.contains("os.fsync") && script.contains("os.replace"),
        "script must finalize evidence through a flushed same-directory temp file and atomic rename"
    );
    assert!(
        script.contains("lock_acquired=0")
            && script.contains("lock_acquired=1")
            && script.contains("[[ \"$lock_acquired\" != \"1\" ]]"),
        "script must fail unless it actually owns the verifier lock"
    );
    assert!(
        !script.contains("--bin db -- exec") && !script.contains(" db exec "),
        "script must not use the obsolete script-only db exec 1k benchmark path"
    );
    for required in [
        "BENCH_ACCEPTANCE: PASS evidence=target/bench_acceptance/section14-benchmark-acceptance.json",
        "BENCH_ACCEPTANCE: FAIL check=",
        "target/bench_acceptance/section14-benchmark-acceptance.json",
        "DB_BENCH: PASS evidence=target/bench_acceptance/section14-benchmark-acceptance.json",
        "schema_version",
        "v1-section14-benchmark-acceptance",
        "row_count",
        "100000",
        "primary_key_type",
        "INTEGER",
        "secondary_index_column",
        "group_key",
        "secondary_index_name",
        "idx_section14_bench_group_key",
        "deterministic_seed",
        "140014",
        "warmup_runs",
        "measurement_runs",
        "runtime_cap_seconds",
        "equality_index_speedup",
        "range_index_speedup",
        "recovery_ms",
        "wal_replay_applied_records",
        "wal_file_bytes",
        "index_use_evidence",
        "hard_fail_checks",
        "indexed_equality_no_full_scan",
        "indexed_range_no_full_scan",
        "recovery_proportionality",
        "wal_replay_applied_records",
    ] {
        assert!(script.contains(required), "script missing {required:?}");
    }
}

#[test]
fn db_bench_file_boundary_contract_is_pinned() {
    let bench = read_repo_file("src/bench.rs");
    assert!(
        bench.contains("symlink_metadata") && bench.contains("atomic_write_evidence"),
        "db bench must reject symlink evidence paths and finalize evidence atomically"
    );
    assert!(
        !bench.contains("fs::write(path, evidence)"),
        "db bench must not write the public evidence artifact through symlink-following fs::write"
    );
    assert!(
        !bench.contains("Command::new(\"kill\")"),
        "db bench stale-lock detection must not resolve kill through PATH"
    );
}

#[test]
fn benchmark_documentation_contract_is_pinned() {
    let docs = read_repo_file("docs/benchmarks.md");
    for required in [
        "Section 14",
        "db bench",
        "scripts/verify_bench_acceptance",
        "target/bench_acceptance/section14-benchmark-acceptance.json",
        "bench_items(id INTEGER PRIMARY KEY, group_key INTEGER, payload TEXT)",
        "100,000",
        "idx_section14_bench_group_key",
        "deterministic_seed=140014",
        "warmup_runs=1",
        "measurement_runs=5",
        "equality_index_speedup = secondary_equality_scan_median_ms / secondary_equality_indexed_median_ms",
        "range_index_speedup = range_scan_median_ms / range_indexed_median_ms",
        "equality_index_speedup >= 5.0",
        "range_index_speedup >= 3.0",
        "recovery_ms <= max(2000, wal_file_bytes / 4096)",
        "BENCH_ACCEPTANCE: PASS evidence=target/bench_acceptance/section14-benchmark-acceptance.json",
        "DB_BENCH: PASS evidence=target/bench_acceptance/section14-benchmark-acceptance.json",
        "schema_version",
        "metrics",
        "recovery",
        "index_use_evidence",
        "hard_fail_checks",
        "METRIC-14-1",
        "METRIC-14-2",
        "METRIC-14-3",
        "METRIC-14-4",
        "FAIL-14-5",
    ] {
        assert!(
            docs.contains(required),
            "docs/benchmarks.md missing {required:?}"
        );
    }
    for obsolete in [
        "target/bench_acceptance/v1-bench-docs-acceptance.json",
        "bench_insert_1k",
        "bench_reopen_select_1k",
        "db bench` remains a reserved future CLI command",
    ] {
        assert!(
            !docs.contains(obsolete),
            "docs/benchmarks.md must not retain obsolete 1k benchmark contract {obsolete:?}"
        );
    }
}

#[test]
fn v1_acceptance_guide_maps_required_gates_to_evidence() {
    let docs = read_repo_file("docs/v1_acceptance.md");
    assert!(
        docs.contains("evidence-v1-acceptance-docs"),
        "acceptance guide must expose the final report evidence id"
    );
    assert!(
        docs.contains("autopilot/ssot/current-artifact.md"),
        "acceptance guide must name the handoff gate source"
    );
    assert!(
        docs.contains("src/index.rs"),
        "primary-index evidence must reference the existing src/index.rs path"
    );
    assert!(
        !docs.contains("src/primary_index.rs"),
        "acceptance guide must not reference nonexistent src/primary_index.rs"
    );
    assert!(
        !docs.contains("verification_ready") && !docs.contains("pending_current_task_verification"),
        "acceptance guide statuses must describe current evidence state, not ambiguous readiness"
    );
    assert!(
        docs.contains("verified_current_run"),
        "acceptance guide must mark locally verified rows explicitly"
    );
    assert!(
        docs.contains("seed_capture_missing"),
        "differential-property seed-capture gap must be explicit"
    );

    for (gate, req) in [
        ("gate-v1-cli-smoke", "req-v1-cli-help-smoke"),
        ("gate-v1-cli-smoke", "req-v1-cli-dispatch-tests"),
        ("gate-v1-disk-page-storage", "req-v1-page-storage-restart"),
        ("gate-v1-disk-page-storage", "req-v1-record-format-doc"),
        ("gate-v1-sql-schema-exec", "req-v1-sql-exec-examples"),
        ("gate-v1-indexes", "req-v1-primary-index-proof"),
        ("gate-v1-indexes", "req-v1-secondary-index-proof"),
        (
            "gate-v1-transactions-wal-recovery",
            "req-v1-wal-recovery-proof",
        ),
        ("gate-v1-crash-testing", "req-v1-crash-matrix-output"),
        (
            "gate-v1-differential-property-tests",
            "req-v1-differential-property-proof",
        ),
        ("gate-v1-db-check-invariants", "req-v1-db-check-proof"),
        (
            "gate-v1-bench-docs-acceptance",
            "req-v1-benchmark-lower-bounds",
        ),
        ("gate-v1-bench-docs-acceptance", "req-v1-acceptance-docs"),
    ] {
        assert!(docs.contains(gate), "acceptance guide missing gate {gate}");
        assert!(
            docs.contains(req),
            "acceptance guide missing requirement {req}"
        );
    }

    assert!(
        docs.contains("blocked_missing_evidence")
            || docs.contains("out_of_scope_for_this_task")
            || docs.contains("seed_capture_missing"),
        "acceptance guide must explicitly mark incomplete evidence instead of relying on progress projection"
    );
}
