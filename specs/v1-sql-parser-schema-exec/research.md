# Research: Minimal SQL Schema/Execute Path

## Decision 1: Keep SQL Support In A New Internal Module
- Decision: add `src/sql.rs` and expose it from `src/lib.rs`; keep `src/main.rs` responsible for argument routing and process exit mapping.
- Rationale: current repo has a small CLI entrypoint plus `PageStore`; a separate module keeps parser/catalog/executor behavior testable without expanding public CLI scope.
- Contract fit: supports the intended touches and keeps the CLI surface limited to `db exec <path> <sql>`.

## Decision 2: Implement A Small Hand-Written Parser
- Decision: use standard-library-only parsing with a tiny lexer/token stream for the three supported statement shapes.
- Rationale: the approved subset is intentionally narrow, dependencies should not broaden without task-level reason, and exact malformed-vs-unsupported errors must be deterministic.
- Parser classification rule:
  - recognized leading supported statement family with broken required shape: malformed SQL
  - unsupported statement family or supported family outside documented grammar, such as projection or `WHERE`: unsupported SQL
  - missing semicolon or invalid literal/identifier/type inside supported shape: malformed SQL

## Decision 3: Reconstruct In-Memory State From Append-Only SQL Records
- Decision: open `PageStore`, read all records, validate each SQL logical record tag, rebuild catalog and row vectors in append order, then execute statements sequentially.
- Rationale: the existing storage primitive is append/read opaque records; rebuilding state is deterministic and adequate for V1.
- Contract fit: restart persistence, row append order, and unknown pre-SQL payload rejection are all directly testable.

## Decision 4: Use A Versioned Text-Compatible Logical Record Encoding
- Decision: encode SQL logical records as UTF-8 bytes with `PDBSQL1\0` prefix and tab-separated fields. Use percent-style byte escaping for internal separators in metadata and values, while still rejecting user `TEXT` literals containing `|`, newline, carriage return, or single quote.
- Required record kinds:
  - catalog: `PDBSQL1\0catalog\t<table>\t<column>:<type>\t...`
  - row: `PDBSQL1\0row\t<table>\t<type>:<value>\t...`
- Rationale: human-inspectable enough for docs, version-tagged, and still opaque to the page layer.
- Contract fit: page headers and page record framing remain unchanged; SQL storage docs can describe the logical layer separately.

## Decision 5: Validate Before Appending Each Statement Record
- Decision: for `CREATE TABLE` and `INSERT`, perform all semantic validation before `append_record`. For `SELECT`, generate output only in memory and buffer stdout until the whole command succeeds.
- Rationale: prevents partial records for failed statements and enforces empty stdout on any failed command.
- Contract fit: no command-level atomicity is promised; successful prior statements remain durable when a later statement fails.

## Decision 6: Error Mapping Is A Stable Boundary
- Decision: map user SQL syntax/semantic errors to exit `2`, SQL storage logical-record errors to exit `1`, and underlying `StorageError` to exit `1` with deterministic CLI-level stderr.
- Rationale: contract fixes user-facing SQL strings and unknown SQL record string; page-level corruption must not be hidden.
- Contract fit: preserves panic-free failure behavior and separates user mistakes from invalid database files.

## Open Technical Unknowns
No product or contract blocker remains. Implementation must still inspect latest code before edits and may adjust private helper names, but not the grammar, output, persistence, or error contracts.

