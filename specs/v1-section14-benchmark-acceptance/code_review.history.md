## 2026-05-19T22:04:24+09:00 - Archived Latest Report Before Code Review Verify 2

# Code Review: v1-section14-benchmark-acceptance

Verdict: FAIL

## Scope

- Reviewed target: current worktree changes for `task-2026-05-19-15-18-42-v1-section14-benchmark-acceptance`.
- `git log --oneline main..HEAD`: no commits; review target is the uncommitted diff plus new files.
- Dirty files reviewed: `src/bench.rs`, `src/main.rs`, `src/lib.rs`, `src/sql.rs`, `src/storage.rs`, `scripts/verify_bench_acceptance`, benchmark/CLI/SQL tests, and Section 14 docs.
- Required context read: `spec.md`, `contracts.md`, and `qa_mapping.md`.
- Focus: correctness, regression risk, architecture boundaries, security, maintainability, additive/proxy-success risk, performance evidence fidelity, and database persistence behavior.

## Specialist Routing

| Reviewer | Trigger | Status | Evidence Source | Accepted Finding IDs | Rejected Finding IDs | Skip/Fallback Reason |
| --- | --- | --- | --- | --- | --- | --- |
| `code-reviewer` | Correctness, completeness, spec mismatch, merge safety | fallback-applied | Agent `019e4032-94fa-72c0-b3c2-7bd8b04d476d` invoked but did not return before timebox; main reviewer applied correctness/spec lens to diff and QA mapping. | CR-001, CR-002, CR-003 | None | Timeboxed companion output unavailable; local review completed to avoid unfinished gate. |
| `testing-reviewer` | Coverage gaps, negative paths, flakiness, merge confidence | invoked | Agent `019e4032-954a-7321-9b21-a178ea88b32b`: reported missing pre-verifier provenance assertions, partial verifier stdout/failure-path coverage, under-tested docs evidence, and slow lock-serialized benchmark tests. | CR-003, CR-006 | TEST-DOC-AUTOMATION, TEST-SLOW-LOCKS | Doc automation and slow lock-serialized tests remain residual because manual doc review is contract-allowed and runtime cost is expected for benchmark acceptance. |
| `security-reviewer` | CLI/file/process boundaries, shell script, public evidence path | invoked | Agent `019e4032-95be-7161-a4de-df7cb04d08b0`: reported symlink overwrite via `fs::write`, PATH-based `kill`, and local absolute path leakage. | CR-004, CR-005 | SEC-LOW-ABS-PATHS | Absolute paths in task review artifacts are pre-existing/generated phase context, not product/runtime code; keep as residual hygiene, not merge blocker for this code-review gate. |
| `performance-reviewer` | Benchmark loops, resource risk, recovery metric validity | invoked | Agent `019e4032-9667-7833-8bec-700b48e3c04e`: reported recovery metric does not exercise actual WAL replay, synthetic no-retry evidence, and stale test lock risk. | CR-001 | PERF-WARN-NO-RETRY, PERF-WARN-TEST-LOCK | No-retry row is weak but the verifier has no retry loop; stale test lock is CI hygiene. Both are residual unless repair touches the same code. |
| `maintainability-reviewer` | Complexity, duplication, brittle structure | fallback-applied | No matching specialist role available in this runtime; main reviewer inspected shell/Rust duplication and validator shape. | CR-003 | None | Specialist unavailable. |
| `red-team-reviewer` | Additive bias, proxy-success evidence, cross-category gaps | fallback-applied | No matching specialist role available; main reviewer applied proxy-success lens to benchmark/recovery evidence. | CR-001, CR-002 | None | Specialist unavailable. |
| `database-reviewer` | SQL, storage, WAL, persistence boundaries | fallback-applied | No matching specialist role available; main reviewer inspected `src/sql.rs`, `src/storage.rs`, WAL replay, and fixture creation. | CR-001, CR-002 | None | Specialist unavailable. |
| `api-reviewer` | Endpoint/transport/schema API changes | skipped | No HTTP/API endpoint or transport contract changed. | None | None | Not triggered. |
| `ui-ux-reviewer` | UI/component/accessibility changes | skipped | No UI surface changed. | None | None | Not triggered. |

## Findings

### CR-001 - WAL recovery evidence does not replay the 10,000 committed WAL workload

- Severity: High
- Files: `src/bench.rs:388`, `src/sql.rs:458`, `src/sql.rs:643`, `src/storage.rs:398`
- Requirement impact: `METRIC-14-4`, recovery proportionality, Section 14 hard-fail evidence.

