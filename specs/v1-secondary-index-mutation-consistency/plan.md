# Implementation Plan: v1-secondary-index-mutation-consistency

## Goal
Add contract-complete evidence that supported `UPDATE` and `DELETE` mutations keep table rows, primary-key lookup, secondary equality/range indexes, WAL replay, and `db check` invariants consistent.

## Non-Goals
- General SQL update/delete support beyond the approved primary-key targeted fixture shape.
- Multi-row mutation predicates, joins, projection, order by, transactions, unique secondary indexes, or text secondary indexes.
- Lower-level page/WAL framing changes.
- Network service behavior, multi-process concurrency, browser evidence, screenshots, visual evidence, or UX design-review evidence.
- Editing `ssot/` or `policies/`.

## Affected Contract Surface
| Surface | Planned Change |
|---|---|
| CLI behavior | `db exec <path> "UPDATE ...;"` and `db exec <path> "DELETE ...;"` succeed silently for the supported forms; unsupported forms retain documented error behavior. |
| SQL grammar | Add narrow `UPDATE <table> SET <column> = <value> WHERE <primary_key> = <int>;` and `DELETE FROM <table> WHERE <primary_key> = <int>;`. |
| Persisted data | Add SQL logical mutation records above existing `PageStore`; preserve lower-level page/WAL format and existing no-index/secondary-index reopen compatibility. |
| Index invariants | Mutation replay must maintain visible rows, primary index, and committed secondary index entries together. |
| `db check` | Positive mutated database passes; stale entry, dangling/deleted pointer, and missing indexed visible row fixtures fail as `secondary index`. |
| Tests | Add focused black-box process-boundary tests and deterministic fixture-builder corruption tests. |
| Docs | Update CLI/SQL/file-format docs only for the implemented mutation contract and new SQL logical-record compatibility. |

## Implementation Boundary
| Area | Planned Change | Constraints |
|---|---|---|
| `src/sql.rs` | Add `Statement::Update`, `Statement::Delete`, mutation record decoding/encoding, replay, execution, and invariant validation | Keep parser narrow; no broad optimizer or table-scan mutation semantics |
| `src/index.rs` | Add removal/update helper only if needed by mutation maintenance | Preserve deterministic `BTreeMap` ordering and duplicate detection |
| `src/check.rs` | Prefer no structural change; continue using SQL validation labels | Failure label must stay exactly `secondary index` for required negative cases |
| `src/main.rs` | No planned change unless help or error hint needs documentation alignment | Preserve exit-code mapping |
| `tests/secondary_index.rs` | Add contract fixture, process-boundary restart/WAL tests, and three negative fixture builders | Required focused command target |
| `tests/sql_exec.rs` | Add narrow grammar/error regression coverage only if broader SQL contract coverage belongs there | Avoid duplicating all secondary-index scenarios |
| `tests/db_check.rs` | Use only if negative fixture coverage is moved out of `tests/secondary_index.rs` | If touched, required focused command expands to `cargo test --test db_check -- --nocapture` |
| `docs/cli_contract.md` | Document supported mutation forms, stdout/stderr, process examples, and db check output | Stable user-facing contract |
| `docs/sql_subset.md` | Document grammar and SQL logical mutation records | Must stop saying UPDATE/DELETE are out of scope after implementation |
| `docs/file_format.md` | Document new SQL logical records and compatibility note | State lower-level page/WAL format unchanged |

## Data Model Plan
- Preserve table catalog and row values in durable row-position order.
- Introduce an internal visible-row state so historical row positions remain stable after delete/update.
- Primary-key indexes map keys only to visible row positions.
- Secondary indexes contain entries only for visible rows.
- `SELECT * FROM users;` on primary-key tables prints visible rows in primary-key ascending order.
- `SELECT * FROM users WHERE id = <int>;` returns visible row or header only.
- Secondary equality/range positions must be filtered by committed visible entries, not by table scan.

## Persisted Mutation Plan
The implementation should add one SQL logical record per successful mutation. Recommended shape:

