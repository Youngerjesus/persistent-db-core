# Code Context Evidence

- available: true
- repo_root: /Users/jeongmin/Downloads/projects/autopilot-test/01-clear-answer-complex-software-db-core/persistent-db-core_worktree/main
- head_sha: d943f7404a992203822d00ef9a8194e766f15f87
- base_branch: main
- dirty_files: none
- collected_at: 2026-05-19T06:45:07.377998+00:00
- selected_files: src/main.rs, src/lib.rs, scripts/verify_bench_acceptance, docs/benchmarks.md, docs/v1_acceptance.md, work_queue/progress.md, tests/bench_acceptance_contract.rs, tests/cli_contract.rs, docs/v1_spec.md

## Omitted Reasons
- WAL/recovery: not_git_tracked
- active/reserved: not_git_tracked
- docs/bug_diary.md: not_git_tracked
- docs/cli_contract.md: context_char_limit
- docs/performance_report.md: not_git_tracked
- flow:section14-performance-acceptance: not_git_tracked
- insert/reopen: not_git_tracked
- project_manager/specs/v1-bench-docs-acceptance/contracts.md: not_git_tracked
- project_manager/specs/v1-bench-docs-acceptance/spec.md: not_git_tracked
- project_manager/tasks/tasks.json: not_git_tracked
- route:db bench: not_git_tracked
- specs/v1-bench-docs-acceptance/contracts.md: context_char_limit
- specs/v1-bench-docs-acceptance/impl_review.md: context_char_limit
- src/bench.rs: not_git_tracked
- ssot/current-artifact.md: not_git_tracked
- ssot/current-plan.md: not_git_tracked
- target/bench_acceptance/v1-bench-docs-acceptance.json: not_git_tracked
- tests/bench_acceptance.rs: not_git_tracked

## File Excerpts

### src/main.rs
- excerpt_chars: 3474
- clipped: false

```text
use std::env;
use std::process;

use persistent_db_core::check::{self, CheckError};
use persistent_db_core::sql::{self, SqlError};

const HELP: &str = "\
db - deterministic single-process V1 database CLI
Usage:
  db --help
  db help
  db exec <path> <sql>
  db check <path>
Supported commands:
  help        Print this help text.
  exec <path> <sql>
  check <path>
Reserved future commands:
  open <path>
  bench <path>
V1 scope:
  This build supports the CLI contract, page storage, and the documented minimal SQL subset.
Non-goals:
  No network server, multi-process concurrency, or distributed storage.
";

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    match args.as_slice() {
        [arg] if arg == "--help" || arg == "help" => {
            print!("{HELP}");
        }
        [command, path, sql_text] if command == "exec" => match sql::execute(path, sql_text) {
            Ok(stdout) => {
                print!("{stdout}");
            }
            Err(error) => exit_with_sql_error(error),
        },
        [command, path] if command == "check" => match check::check_database(path) {
            Ok(()) => {
                print!("{}", check::SUCCESS_OUTPUT);
            }
            Err(error) => exit_with_check_error(error),
        },
        [token, ..] => {
            eprintln!("error: unsupported argument or command: {token}");
            eprintln!("hint: run 'db --help' for the supported V1 CLI contract.");
            process::exit(2);
        }
        [] => {
            eprintln!("error: unsupported argument or command: <none>");
            eprintln!("hint: run 'db --help' for the supported V1 CLI contract.");
            process::exit(2);
        }
    }
}

fn exit_with_check_error(error: CheckError) -> ! {
    match error {
        CheckError::OpenRead { path } => {
            eprintln!(
                "error: could not open or read database path: {}",
                path.display()
            );
            process::exit(1);
        }
        CheckError::Invariant { label } => {
            eprintln!("error: db check failed: {label}");
            process::exit(1);
        }
    }
}

fn exit_with_sql_error(error: SqlError) -> ! {
    match error {
        SqlError::Unsupported(statement) => {
            eprintln!("error: unsupported SQL statement: {statement}");
            eprintln!("hint: supported SQL subset is documented in docs/sql_subset.md.");
            process::exit(2);
        }
        SqlError::Malformed(statement) => {
            eprintln!("error: malformed SQL statement: {statement}");
            eprintln!("hint: terminate each statement with ';' and use the documented SQL subset.");
            process::exit(2);
        }
        SqlError::Semantic { message, hint } => {
            eprintln!("error: SQL semantic error: {message}");
            eprintln!("hint: {hint}");
            process::exit(2);
        }
        SqlError::InvalidStorageRecord => {
            eprintln!("error: invalid SQL storage record: unknown record tag");
            eprintln!(
                "hint: run against a database file created by this SQL contract or restore from a valid backup."
            );
            process::exit(1);
        }
        SqlError::Storage(error) => {
            eprintln!("error: storage error: {error:?}");
            eprintln!("hint: database file must use the documented V1 page format.");
            process::exit(1);
        }
    }
}
```

