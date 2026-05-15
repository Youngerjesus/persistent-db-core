# Code Context Evidence

- available: true
- repo_root: /Users/jeongmin/Downloads/projects/autopilot-test/01-clear-answer-complex-software-db-core/persistent-db-core_worktree/main
- head_sha: f3fa75a95ba099d7145ab01175713b56664a25bb
- base_branch: main
- dirty_files: none
- collected_at: 2026-05-15T07:09:45.011471+00:00
- selected_files: src/main.rs, Cargo.toml, docs/v1_spec.md, docs/history_archives/history.md, work_queue/progress.md, AGENTS.md, .codex/skills/spec-creator/SKILL.md, .codex/skills/spec-reviewer/SKILL.md

## Omitted Reasons
- Cargo.lock: binary_or_large_suffix
- docs/cli_contract.md: not_git_tracked
- flow:cli-command-dispatch: not_git_tracked
- project_manager/specs/v1-bootstrap-cli-contract/contracts.md: not_git_tracked
- project_manager/specs/v1-bootstrap-cli-contract/review_loop/design.md: not_git_tracked
- project_manager/specs/v1-bootstrap-cli-contract/spec.md: not_git_tracked
- project_manager/tasks/tasks.json: not_git_tracked
- route:db-help: not_git_tracked
- ssot/current-artifact.md: not_git_tracked
- ssot/current-objective.md: not_git_tracked
- ssot/current-plan.md: not_git_tracked
- tests/cli_contract.rs: not_git_tracked

## File Excerpts

### src/main.rs
- excerpt_chars: 729
- clipped: false

```text
use std::env;
use std::process;

const HELP: &str = "\
db 0.1.0

Usage:
  db --help
  db help

Options:
  -h, --help    Print this help message.

V1 persistent-db-core is currently bootstrapped as a CLI skeleton. Future gaps will add page storage, SQL execution, indexes, transactions, WAL recovery, crash testing, differential tests, invariant checks, and benchmarks.
";

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    if args.is_empty() || args.iter().any(|arg| arg == "-h" || arg == "--help") || args == ["help"] {
        print!("{HELP}");
        return;
    }

    eprintln!("db: unsupported arguments: {}", args.join(" "));
    eprintln!("Run `db --help` for usage.");
    process::exit(2);
}
```

### Cargo.toml
- excerpt_chars: 131
- clipped: false

```text
[package]
name = "persistent-db-core"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "db"
path = "src/main.rs"

[dependencies]
```

### docs/v1_spec.md
- excerpt_chars: 4000
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
- malformed database or WAL metadata found during `db check` or `db recover`

Syntax errors and unsupported SQL must not crash the process.

## 6. Storage Requirements

The database must use disk-backed persistent storage.

Required properties:

- Data must be stored in a disk file.
- Storage must be page-based.
- Default page size must be 4096 bytes.
- Data must survive process restart.
- The implementation must not keep all data only in memory and dump it at the end.
- The implementation must not rewrite the entire database file on every operation.
- Reads and writes must operate through page-level storage abstractions.

The following internal details are intentionally left to Autopilot:

- page header format
- record layout
- free page management
- overflow page design
- buffer/cache design
- page allocation policy
- compaction strategy

## 7. Index Requirements

The database must use disk-backed ordered indexes.

Required properties:

- `INTEGER PRIMARY KEY` must be implemented as a disk-backed ordered index.
- The ordered index must be a B+Tree or an equivalent ordered page-based structure.
- `CREATE INDEX` must create a secondary index.
- Equality lookup and range scan must use indexes where applicable.
- Insert, update, and delete must keep table rows and indexes consistent.
- Indexes must survive process restart.

Required invariants:

- primary key uniqueness
- B+Tree ordering
- correct leaf scan order
- secondary index entry matches the referenced table row
- no dangling index pointer
- no visible row missing from the required index

## 8. T
```

### docs/history_archives/history.md
- excerpt_chars: 243
- clipped: false

```text
# Persistent DB Core History

## 2026-05-15

- Created `persistent-db-core` as a V1 managed repo for CAO Autopilot.
- Initial product boundary is a Rust CLI binary named `db`.
- No V1 implementation gaps have verified completion evidence yet.
```

### work_queue/progress.md
- excerpt_chars: 1318
- clipped: false

```text
# Persistent DB Core Progress

## Current State

`persistent-db-core` is bootstrapped as a Rust CLI skeleton. The next smallest implementation handoff should target `gap-v1-bootstrap-cli-contract`.