`run_recovery_evidence()` builds the recovery fixture by calling `create_section14_fixture_for_bench()`, which appends the catalog, index metadata, and all 10,000 rows directly into the main page file through `PageStore::append_record()`. That same append path also writes committed WAL frames, but the data is already durable before the measured reopen. On reopen, `replay_wal()` starts with `current_record_count = total_record_count(path)` and applies a frame only when `current_record_count == record_count_before`; for this fixture, the durable record count is already ahead, so replay scans/skips already-applied frames instead of applying the 10,000 committed workload. The measured `sql::execute()` path then rebuilds `Database::from_records(...)` from the page file and performs a lookup.

This means the JSON field `recovery.committed_transaction_count=10000` and the documented `recovery_ms` are proxy evidence for reopening an already-applied database, not evidence that WAL recovery applied 10,000 committed transactions proportionally. The fix should build a recovery fixture where the 10,000 committed WAL records are absent from the page file before reopen, then time the operation that forces `replay_wal()` to apply them and validate recovered row count plus representative lookup.

### CR-002 - Sequential insert metric bypasses the public SQL insert/index-maintenance path

- Severity: High
- Files: `src/bench.rs:374`, `src/sql.rs:643`
- Requirement impact: `METRIC-14-1`, `METRIC-14-2`, workload contract, evidence fidelity.

`sequential_insert_elapsed_ms` is timed around `sql::create_section14_fixture_for_bench(...)`. That helper writes encoded catalog/index metadata and pre-indexed row records directly through storage records. It does not execute the contracted table schema through the normal SQL `CREATE TABLE`, `CREATE INDEX`, and sequential `INSERT` path, and it bypasses normal parser/semantic/index-maintenance work. As a result, the reported insert elapsed time and throughput do not prove the performance of the public database write path that users exercise through `db exec` and that Section 14 describes as sequential inserts over the fixed schema.

The benchmark may use internal helpers for setup where the contract permits it, but the metric named and documented as sequential insert throughput must be tied to the same database write semantics it claims to measure, or the evidence should be renamed/scoped and the contract updated. Since this contract is already approved, repair should make the benchmark workload perform and measure actual sequential inserts through the normal SQL/storage path while preserving the fixed fixture constants.

### CR-003 - Verifier lock can proceed without owning the lock after timeout

- Severity: Medium
- File: `scripts/verify_bench_acceptance:62`
- Requirement impact: deterministic evidence generation, outside-cwd verifier reliability.

The verifier lock loop attempts `mkdir "$verifier_lock_dir"` up to 3,600 times, but after the loop it only checks whether `$verifier_lock_dir/pid` exists. If another live process holds the lock for the full timeout, the pid file still exists, so the script falls through and runs without ever setting the `trap` or owning the lock. That can race two benchmark verifiers against the fixed evidence path, including the explicit `rm -f "$evidence_path"` at line 76, and can produce nondeterministic pass/fail artifacts.

Track ownership explicitly, for example with `lock_acquired=1` only after successful `mkdir`, and fail if ownership was not acquired. This also needs a focused script/test coverage path for a live lock holder.

### CR-004 - Public `db bench` evidence write follows symlinks

- Severity: High
- File: `src/bench.rs:124`
- Requirement impact: public CLI file boundary security.

`run_section14_benchmark()` writes the public evidence artifact with `fs::write(path, evidence)`. That open-and-truncate path follows symlinks. A malicious or accidental symlink at `target/bench_acceptance/section14-benchmark-acceptance.json` can cause `db bench` to overwrite another user-writable file. The verifier already uses a same-directory temp file plus `os.fsync` and `os.replace`; the public CLI writer should use equivalent atomic finalization and reject or safely replace symlinks.

### CR-005 - Stale lock detection shells out through `PATH`

- Severity: Medium
- File: `src/bench.rs:320`
- Requirement impact: public CLI process boundary security.

On Unix systems without `/proc`, including the current macOS environment, stale-lock detection executes `Command::new("kill").arg("-0")`. Because `kill` is resolved through `PATH`, a hostile environment can execute an attacker-controlled binary during `db bench` if a stale lock pid exists. Use a direct OS call, a standard-library-only non-shell approach, or an absolute trusted path rather than resolving `kill` from `PATH`.

### CR-006 - Writer/verifier separation and verifier failure contract are under-tested

- Severity: Medium
- Files: `tests/bench_acceptance.rs:149`, `tests/bench_acceptance.rs:303`, `tests/bench_acceptance.rs:336`
- Requirement impact: writer/validator separation, `DB_BENCH` and `BENCH_ACCEPTANCE` stdout/exit-code contract.