### src/lib.rs
- excerpt_chars: 60
- clipped: false

```text
pub mod check;
pub mod index;
pub mod sql;
pub mod storage;
```

### scripts/verify_bench_acceptance
- excerpt_chars: 4000
- clipped: true

```text
#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

artifact_dir="target/bench_acceptance"
artifact_path="$artifact_dir/v1-bench-docs-acceptance.json"
artifact_contract_path="target/bench_acceptance/v1-bench-docs-acceptance.json"
mkdir -p "$artifact_dir"

row_count=1000
warmup_iterations=1
measured_iterations=3
insert_threshold=25
select_threshold=50
command_text="cargo run --quiet --bin db -- exec"
evidence_id="evidence-v1-benchmark-lower-bounds"

require_tool() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "error: required tool not found: $1" >&2
    exit 1
  fi
}

require_tool cargo
require_tool rustc
require_tool awk
require_tool git

now_ns() {
  local value
  value="$(date +%s%N 2>/dev/null || true)"
  if [[ "$value" =~ ^[0-9]+$ ]] && [[ "$value" != *N ]]; then
    printf '%s\n' "$value"
    return
  fi
  require_tool python3
  python3 - <<'PY'
import time
print(time.time_ns())
PY
}

json_escape() {
  printf '%s' "$1" | sed 's/\\/\\\\/g; s/"/\\"/g'
}

run_db_exec() {
  local db_path="$1"
  local sql="$2"
  local stderr_path
  stderr_path="$(mktemp)"
  set +e
  RUN_STDOUT="$(cargo run --quiet --bin db -- exec "$db_path" "$sql" 2>"$stderr_path")"
  RUN_STATUS=$?
  set -e
  RUN_STDERR="$(cat "$stderr_path")"
  rm -f "$stderr_path"
}

build_insert_sql() {
  local sql="CREATE TABLE bench_items(id INT, value TEXT);"
  local id
  local value
  for id in $(seq 1 "$row_count"); do
    printf -v value "value-%04d" "$id"
    sql+=" INSERT INTO bench_items VALUES ($id, '$value');"
  done
  printf '%s' "$sql"
}

insert_sql="$(build_insert_sql)"
select_sql="SELECT * FROM bench_items;"

assert_success_empty_stderr() {
  local scenario="$1"
  if [[ "$RUN_STATUS" -ne 0 ]]; then
    echo "error: $scenario exited with status $RUN_STATUS" >&2
    echo "$RUN_STDERR" >&2
    exit 1
  fi
  if [[ -n "$RUN_STDERR" ]]; then
    echo "error: $scenario wrote stderr" >&2
    echo "$RUN_STDERR" >&2
    exit 1
  fi
}

assert_select_output() {
  local line_count
  local first_row
  local last_row
  line_count="$(printf '%s' "$RUN_STDOUT" | awk 'END { print NR }')"
  first_row="$(printf '%s' "$RUN_STDOUT" | sed -n '2p')"
  last_row="$(printf '%s' "$RUN_STDOUT" | sed -n '$p')"

  if [[ "$(printf '%s' "$RUN_STDOUT" | sed -n '1p')" != "id|value" ]]; then
    echo "error: bench_reopen_select_1k stdout header mismatch" >&2
    exit 1
  fi
  if [[ "$line_count" -ne $((row_count + 1)) ]]; then
    echo "error: bench_reopen_select_1k expected $((row_count + 1)) lines, got $line_count" >&2
    exit 1
  fi
  if [[ "$first_row" != "1|value-0001" ]]; then
    echo "error: bench_reopen_select_1k first row mismatch: $first_row" >&2
    exit 1
  fi
  if [[ "$last_row" != "1000|value-1000" ]]; then
    echo "error: bench_reopen_select_1k last row mismatch: $last_row" >&2
    exit 1
  fi
}

rows_per_second() {
  local duration_ms="$1"
  awk -v rows="$row_count" -v ms="$duration_ms" 'BEGIN { printf "%.3f", rows * 1000 / ms }'
}

meets_threshold() {
  local observed="$1"
  local threshold="$2"
  awk -v observed="$observed" -v threshold="$threshold" 'BEGIN { exit !(observed >= threshold) }'
}

make_temp_db() {
  local temp_dir
  temp_dir="$(mktemp -d "${TMPDIR:-/tmp}/db-bench-acceptance.XXXXXX")"
  printf '%s/bench.db\n' "$temp_dir"
}

cleanup_temp_db() {
  local db_path="$1"
  rm -rf "$(dirname "$db_path")"
}

run_insert_iteration() {
  local iteration="$1"
  local measured="$2"
  local db_path
  local start_ns
  local end_ns
  local duration_ms
  local rps
  db_path="$(make_temp_db)"
  start_ns="$(now_ns)"
  run_db_exec "$db_path" "$insert_sql"
  end_ns="$(now_ns)"
  assert_success_empty_stderr "bench_insert_1k"
  cleanup_temp_db "$db_path"

  duration_ms="$(( (end_ns - start_ns) / 1000000 ))"
  if [[ "$duration_ms" -lt 1 ]]; then
    duration_ms=1
  fi
  rps="$(rows_per_second "$duration_ms")"
  if [[ "$measured" == "yes" ]]; then
    INSERT_ITERATIONS_JSON+="${INSERT_ITERATIONS_JSON:+,}{\"iterat
```

