# Implementation Plan: v1-primary-btree-index

## Goal
Add primary-key B-tree index proof for persisted SQL rows: insertion duplicate checks, exact primary-key lookup, primary-key ordered scan, restart rebuild, negative/edge coverage, and durable docs.

## Non-Goals
- Secondary indexes.
- Query optimizer.
- ORDER BY or range predicates.
- Multi-column primary key.
- Non-INT primary key.
- Persisted index metadata or separate index files.
- Network services, daemons, multi-process concurrency, or browser/design evidence.

## Affected Contract Surface
- CLI behavior: db exec observable stdout/stderr/exit codes for primary-key SQL.
- Persisted-data compatibility: existing row-only SQL database files must continue to load.
- SQL grammar: add single INT PRIMARY KEY column declaration and exact primary-key WHERE lookup.
- Documentation: docs/file_format.md, docs/sql_subset.md, docs/cli_contract.md.
- Tests: tests/primary_index.rs and tests/sql_exec.rs.

## Implementation Boundary
| Area | Planned Change | Constraints |
|---|---|---|
| src/index.rs | New primary B-tree primitive over BTreeMap<i64, usize> | std only; deterministic lookup and ordered traversal |
| src/lib.rs | Export index module | keep small public surface |
| src/sql.rs | Track optional primary key metadata, rebuild index from records, enforce duplicates, route lookup/scan through index | no persisted index metadata; preserve non-PK insert-order SELECT |
| src/main.rs | Usually no change except help text if docs/tests require updated wording | preserve CLI error mapping |
| src/storage.rs | Prefer no change unless tests need fixture support exposed elsewhere | do not change page framing |
| tests/primary_index.rs | Add primitive and rebuild tests | include duplicate, missing, empty, restart/rebuild, corrupt row evidence |
| tests/sql_exec.rs | Add primary_key focused CLI behavior tests | assert stdout, stderr, and exit code |
| docs/*.md | Update SQL grammar, output ordering, persistence model, compatibility notes | mention no missing-index-metadata failure mode |

## Data Model Plan
- Column metadata gains primary key marker or Table gains primary_key_column: Option<usize>.
- Primary key is valid only for INT columns.
- Table stores rows in durable append order and optionally a PrimaryIndex mapping key -> row position.
- Existing old catalog records decode with no primary key.
- New catalog encoding must carry primary key metadata in a backward-compatible way for this project version.

## Query Path Mapping
- db exec opens PageStore and reads records.
- Database::from_records decodes catalog/row records.
- For each primary-key table, from_records rebuilds PrimaryIndex from row values.
- INSERT on a primary-key table probes PrimaryIndex before append_record.
- SELECT * FROM table WHERE pk = value uses PrimaryIndex::get to retrieve the row position.
- SELECT * FROM primary-key table uses PrimaryIndex::scan ascending row positions.
- SELECT * FROM non-primary-key table keeps existing rows Vec iteration order.

## Test Strategy
1. Add failing tests first for index primitive behavior in tests/primary_index.rs.
2. Add failing CLI tests in tests/sql_exec.rs with test names containing primary_key so cargo test --test sql_exec primary_key exercises the contract.
3. Implement minimal index module and SQL integration.
4. Update docs after behavior is green.
5. Run required verification commands.

## Required Verification
- ./scripts/verify
- cargo test --test primary_index
- cargo test --test sql_exec primary_key

## Acceptance Mapping
| Acceptance Item | Planned Evidence |
|---|---|
| INT PRIMARY KEY and exact WHERE grammar | tests/sql_exec.rs primary_key grammar/lookup tests |
| PK SELECT ordered by key | tests/sql_exec.rs ordered scan; tests/primary_index.rs ordered traversal |
| Non-PK SELECT remains insert-order | existing sql_exec test plus baseline ./scripts/verify |
| Persisted row index insert/find/scan | tests/primary_index.rs |
| Restart/reopen rebuild | tests/primary_index.rs rebuild test and tests/sql_exec.rs multi-command reopen test |
| SQL path uses index | tests cover duplicate pre-append and ordered scan; final implementation report maps execute_insert/execute_select to PrimaryIndex |
| Duplicate, missing, empty table | tests/primary_index.rs and tests/sql_exec.rs |
| Docs persistence model | docs/file_format.md, docs/sql_subset.md, docs/cli_contract.md review in final report |
| Required commands pass | command output summaries in final implementation report |

## Risks
- Catalog encoding compatibility: decoder must accept both old and new catalog records.
- Error classification drift: unsupported/malformed/semantic strings are stable test contracts.
- Internal visibility: tests/primary_index.rs may need public or crate-visible APIs without exposing unnecessary CLI surface.

## Stop Conditions
- Any required behavior needs a change to spec.md or contracts.md.
- Existing row-only SQL files cannot remain compatible without changing the approved file-format contract.
- A second recovery attempt is needed after verifier rejection.