The tests for a bare `db bench` run assert schema keys and command success, but they do not assert the pre-verifier state required by the QA mapping: `commands.verify_bench_acceptance.status="pending"` and `result="pending"` before the verifier finalizes the file. The verifier tests also use `stdout.contains(VERIFY_SENTINEL)` instead of exact stdout equality, and there is no black-box failure-path test proving `BENCH_ACCEPTANCE: FAIL check=<check_id> reason=<reason>` with a non-zero exit. That leaves the public evidence lifecycle and verifier stdout contract able to drift while tests still pass.

Repair should add focused assertions for pre-verifier provenance, exact verifier stdout on success, and at least one black-box verifier failure case with the required sentinel shape.

## Must Fix Now

- CR-001: Make recovery evidence force real WAL replay of the 10,000 committed workload before claiming `recovery_ms`, `recovered_row_count`, and proportionality.
- CR-002: Make the sequential insert metric measure the real SQL/database insert path, or otherwise align the implementation with the approved workload contract without weakening the contract.
- CR-003: Fix verifier lock ownership so timeout cannot run the verifier without holding the lock.
- CR-004: Make `db bench` evidence finalization symlink-safe and atomic.
- CR-005: Remove `PATH`-resolved `kill` execution from public CLI stale-lock detection.
- CR-006: Add coverage for pre-verifier pending evidence state, exact verifier stdout, and verifier failure sentinel shape.

## Residual Risks

- `hard_fail_checks.no_retry_required` is currently a hardcoded pass row. The verifier has no retry loop, so this is not accepted as a merge blocker here, but the evidence would be stronger if it recorded an explicit attempt count.
- Benchmark test helpers can wait up to six minutes on stale `.section14.test.lock` directories. This is CI hygiene unless the same repair touches the lock helpers.
- `docs/performance_report.md` and `docs/bug_diary.md` are required manual-review artifacts but are not pinned by automated doc tests. This remains residual because the contract permits manual doc review plus `scripts/verify`.
- Full `scripts/verify_bench_acceptance` and `scripts/verify` were not rerun during this code-review pass. A focused validator smoke ran instead: `cargo test --test bench_acceptance hard_fail_validator` passed with 8 tests.
- The first attempted focused cargo command used invalid multiple test-name syntax and failed before running tests; the corrected filtered command above passed.

## Next Action

Return to `code_review_retry` for implementation repair. After repair, rerun `scripts/verify_bench_acceptance` from repo root, the outside-cwd absolute verifier invocation, and baseline `scripts/verify`, then refresh this review gate.

## Updated At

2026-05-19T21:37:34+09:00
## 2026-05-19T22:21:51+09:00 - Archived Latest Report Before Code Review Verify 3 PASS Refresh

# Code Review Verification: v1-section14-benchmark-acceptance

Verdict: FAIL

## Scope

- Verification target: current worktree for `task-2026-05-19-15-18-42-v1-section14-benchmark-acceptance`, including uncommitted tracked changes and untracked task files.
- `git log --oneline main..HEAD`: no commits; all implementation/review changes are in the worktree.
- Reviewed latest `code_review.md`, archived the previous report to `code_review.history.md`, and independently checked the prior `Must Fix Now` items against the current code.
- Confirmed prior CR-001 through CR-005 appear repaired in code: recovery fixture now writes 10,000 committed WAL records for replay, sequential insert timing uses the SQL path, verifier lock ownership is explicit, `db bench` evidence finalization is atomic/symlink-aware, and public Rust stale-lock handling no longer shells out through `Command::new("kill")`.
- Executed required gates: `scripts/verify_bench_acceptance`, outside-cwd absolute invocation of `scripts/verify_bench_acceptance`, and baseline `scripts/verify`.

## Specialist Routing