### docs/benchmarks.md
- excerpt_chars: 4000
- clipped: true

```text
# V1 Benchmark Acceptance

This document defines the repo-local benchmark evidence for V1 acceptance. The authoritative command is:

```bash
scripts/verify_bench_acceptance
```

The command writes machine-readable evidence to:

```text
target/bench_acceptance/v1-bench-docs-acceptance.json
```

The evidence id for final reporting is `evidence-v1-benchmark-lower-bounds`.

## Scope

The benchmark measures the existing single-process Rust CLI path by invoking `cargo run --quiet --bin db -- exec <temp-db> <sql>`. It does not add or call a public `db bench` command; `db bench` remains a reserved future CLI command and is not available for users.

The workload uses a deterministic temporary database and this table:

```sql
CREATE TABLE bench_items(id INT, value TEXT);
```

Each scenario uses 1,000 rows with `id` values from `1` through `1000` and `value` strings from `value-0001` through `value-1000`.

## Scenarios

| Scenario | Measured operation | Required validation | Lower-bound floor |
| --- | --- | --- | --- |
| `bench_insert_1k` | Create `bench_items(id INT, value TEXT)` and insert 1,000 rows through `db exec`. | The command exits successfully and stderr is empty. | `insert_rows_per_second >= 25` |
| `bench_reopen_select_1k` | Reopen the populated database in a new `db exec` process and run `SELECT * FROM bench_items;`. | The output has header `id|value`, 1,000 data rows, first row `1|value-0001`, last row `1000|value-1000`, and stderr is empty. | `select_rows_per_second >= 50` |

## Measurement Policy

Each scenario runs one warmup iteration that is not included in pass/fail evidence, followed by three measured iterations. Every measured iteration uses a fresh temporary database. The pass/fail rule uses the minimum measured rows per second, recorded as `observed_min_rows_per_second`; average, median, or best-case values are not sufficient. If any measured iteration is below its floor, `scripts/verify_bench_acceptance` exits non-zero.

These floors are acceptance lower bounds for the repo-local V1 smoke workload. They are intentionally conservative and do not claim throughput on arbitrary hardware.

## Current Evidence

The authoritative current-run values are the `observed_min_rows_per_second` fields in `target/bench_acceptance/v1-bench-docs-acceptance.json`. The latest local implementation evidence used for this acceptance update recorded:

| Scenario | Current observed minimum | Required floor | Interpretation |
| --- | --- | --- | --- |
| `bench_insert_1k` | `2793.296` rows/second | `insert_rows_per_second >= 25` | The minimum measured insert iteration exceeded the acceptance floor. |
| `bench_reopen_select_1k` | `4065.041` rows/second | `select_rows_per_second >= 50` | The minimum measured reopen/select iteration exceeded the acceptance floor. |

The final verifier should rerun `scripts/verify_bench_acceptance` and treat the regenerated JSON as the current source of truth if the numbers differ from this documented implementation-run snapshot.

## Environment Assumptions

The evidence is local-machine benchmark evidence. The generated JSON records the concrete OS, architecture, CPU, `rustc` version, `cargo` version, and logical CPU count for the run. The implementation-run snapshot above was produced on Darwin arm64 with an Apple M2 Max CPU and Rust/Cargo 1.84.0 toolchain. Different OS, CPU, thermal state, storage, or toolchain conditions may produce different measured values; acceptance is based on meeting the conservative floors in the current verifier run, not on reproducing the exact snapshot values.

## JSON Schema

The JSON artifact includes at least these top-level fields:

```text
schema_version
evidence_id
repo_sha
created_at
command
environment
policy
scenarios
overall_passed
```

`environment` records OS, architecture, `rustc` version, `cargo` version, logical CPU count, and CPU model when available.

Each `scenarios[]` entry includes:

```text
id
row_count
warmup_iterations
measured_iterations
threshold_rows_pe
```

### docs/v1_acceptance.md
- excerpt_chars: 4000
- clipped: true

```text
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

