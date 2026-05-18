# V1 Acceptance Guide

Evidence id: `evidence-v1-acceptance-docs`

Gate source at task handoff: `autopilot/ssot/current-artifact.md`, specifically the Launch Gate Evidence Contract and Evidence Requirements sections. This guide maps that source to current repo evidence without treating progress projection as proof.

## Gate Evidence Map

| Gate id | Requirement id | Evidence path | Verification command or manual review evidence | Current status |
| --- | --- | --- | --- | --- |
| `gate-v1-cli-smoke` | `req-v1-cli-help-smoke` | `docs/cli_contract.md`; `src/main.rs`; `tests/cli_contract.rs` | `scripts/verify`; `cargo run --bin db -- --help`; `cargo test --test cli_contract` | `verified_current_run` |
| `gate-v1-cli-smoke` | `req-v1-cli-dispatch-tests` | `tests/cli_contract.rs` | `cargo test --test cli_contract` | `verified_current_run` |
| `gate-v1-disk-page-storage` | `req-v1-page-storage-restart` | `src/storage.rs`; `tests/page_storage.rs` | `cargo test --test page_storage`; included in `scripts/verify` | `verified_current_run` |
| `gate-v1-disk-page-storage` | `req-v1-record-format-doc` | `docs/file_format.md` | Manual review of documented page, SQL logical record, and WAL sidecar compatibility notes | `verified_current_run` |
| `gate-v1-sql-schema-exec` | `req-v1-sql-exec-examples` | `docs/sql_subset.md`; `tests/sql_exec.rs` | `cargo test --test sql_exec`; included in `scripts/verify` | `verified_current_run` |
| `gate-v1-indexes` | `req-v1-primary-index-proof` | `tests/primary_index.rs`; `src/index.rs`; `docs/sql_subset.md` | `cargo test --test primary_index`; included in `scripts/verify` | `verified_current_run` |
| `gate-v1-indexes` | `req-v1-secondary-index-proof` | `tests/secondary_index.rs`; `src/sql.rs`; `src/index.rs`; `docs/cli_contract.md`; `docs/file_format.md` | `cargo test --test secondary_index -- --nocapture`; included in `scripts/verify`; manual review of persisted `E`/`X`/`I` record docs and `db check` invariant coverage | `verified_current_run` |
| `gate-v1-transactions-wal-recovery` | `req-v1-wal-recovery-proof` | `tests/wal_recovery.rs`; `docs/file_format.md` | `cargo test --test wal_recovery`; included in `scripts/verify` | `verified_current_run` |
| `gate-v1-crash-testing` | `req-v1-crash-matrix-output` | `tests/crash_matrix.rs`; `tests/fixtures/crash_matrix/README.md`; `target/crash_matrix/` when generated | `scripts/verify_crash_matrix` when crash-matrix evidence is required; crash tests are also covered by `scripts/verify` if present in the normal test suite | `verified_current_run` |
| `gate-v1-differential-property-tests` | `req-v1-differential-property-proof` | `tests/differential_property.rs`; `scripts/verify_differential_property`; `target/differential_property/` only when a mismatch artifact is generated | `scripts/verify_differential_property`; blocker: no current passing-run deterministic seed-capture artifact is produced by the existing test command | `seed_capture_missing` |
| `gate-v1-db-check-invariants` | `req-v1-db-check-proof` | `docs/cli_contract.md`; `tests/db_check.rs` | `cargo test --test db_check`; included in `scripts/verify` | `verified_current_run` |
| `gate-v1-bench-docs-acceptance` | `req-v1-benchmark-lower-bounds` | `docs/benchmarks.md`; `scripts/verify_bench_acceptance`; `target/bench_acceptance/v1-bench-docs-acceptance.json` | `scripts/verify_bench_acceptance`; final report evidence id `evidence-v1-benchmark-lower-bounds` | `verified_current_run` |
| `gate-v1-bench-docs-acceptance` | `req-v1-acceptance-docs` | `docs/v1_acceptance.md` | Manual review of this guide against `autopilot/ssot/current-artifact.md`; final report evidence id `evidence-v1-acceptance-docs` | `verified_current_run` |

## Acceptance Boundary

V1 remains a single-process Rust CLI database. This guide does not claim network service behavior, multi-process concurrency, distributed storage, public benchmark CLI support, mutation-maintained secondary-index behavior beyond append-only `INSERT`, or performance beyond the lower-bound workload documented in `docs/benchmarks.md`.