| Reviewer | Trigger | Status | Evidence Source | Accepted Finding IDs | Rejected Finding IDs | Skip/Fallback Reason |
| --- | --- | --- | --- | --- | --- | --- |
| `code-reviewer` | Code review verification of latest report, diff scope, and merge safety | fallback-applied | Current verify phase did not start new subagent reviews per phase instruction; main verifier checked `git status --short`, `git log --oneline main..HEAD`, prior report, implementation files, tests, and command results. | CRV-001 | None | New subagent review is explicitly disallowed in this verify phase. |
| `testing-reviewer` | Prior CR-006 test coverage repair verification | fallback-applied | `rg -n "contains\\(VERIFY_SENTINEL\\)|BENCH_ACCEPTANCE: FAIL|\\\"status\\\":\\\"pending\\\""` over benchmark tests/scripts plus `scripts/verify` output. | CRV-001 | None | New subagent review is explicitly disallowed in this verify phase. |
| `security-reviewer` | Prior public CLI file/process boundary findings CR-004/CR-005 | fallback-applied | `src/bench.rs` now uses `atomic_write_evidence`, `symlink_metadata`, same-directory temp file with `create_new`, flush/sync, and no `Command::new("kill")`; `tests/bench_acceptance_contract.rs` pins those constraints. | None | CR-004, CR-005 | Findings verified fixed; no new security specialist invocation allowed. |
| `performance-reviewer` | Prior benchmark/recovery evidence findings CR-001/CR-002 | fallback-applied | `src/bench.rs` uses `sql::execute(path, &section14_insert_sql(rows))`; `src/sql.rs` has `create_section14_wal_recovery_fixture_for_bench`; `scripts/verify_bench_acceptance` passed from repo root and outside cwd. | None | CR-001, CR-002 | Findings verified fixed; no new performance specialist invocation allowed. |
| `database-reviewer` | SQL/storage/WAL replay boundary verification | fallback-applied | `src/sql.rs` WAL recovery fixture appends committed WAL records after durable catalog/index metadata, and baseline WAL/secondary-index tests passed under `scripts/verify`. | None | CR-001 | Finding verified fixed; no matching specialist role invoked in this verify phase. |
| `api-reviewer` | HTTP/API endpoint changes | skipped | No HTTP/API endpoint or transport contract changed. | None | None | Not triggered by diff scope. |
| `ui-ux-reviewer` | UI/component/accessibility changes | skipped | No UI surface changed. | None | None | Not triggered by diff scope. |

## Findings

### CRV-001 - CR-006 remains partially unrepaired in benchmark verifier tests

- Severity: Medium
- Files: `tests/bench_acceptance.rs:306`, `tests/bench_acceptance.rs:339`
- Requirement impact: writer/validator separation, exact verifier stdout contract, black-box failure sentinel contract.

The prior code-review finding CR-006 required focused assertions for the pre-verifier evidence state, exact verifier stdout on success, and at least one black-box verifier failure path proving `BENCH_ACCEPTANCE: FAIL check=<check_id> reason=<reason>` with a non-zero exit.

Current tests still check verifier success with `output.stdout.contains(VERIFY_SENTINEL)` in both root and outside-cwd verifier tests, so additional stdout could be added without failing tests. A targeted `rg` check found only static/documentation references to `BENCH_ACCEPTANCE: FAIL check=` and script implementation lines; no black-box test corrupts or withholds evidence to assert the verifier failure sentinel and non-zero exit. The `db bench` implementation does write pending verifier state (`build_evidence_json(&run, "pending", "pending")`), but the black-box `db bench` test does not assert `commands.verify_bench_acceptance.status="pending"` or `result="pending"` before the verifier finalizes the file.

Automated gates pass, so this is not a runtime regression, but the latest review repair is incomplete relative to the accepted testing-review finding and QA mapping provenance contract. Per this phase's golden rule, incomplete review repair and report drift require `retry`.

## Must Fix Now

- CRV-001: Strengthen `tests/bench_acceptance.rs` so bare `db bench` asserts pre-verifier `commands.verify_bench_acceptance.status="pending"` and `result="pending"`.
- CRV-001: Replace verifier success `stdout.contains(VERIFY_SENTINEL)` checks with exact stdout equality, including the trailing newline contract used by command output.
- CRV-001: Add a black-box verifier failure-path test that proves `scripts/verify_bench_acceptance` exits non-zero and prints `BENCH_ACCEPTANCE: FAIL check=<check_id> reason=<reason>` for a controlled invalid evidence or validation failure path.

## Residual Risks

- `scripts/verify_bench_acceptance` passed twice and `scripts/verify` passed, but those gates do not currently catch the incomplete CR-006 test obligations above.
- `hard_fail_checks.no_retry_required` remains a synthetic pass row. This was previously accepted as residual because the verifier has no retry loop.
- Runtime cost is high: `scripts/verify` took benchmark acceptance tests through repeated 100k benchmark runs; this is expected for this task but remains CI cost.
- Python-specific `ruff check`, `mypy`, and `pytest` were not applicable to this Rust repository; the relevant static/test gate is `scripts/verify` (`cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`, and `cargo run --bin db -- --help`), which passed.

## Next Action

Return to `code_review_retry` to complete CRV-001 test coverage repair, then rerun `scripts/verify_bench_acceptance`, outside-cwd absolute verifier invocation, and `scripts/verify`. After repair, refresh this code-review report with only current open findings.

## Updated At

2026-05-19T22:04:24+09:00