V1 remains a single-process Rust CLI database. This guide does not claim network service behavior, multi-process concurrency, distributed storage, public benchmark CLI support, mutation-maintained secondary-index behavior beyond append-only `INSERT`, or performanc
```

### work_queue/progress.md
- excerpt_chars: 4000
- clipped: true

```text
# Persistent DB Core Progress

## Current State

`persistent-db-core` now has the V1 CLI smoke contract, durable page storage, the minimal SQL schema/execute path, primary-key indexed lookup/ordered scan proof, disk-backed secondary-index equality/range proof, mutation-maintained secondary-index UPDATE/DELETE proof, current-SHA transaction WAL replay evidence for `db exec`, deterministic crash matrix coverage for WAL recovery boundaries, `db check` invariant validation for existing database files, SQLite-backed differential/property evidence for the supported SQL subset, and repo-local benchmark/acceptance documentation evidence. The next smallest implementation handoff should target the remaining V1 acceptance blocker on top of the SQL execution, recovery, check, differential, benchmark, and index baselines.

## Gap Snapshot

| gap_id | state | note |
| --- | --- | --- |
| gap-v1-bootstrap-cli-contract | missing_evidence | CLI skeleton exists, but the first CAO handoff should formalize the V1 command contract and smoke coverage. |
| gap-v1-page-storage-record-format | missing_evidence | No page storage or record format implementation yet. |
| gap-v1-sql-parser-schema-exec | verification_ready | `db exec <path> <sql>` implements the documented minimal SQL subset with deterministic tests, persistence coverage, and durable docs. |
| gap-v1-primary-btree-index | verification_ready | Primary-key tables rebuild an in-memory B-tree index from durable row records, support exact lookup, scan in primary-key order, and preserve row-only table compatibility. |
| gap-v1-secondary-index-range-scan | verification_ready | `CREATE INDEX` creates durable secondary `INT` indexes with indexed equality/range query paths, deterministic ordering, reopen/backfill/WAL replay coverage, and `db check` secondary-index invariant evidence. |
| gap-v1-secondary-index-mutation-consistency | verification_ready | Primary-key-targeted UPDATE/DELETE maintain table rows, primary indexes, and secondary indexes across restart, retained WAL replay, WAL-only mutation replay, positive `db check`, and deterministic stale/dangling/missing secondary-index negative fixtures. |
| gap-v1-transaction-wal-recovery | verification_ready | Current-SHA WAL sidecar replay proof covers committed mutation survival, rolled-back/uncommitted frame absence, incomplete-tail exclusion, and retained sidecar state after reopen. |
| gap-v1-deterministic-crash-matrix | verification_ready | Deterministic crash matrix covers pre-WAL append, partial WAL frame, uncommitted frame, committed replay idempotence, interrupted recovery retry, and corrupt tail cleanup evidence. |
| gap-v1-differential-property-tests | verification_ready | SQLite-backed deterministic differential/property tests cover supported SQL subset generation, duplicate-key errors, missing lookups, ordered scans, seed replay, and failure artifact reporting. |
| gap-v1-db-check-invariants | verification_ready | `db check <path>` validates existing page records, SQL catalog/row invariants, primary-key rebuildability, WAL sidecar ordering, missing paths, and directory-path open/read errors. |
| gap-v1-bench-docs-acceptance | verification_ready | `scripts/verify_bench_acceptance` records deterministic lower-bound JSON evidence, and `docs/v1_acceptance.md` maps launch gates to current evidence and explicit blockers. |