```text
PDBSQL1\0
U
u16 table_name_len little-endian
table_name UTF-8 bytes
u16 primary_key_column little-endian
i64 primary_key little-endian
u16 set_column little-endian
encoded replacement row values
encoded removed secondary entries
encoded added secondary entries
```

```text
PDBSQL1\0
D
u16 table_name_len little-endian
table_name UTF-8 bytes
u16 primary_key_column little-endian
i64 primary_key little-endian
encoded removed secondary entries
```

Implementation may refine the exact layout, but final docs and fixture builders must match the implemented bytes exactly. Each mutation record must include enough row identity and secondary-index delta data for `db check` to detect:
- stale old-key secondary entries after update;
- dangling/deleted row pointers after delete;
- missing entry for a visible indexed row.

No production implementation should encode a mutation as separate row plus independent index records, because that would create partial statement states outside the contract.

## Execution Flow
`UPDATE users SET age = 30 WHERE id = 2;`:
1. Parse only one-column assignment with primary-key equality predicate.
2. Validate table, set column, primary key column, value type, and target row existence.
3. Build a full replacement row preserving unchanged columns.
4. Validate primary-key uniqueness if the assignment changes the primary key in a future allowed case; for this contract, age changes only.
5. Compute secondary removal entries for the old visible row and secondary addition entries for the replacement visible row.
6. Append one durable update record.
7. Apply runtime row replacement at the same row position.
8. Update primary and secondary indexes only after append succeeds.

`DELETE FROM users WHERE id = 3;`:
1. Parse only primary-key equality deletion.
2. Validate table, primary-key column, and target row existence.
3. Compute secondary removal entries for the visible target row.
4. Append one durable delete record.
5. Mark the row position deleted/invisible.
6. Remove primary and secondary index entries only after append succeeds.

## Acceptance Evidence Mapping
| Contract Item | Planned Evidence |
|---|---|
| fixed users fixture | helper in `tests/secondary_index.rs` creates exact schema and seed rows |
| update exits 0 with empty stdout/stderr | black-box `db exec` assertion |
| old key equality after update | separate process `SELECT * FROM users WHERE age = 20;` exact stdout |
| new key equality after update | separate process `SELECT * FROM users WHERE age = 30;` exact stdout and ordering |
| range after update | separate process exact stdout, secondary key then primary key order |
| primary-key lookup after update | separate process exact stdout |
| table scan after update | separate process exact stdout |
| delete exits 0 with empty stdout/stderr | black-box `db exec` assertion |
| equality/range/primary/table scan after delete | separate process exact stdout assertions |
| restart/reopen | setup, update, delete, query, `db check` run as separate compiled `db` process invocations |
| WAL replay | page file and `<path>.wal` sidecar retained; reopen query and `db check` run in separate process |
| stale secondary entry fixture | deterministic builder creates old key `20` entry after visible `id=2 age=30`; `db check` exits 1 with exact stderr |
| dangling row pointer fixture | deterministic builder points committed entry at deleted/nonexistent row; `db check` exits 1 with exact stderr |
| missing indexed visible row fixture | deterministic builder omits entry for visible `id=4 age=30`; `db check` exits 1 with exact stderr |
| storage compatibility | final report states lower-level page/WAL format unchanged or documents SQL record changes; tests reopen existing secondary-index files |
| requirement ids | final report maps `REQ-7-insert-update-and-delete-must-997871f9` and `EVID-7-validate-index-invariants-for-uniqueness-2d153f8e` |

## Verification Strategy
Implementation phase must run:
- `./scripts/verify`
- `cargo test --test secondary_index -- --nocapture`

If negative fixture coverage is placed in `tests/db_check.rs`, also run:
- `cargo test --test db_check -- --nocapture`

The implementation report must include command, exit status, stdout/stderr summary, relevant test names, and requirement/evidence id mapping.

## Stop Conditions
- A required behavior cannot be implemented without changing `spec.md` or `contracts.md`.
- The implementation would need a broader SQL mutation semantics decision than primary-key targeted single-row mutation.
- Lower-level page/WAL framing must change to satisfy the contract.
- Verifier rejection requires a second recovery attempt.