## Gap Snapshot

| gap_id | state | note |
| --- | --- | --- |
| gap-v1-bootstrap-cli-contract | missing_evidence | CLI skeleton exists, but the first CAO handoff should formalize the V1 command contract and smoke coverage. |
| gap-v1-page-storage-record-format | missing_evidence | No page storage or record format implementation yet. |
| gap-v1-sql-parser-schema-exec | missing_evidence | No SQL parser, schema catalog, or executor yet. |
| gap-v1-primary-btree-index | missing_evidence | No primary B-tree index yet. |
| gap-v1-secondary-index-range-scan | missing_evidence | No secondary index support yet. |
| gap-v1-transaction-wal-recovery | missing_evidence | No transaction or WAL recovery path yet. |
| gap-v1-deterministic-crash-matrix | missing_evidence | No deterministic crash matrix yet. |
| gap-v1-differential-property-tests | missing_evidence | No SQLite differential/property test harness yet. |
| gap-v1-db-check-invariants | missing_evidence | No `db check` invariant command yet. |
| gap-v1-bench-docs-acceptance | missing_evidence | No benchmark lower-bound evidence or V1 acceptance docs yet. |
```

### AGENTS.md
- excerpt_chars: 1480
- clipped: false

```text
# AGENTS (persistent-db-core)

This repository is the managed product repo for the V1 persistent database core. Product code and repo-local context live here. Autopilot orchestration, scheduling, reports, and runtime state live in the sibling `autopilot/` control-plane repository.

## Product Direction

Build a small, deterministic Rust CLI database binary named `db`.

V1 must prove durable single-process database behavior through:

- CLI contract and smoke behavior.
- Disk page storage and record format.
- SQL parser, schema catalog, and basic execution.
- Primary B-tree index.
- Secondary index range scans.
- Transactions, WAL, and recovery.
- Deterministic crash matrix.
- SQLite differential/property tests.
- `db check` invariant validation.
- Benchmark and acceptance documentation.

## Engineering Rules

- Keep changes scoped to the current CAO gap and its spec package.
- Prefer deterministic tests and explicit on-disk fixtures over implicit state.
- Treat file format, WAL, and recovery behavior as compatibility-sensitive once introduced.
- Do not add network services, background daemons, or remote dependencies for V1.
- Do not store Autopilot runtime state in this repository.

## Verification Baseline

- `cargo test`
- `cargo run --bin db -- --help`

## Repo-Local Skills

`.codex/skills/spec-reviewer/SKILL.md` and `.codex/skills/spec-creator/SKILL.md` are repo-local product context contracts for V1 DB work. They are not the generic Autopilot runtime.
```

### .codex/skills/spec-creator/SKILL.md
- excerpt_chars: 881
- clipped: false

```text
# V1 Persistent DB Spec Creator

Use this repo-local contract when creating specs for `persistent-db-core`.

## Spec Creation Rules

- Select the smallest coherent slice that advances one current V1 gap.
- Preserve the `db` binary contract and keep the implementation in Rust.
- Include scope, non-goals, expected behavior, edge cases, and concrete acceptance criteria.
- Include deterministic verification commands, starting with `cargo test` and a relevant `cargo run --bin db -- ...` smoke check.
- For storage and recovery features, name the on-disk artifacts and corruption or crash cases that must be tested.
- For SQL and index features, define input statements, expected rows, and ordering guarantees.

## Non-Goals For V1 Specs

- Networked database server behavior.
- Multi-process concurrency.
- Distributed storage.
- Query optimization beyond the V1 acceptance gates.
```

### .codex/skills/spec-reviewer/SKILL.md
- excerpt_chars: 916
- clipped: false

```text
# V1 Persistent DB Spec Reviewer

Use this repo-local contract when reviewing specs for `persistent-db-core`.

## Review Focus

- Confirm the spec maps to exactly one V1 gap from `docs/v1_spec.md`, `docs/history_archives/history.md`, or `work_queue/progress.md`.
- Require observable CLI behavior, deterministic tests, and clear evidence for storage, recovery, or query semantics.
- Reject broad rewrites that combine unrelated gaps unless the dependency is unavoidable and explicitly justified.
- For file format, WAL, transaction, and crash behavior, require compatibility notes and failure-mode tests.
- For SQL or index behavior, require examples that state expected output ordering and error behavior.

## Required Reviewer Output

- Verdict: approved, needs_revision, or blocked.
- Missing acceptance criteria, if any.
- Required verification commands.
- Evidence paths that should exist after implementation.
```