## Recent Entries

- 2026-05-19: Added mutation-maintained secondary-index proof for primary-key-targeted `UPDATE`/`DELETE`, including restart/reopen query evidence, retained and WAL-only replay coverage, positive `db check`, and deterministic stale/dangling/missing secondary-index negative fixtures.
- 2026-05-19: Added disk-backed secondary-index support for `CREATE INDEX`, indexed equality and inclusive `BETWEEN` range scans, deterministic key/tie-break ordering, reopen/backfill/WAL replay coverage, and `db check` secondary-index invariant validation.
- 2026-05-18: Added repo-local benchmark acceptance evidence with `s
```

### tests/bench_acceptance_contract.rs
- excerpt_chars: 4000
- clipped: true

```text
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
        ("gate-v1-disk-page-st
```

### tests/cli_contract.rs
- excerpt_chars: 3969
- clipped: false

```text
use std::process::{Command, Output};

const REQUIRED_HELP_LINES: &[&str] = &[
    "db - deterministic single-process V1 database CLI",
    "Usage:",
    "  db --help",
    "  db help",
    "  db exec <path> <sql>",
    "  db check <path>",
    "Supported commands:",
    "  help        Print this help text.",
    "  exec <path> <sql>",
    "  check <path>",
    "Reserved future commands:",
    "  open <path>",
    "  bench <path>",
    "V1 scope:",
    "  This build supports the CLI contract, page storage, and the documented minimal SQL subset.",
    "Non-goals:",
    "  No network server, multi-process concurrency, or distributed storage.",
];

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

fn assert_help_contract(output: &Output) {
    assert!(
        output.status.success(),
        "expected exit 0, got {:?}; stderr={:?}",
        output.status.code(),
        stderr(output)
    );
    assert_eq!("", stderr(output), "help stderr must be empty");

    let out = stdout(output);
    let mut search_from = 0usize;
    for line in REQUIRED_HELP_LINES {
        let relative = out[search_from..].find(line).unwrap_or_else(|| {
            panic!("missing help line after byte {search_from}: {line:?}\nstdout:\n{out}")
        });
        search_from += relative + line.len();
    }
}

#[test]
fn help_flag_prints_required_contract() {
    let output = db(&["--help"]);

    assert_help_contract(&output);
}

#[test]
fn help_subcommand_matches_help_flag() {
    let help_flag = db(&["--help"]);
    let help_subcommand = db(&["help"]);

    assert_help_contract(&help_flag);
    assert_help_contract(&help_subcommand);
    assert_eq!(stdout(&help_flag), stdout(&help_subcommand));
}

#[test]
fn unsupported_argument_reports_first_token() {
    let output = db(&["--unknown"]);

    assert_eq!(Some(2), output.status.code());
    assert_eq!("", stdout(&output), "unsupported stdout must be empty");
    assert_eq!(
        "error: unsupported argument or command: --unknown\nhint: run 'db --help' for the supported V1 CLI contract.\n",
        stderr(&output)
    );
}

#[test]
fn reserved_future_command_remains_unsupported() {
    let output = db(&["open", "demo.db"]);

    assert_eq!(Some(2), output.status.code());
    assert_eq!("", stdout(&output), "unsupported stdout must be empty");
    assert_eq!(
        "error: unsupported argument or command: open\nhint: run 'db --help' for the supported V1 CLI contract.\n",
        stderr(&output)
    );
}

#[test]
fn bench_reserved_future_command_remains_unsupported() {
    let output = db(&["bench", "demo.db"]);

    assert_eq!(Some(2), output.status.code());
    assert_eq!("", stdout(&output), "unsupported stdout must be empty");
    assert_eq!(
        "error: unsupported argument or command: bench\nhint: run 'db --help' for the supported V1 CLI contract.\n",
        stderr(&output)
    );
}

#[test]
fn exec_requires_path_and_single_sql_argument() {
    let output = db(&["exec", "demo.db"]);

    assert_eq!(Some(2), output.status.code());
    assert_eq!("", stdout(&output), "unsupported stdout must be empty");
    assert_eq!(
        "error: unsupported argument or command: exec\nhint: run 'db --help' for the supported V1 CLI contract.\n",
        stderr(&output)
    );
}

#[test]
fn check_requires_path_argument() {
    let output = db(&["check"]);

    assert_eq!(Some(2), output.status.code());
    assert_eq!("", stdout(&output), "unsupported stdout must be empty");
    assert_eq!(
        "error: unsupported argument or command: check\nhint: run 'db --help' for the supported V1 CLI contract.\n",
        stderr(&output)
    );
}
```

