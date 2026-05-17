# Research: v1-wal-recovery-current-sha-proof

## Goal
Close the stale-evidence gap for WAL recovery by proving the existing or repaired implementation against the current task worktree SHA, not against the prior manifest SHA `754958b37fd01f796b9d4f7522a2062b6e65abc5`.

## Decisions

### Evidence Strategy
Use an evidence-first implementation pass. The worker must first re-check current HEAD, dirty state, relevant files, and latest review/report context. If the current implementation already satisfies the contract, the artifact delta may be a task-scoped evidence transcript/report plus any missing narrow planning-to-evidence mapping. If a gap is found, repair only the minimum code, test, or documentation surface required by `contracts.md`.

Rationale: the approved package describes a stale proof problem, not a request to redesign WAL. Existing `tests/wal_recovery.rs`, `src/storage.rs`, `docs/file_format.md`, and `docs/cli_contract.md` already contain WAL recovery behavior at the observed HEAD, so the smallest valid closure path is current-SHA proof unless verification exposes a real defect.

### Required Proof Layers
Use these proof layers:
- deterministic test layer: `cargo test --test wal_recovery`;
- baseline verification layer: `./scripts/verify`;
- CLI smoke layer: exact create/insert and reopen/select process transcripts;
- file-state layer: `<db-path>.wal` existence and byte length after create/insert and after reopen/select;
- mapping layer: final transcript/report explicitly maps evidence to `gap-v1-transaction-wal-recovery`, `gate-v1-transactions-wal-recovery`, and `req-v1-wal-recovery-proof`.

Browser, DOM, screenshot, rendered-route, and UX design-review evidence are excluded by the contract and must not be substituted for these layers.

### Current WAL Semantics To Verify
Treat the currently documented WAL sidecar behavior as the contract to re-prove:
- sidecar path is `<database-path>.wal`;
- complete committed append frames are retained and idempotent by `record_count_before`;
- incomplete trailing headers or payloads are ignored, cleaned up, and not exposed as SQL rows;
- future complete frames remain replayable after incomplete-tail cleanup;
- a committed frame ahead of the page store fails deterministically.

Rationale: these are the behaviors currently represented in `tests/wal_recovery.rs` and `docs/file_format.md`, and they directly cover committed survival, uncommitted absence, and incomplete trailing WAL exclusion.

### Final Evidence Location
Implementation should create a task-scoped report or transcript under `specs/v1-wal-recovery-current-sha-proof/`, such as `evidence_transcript.md` or `final_report.md`.

The report must include:
- `git rev-parse HEAD`;
- `git status --short`;
- command lines for `cargo test --test wal_recovery`, `./scripts/verify`, and CLI smoke commands;
- exit code, stdout, and stderr for required commands or a precise transcript path;
- WAL sidecar existence and byte length after create/insert and after reopen/select;
- explanation that direct WAL fixture bytes represent the V1-observable uncommitted/incomplete state because there is no public rollback or incomplete transaction command.

### Repair Policy
If verification fails, repair only the failing acceptance item. Do not broaden CLI behavior, add public transaction commands, introduce network/background services, add dependencies, or change protected `ssot/` or `policies/` areas.

If the fix requires changing `spec.md`, `contracts.md`, protected areas, or public CLI stdout/stderr/exit code beyond the approved scope, stop and report a blocker.

## Risks
- A clean current-SHA verification may still leave no durable artifact delta unless a transcript/report is created.
- CLI smoke evidence must record exact empty stdout/stderr for mutation and exact select stdout, not just "passed".
- WAL sidecar length evidence must be captured at two points; running cleanup too early would lose required file-state evidence.
- If verifier rejects and a second recovery attempt is needed, the contract requires escalation.

