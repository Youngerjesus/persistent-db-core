# Development State: v1-sql-parser-schema-exec

## Current Status

Implementation retry 6 is verifier-ready.

## Completed Work

- Reconfirmed HEAD `8aea6208d2a42d51a78306ccd57dbbc5e7aad6a4`, dirty state, QA mapping, source, tests, and docs before implementation edits.
- Added `src/sql.rs` with parser, schema catalog rebuild, append-order executor, SQL logical record encode/decode, and typed error mapping.
- Wired `db exec <path> <sql>` through `src/main.rs` and exported `sql` from `src/lib.rs`.
- Updated durable docs for CLI behavior, SQL subset, semantic failure matrix, and `PDBSQL1\0` logical records.
- Marked task list entries complete from the implementation side.
- Retry repair for `IBR-001`: accepted noncanonical signed decimal `INT` literal spellings such as `-0` by replacing canonical-spelling validation with optional-sign decimal lexical validation plus `i64` range parsing.
- Retry repair for `IBR-002` verifier risk: clarified SQL row-value payload documentation so `INT` values are documented as canonical decimal UTF-8 bytes and `TEXT` values as literal text bytes.
- Retry repair for `IBR-003`: added SQL-prefixed invalid logical record fixture tests for duplicate catalog columns and output-breaking `TEXT` rows, then hardened `Database::from_records` to reject catalog identifiers, duplicate columns, empty schemas, and output-unsafe loaded text values with the documented invalid SQL storage record error.
- Retry repair for `IBR-004`: added a malformed `SELECT * users;` regression and refined `SELECT` parse classification so broken supported-shape `SELECT * FROM <table_name>;` attempts return malformed SQL while projection/filter variants remain unsupported.
- Retry repair for re-opened `IBR-004`: added `SELECT * FROM users extra;` malformed regression and `SELECT * FROM users WHERE id = 1;` unsupported guard coverage, then refined trailing-token classification so arbitrary trailing tokens after a valid `SELECT * FROM <identifier>` prefix are malformed while documented out-of-scope clause starters remain unsupported.
- Retry repair for `IBR-006`: added a SQL-prefixed fixture for noncanonical persisted `INT` row bytes `01` and hardened row decode to reject `INT` payload bytes unless they match `parsed.to_string()`.
- Retry repair for `IMPL-VERIFY-001`: updated `docs/cli_contract.md` to document duplicate table and duplicate column case-variant target spelling (`Users`, `ID`), matching `docs/sql_subset.md` and the contract-required semantic failure matrix parity.

## Verification Evidence

- Red reproduction before retry fix: `cargo test --test sql_exec signed_decimal_int_literals_accept_noncanonical_zero_spelling` failed because `INSERT INTO nums VALUES (-0);` returned malformed SQL.
- Red reproduction before retry 2 fix: `cargo test --test sql_exec sql_prefixed_` failed because invalid SQL-prefixed fixture records returned successful `SELECT` output.
- Red reproduction before retry 3 fix: `cargo test --test sql_exec malformed_select_shape_reports_exact_statement` failed because `SELECT * users;` returned unsupported SQL.
- Red reproduction before retry 4 fix: `cargo test --test sql_exec malformed_select_trailing_token_reports_exact_statement` failed because `SELECT * FROM users extra;` returned unsupported SQL.
- Red reproduction before retry 5 fix: `cargo test --test sql_exec sql_prefixed_noncanonical_int_row_record_fails_deterministically` failed because persisted `INT` bytes `01` returned successful output.
- `cargo test --test sql_exec`: pass, 18 tests.
- `cargo test --test cli_contract`: pass, 5 tests.
- `cargo fmt --check`: pass.
- `cargo clippy --all-targets -- -D warnings`: pass.
- `cargo test`: pass.
- `./scripts/verify`: pass.
- Required smoke command: pass, stdout `id|name\n1|ada\n2|bea\n`, stderr empty, exit `0`.

## Notes

- `PageStore` page framing is unchanged. SQL records are stored as opaque payloads above `PageStore`.
- The SQL CLI path initializes a zero-byte target file before opening it so the contract smoke command using `mktemp` succeeds; nonempty invalid files still flow through storage validation.
- `impl_review.md` was absent during retries. `impl_brake_review.md` was read and left unchanged as reviewer-owned SSOT.
