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
        script.contains("cargo run --quiet --bin db -- exec"),
        "benchmark work must run through db exec via cargo run"
    );
    assert!(
        !script.contains(" db bench") && !script.contains("-- bench"),
        "script must not call the reserved user-facing db bench command"
    );
    for required in [
        "bench_insert_1k",
        "bench_reopen_select_1k",
        "bench_items(id INT, value TEXT)",
        "value-0001",
        "value-1000",
        "target/bench_acceptance/v1-bench-docs-acceptance.json",
        "evidence-v1-benchmark-lower-bounds",
        "schema_version",
        "repo_sha",
        "created_at",
        "environment",
        "overall_passed",
        "observed_min_rows_per_second",
    ] {
        assert!(script.contains(required), "script missing {required:?}");
    }
}

#[test]
fn benchmark_documentation_contract_is_pinned() {
    let docs = read_repo_file("docs/benchmarks.md");
    for required in [
        "scripts/verify_bench_acceptance",
        "target/bench_acceptance/v1-bench-docs-acceptance.json",
        "bench_items(id INT, value TEXT)",
        "1,000",
        "bench_insert_1k",
        "bench_reopen_select_1k",
        "insert_rows_per_second >= 25",
        "select_rows_per_second >= 50",
        "minimum",
        "schema_version",
        "overall_passed",
        "Current Evidence",
        "observed minimum",
        "Environment Assumptions",
        "OS",
        "CPU",
        "toolchain",
        "not",
        "db bench",
    ] {
        assert!(
            docs.contains(required),
            "docs/benchmarks.md missing {required:?}"
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