### docs/v1_spec.md
- excerpt_chars: 2497
- clipped: true

```text
# Autopilot V1 Spec: Persistent DB Core

## 1. Summary

V1 is an implementation test for a small SQLite-like database core.

The system must implement a persistent, page-based, disk-backed database with ordered indexes, single-process transactions, WAL-based recovery, deterministic crash simulation, invariant checking, differential/property-based tests, and basic performance constraints.

V1 is not an in-memory toy database. It is a test of whether Autopilot can design, implement, debug, and verify a complex persistent stateful system end to end.

## 2. Capability Being Tested

Passing V1 demonstrates:

- complex persistent stateful system implementation
- disk-backed data structure implementation
- transaction atomicity
- crash and recovery reasoning
- table/index consistency maintenance
- invariant-driven validation
- differential and property-based testing
- performance awareness sufficient to reject toy implementations

## 3. Required SQL Subset

### 3.1 Required Statements

The implementation must support the following SQL forms:

```sql
CREATE TABLE table_name (
  id INTEGER PRIMARY KEY,
  col1 INTEGER,
  col2 TEXT
);

CREATE INDEX index_name ON table_name (col1);

INSERT INTO table_name (id, col1, col2) VALUES (1, 10, 'hello');

SELECT * FROM table_name WHERE id = 1;

SELECT col1, col2 FROM table_name WHERE col1 = 10;

UPDATE table_name SET col1 = 20 WHERE id = 1;

DELETE FROM table_name WHERE id = 1;

BEGIN;
COMMIT;
ROLLBACK;
```

The parser may support only this subset, but unsupported SQL must fail with a clear error rather than crashing.

### 3.2 Required Predicates

The following `WHERE` predicates must be supported for primary key and indexed integer columns:

```sql
WHERE column = value
WHERE column < value
WHERE column <= value
WHERE column > value
WHERE column >= value
WHERE column BETWEEN a AND b
```

### 3.3 Explicitly Excluded SQL Features

The following are out of scope for V1:

- `JOIN`
- `GROUP BY`
- `HAVING`
- aggregation
- subqueries
- foreign keys
- `NULL`
- floating-point types
- concurrent transactions

## 4. Data Types

The implementation must support:

- `INTEGER`: signed 64-bit integer
- `TEXT`: UTF-8 string, maximum 1024 bytes

`NULL` is not supported.

## 5. Error Behavior

The system must return clear errors for:

- duplicate primary key insert
- access to a missing table
- access to a missing column
- unsupported SQL
- syntax errors
- type errors
- malformed database or WAL metadata found during `db check` or `db recove
```
