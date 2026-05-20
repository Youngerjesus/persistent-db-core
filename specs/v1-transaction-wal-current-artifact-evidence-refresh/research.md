# Research: Transaction WAL Recovery Current-Artifact Evidence Refresh

## Research Goal
Determine how to regenerate current-artifact evidence for `gate-v1-transactions-wal-recovery` without changing the approved WAL recovery contract or broadening product behavior.

## Current Repo Observations
- `scripts/verify` exists and runs the baseline checks from repo policy: `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`, and `cargo run --bin db -- --help`.
- `tests/wal_recovery.rs` exists and contains focused tests for:
  - `committed_wal_replay_survives_reopen_via_cli`
  - `rolled_back_wal_frame_is_not_replayed_as_uncommitted_change`
  - `incomplete_wal_entry_is_not_replayed_without_public_rollback_cli`
  - `committed_frame_after_incomplete_tail_cleanup_remains_replayable`
  - `committed_wal_frame_ahead_of_page_store_fails_deterministically`
- `scripts/verify_crash_matrix` exists and runs `cargo test --test crash_matrix`, then validates `target/crash_matrix/crash_matrix_report.md`.
- `tests/crash_matrix.rs` emits deterministic case reports for `CM-001` through `CM-006`, including interrupted recovery, partial WAL frame, rollback marker, repeated reopen, and corrupt-tail scenarios.
- Prior evidence packages exist, but the current task explicitly requires new current-artifact requirement-ID mapping for this artifact slug.

## Decisions
| Decision | Rationale | Contract impact |
|---|---|---|
| Treat this as evidence refresh first, not feature implementation. | Current tests and scripts already appear to cover the required behavior surface; the gap is current-artifact matcher evidence. | Matches scope and avoids unrelated code changes. |
| Capture command output into task-scoped evidence files during implementation. | Contract requires current SHA command output and artifact paths, not scheduler success. | Required for every candidate acceptance criterion. |
| Use `scripts/verify_crash_matrix` as the checkpoint/log-truncation proof source only if it passes and its report validates `CM-001` through `CM-006`. | Contract marks missing/failing/non-specific crash matrix evidence as a human-required blocker. | Prevents false completion for data-loss-risk evidence. |
| Keep visual/UX evidence explicitly not applicable. | Canonical spec and contract state this is a Rust CLI evidence task. | Avoids adding irrelevant screenshot or DOM proof layers. |
| Preserve public CLI and persisted-data behavior unless verification exposes a contract failure. | This phase should prove existing behavior, not redesign WAL. | Any behavior change would require focused tests and docs in implementation, but is not planned. |

## Unknowns To Resolve In Implementation
- Whether all required commands pass at the implementation phase's live HEAD.
- Whether generated crash matrix report still contains every validator-required literal and current run id.
- Whether a fresh WAL sidecar smoke can be captured with current SHA and exact stdout/stderr/exit evidence.

## Blocker Conditions
- `scripts/verify_crash_matrix` missing, failing, or no longer proving checkpoint/log-truncation interruption safety.
- Required WAL recovery test name missing or failing.
- Fresh evidence cannot be tied to current repo SHA.
- Evidence artifacts omit requirement IDs, commands, expected behavior, or artifact refs.
