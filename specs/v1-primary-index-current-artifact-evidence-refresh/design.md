# Technical Design

## Design Summary
The implementation should make current-artifact primary-index evidence explicit while preserving the existing Rust CLI database architecture. The intended shape is additive: exact tests, one focused verification script, and evidence mapping. Source changes are only justified if focused tests expose a real current-contract gap.

## Evidence Test Design

### PrimaryIndex Primitive Evidence
- Instantiate `PrimaryIndex::new()`.
- Insert `2 -> 0` and `1 -> 1`.
- Assert exact lookup behavior:
  - `get(2) == Some(0)`
  - `get(1) == Some(1)`
  - `get(3) == None`
- Attempt `insert(2, 99)`.
- Assert the insert returns `Err(_)` and `get(2)` remains `Some(0)`.
- For ordered traversal, insert `30 -> 0`, `-5 -> 1`, `10 -> 2`; assert `ordered_positions() == [1, 2, 0]`.
- For empty traversal, assert a fresh index returns an empty `Vec<usize>`.
- Map this test evidence to `REQ-7-implement-integer-primary-key-as-9c698e08`.

### Persisted SQL Rebuild Evidence
- Use a temp database path.
- In one `db exec` process, execute:
  - `CREATE TABLE users (id INT PRIMARY KEY, name TEXT);`
  - `INSERT INTO users VALUES (2, 'bea');`
  - `INSERT INTO users VALUES (1, 'ada');`
  - `INSERT INTO users VALUES (3, 'cal');`
- In later `db exec` calls against the same path, assert:
  - `SELECT * FROM users WHERE id = 2;` exits `0`, stderr is empty, stdout is `id|name\n2|bea\n`;
  - `SELECT * FROM users;` exits `0`, stderr is empty, stdout is `id|name\n1|ada\n2|bea\n3|cal\n`.
- This proves reopen/rebuild from persisted rows and deterministic primary-key ordering.

### Combined CLI SQL Evidence
- In `tests/sql_exec.rs`, use one command with this exact SQL:
  - `CREATE TABLE users (id INT PRIMARY KEY, name TEXT); INSERT INTO users VALUES (2, 'bea'); INSERT INTO users VALUES (1, 'ada'); INSERT INTO users VALUES (3, 'cal'); SELECT * FROM users; SELECT * FROM users WHERE id = 2; SELECT * FROM users WHERE id = 9;`
- Assert:
  - exit code `0`;
  - stderr `""`;
  - stdout `id|name\n1|ada\n2|bea\n3|cal\nid|name\n2|bea\nid|name\n`.
- Use a test name containing `primary_key` so `cargo test --test sql_exec primary_key` exercises it.

### Same-Path Reopen Evidence
- Reuse a database path populated by one `db exec` process.
- Open a new `db exec` process against the same path.
- Assert `SELECT * FROM users;` and `SELECT * FROM users WHERE id = 2;` return the same ordered scan and exact lookup results.
- Keep this test black-box; it should not rely on private executor internals.

### Duplicate Insert Preservation Evidence
- Populate `users` with primary key `2` and payload `bea`.
- Run duplicate insert `INSERT INTO users VALUES (2, 'dupe');`.
- Assert:
  - exit code `2`;
  - stdout `""`;
  - stderr `error: SQL semantic error: duplicate primary key for table users: 2\nhint: primary key values must be unique.\n`.
- Run a follow-up select to prove existing row state is unchanged:
  - `SELECT * FROM users WHERE id = 2;` returns `id|name\n2|bea\n`.

### Persisted Duplicate Fixture Evidence
- Build records through fixture helpers rather than malformed bytes:
  - valid SQL storage catalog record for table `users` with columns `id INT PRIMARY KEY` and `name TEXT`;
  - valid row record with values `2`, `bea`;
  - valid row record with values `2`, `dupe`.
- Append records through `PageStore::append_record`.
- Reopen through a fresh `db exec` process.
- Assert:
  - exit code `1`;
  - stdout `""`;
  - stderr `error: invalid SQL storage record: duplicate primary key for table users: 2\nhint: primary key values must be unique in persisted SQL storage.\n`.
- This test must not use malformed record tags, unknown tags, broken prefixes, or corrupted length fields as substitutes.

## Script Design
Add `scripts/verify_primary_index_acceptance` as a repo-root portable Bash script:

```bash
#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

cargo test --test primary_index
cargo test --test sql_exec primary_key
```

The script should be executable and deterministic. It supplements but does not replace `scripts/verify`.

## Evidence Artifact Design
- `qa_mapping.md` should map every scenario above to:
  - `gate-v1-indexes`;
  - `REQ-7-implement-integer-primary-key-as-9c698e08`;
  - specific test names or manual review evidence;
  - required command(s).
- `final_review.md` should include:
  - current managed repo SHA;
  - command table with exit code and pass/fail for `cargo test --test primary_index`, `cargo test --test sql_exec primary_key`, and `scripts/verify`;
  - focused script result if `scripts/verify_primary_index_acceptance` is added;
  - explicit non-claim statement for the three excluded requirement IDs.
- `docs/v1_acceptance.md`, if updated, should add or update only the primary-index current artifact row and include the final evidence path and current SHA.

## Compatibility
- No persisted format change is intended.
- No new public CLI syntax is intended.
- Existing CLI output and documented error strings remain stable unless a focused test exposes a mismatch with the approved current contract; any repair must preserve the contract text exactly.

