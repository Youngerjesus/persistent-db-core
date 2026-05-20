# Design: Current-Artifact WAL Recovery Evidence Refresh

## Boundary
The design boundary is task-scoped evidence under `specs/v1-transaction-wal-current-artifact-evidence-refresh/`. Existing runtime behavior is verified through CLI and Rust integration tests. Production code changes are not expected unless a required current-SHA proof fails.

## Evidence Flow
1. Implementation records current repo identity.
2. Implementation executes the required command suite from the repo root.
3. `cargo test --test wal_recovery` and its focused cases prove committed replay, rollback/uncommitted exclusion, incomplete-tail exclusion, idempotence, and deterministic ahead-of-store failure.
4. `scripts/verify_crash_matrix` proves crash interruption cases and validates generated report shape.
5. A fresh `db exec` smoke proves a retained WAL sidecar is created and consumed across separate process starts.
6. Evidence artifacts map observations to exact requirement IDs.
7. `final_review.md` summarizes closure or records a blocker.

## Artifact Layout
```text
specs/v1-transaction-wal-current-artifact-evidence-refresh/
  spec.md
  contracts.md
  spec-progress.md
  readiness-preflight.md
  research.md
  plan.md
  design.md
  tasks.md
  analysis_report.md
  readiness.md
  evidence/
    current-repo-sha.txt
    command-log.md
    requirement-evidence.md
    wal-sidecar-smoke.md
    crash-matrix-log.md
  final_review.md
```

The `evidence/` directory and `final_review.md` are implementation-phase outputs, not planning-phase outputs.

## Requirement Mapping
| Requirement ID | Primary proof | Secondary proof |
|---|---|---|
| `REQ-8-begin-commit-and-rollback-provide-44e7901f` | `rolled_back_wal_frame_is_not_replayed_as_uncommitted_change` | Crash matrix `CM-003` if present in `crash-matrix-log.md`. |
| `REQ-8-committed-writes-survive-crash-and-35caf667` | `committed_wal_replay_survives_reopen_via_cli` | Fresh WAL sidecar/reopen smoke. |
| `REQ-9-provide-wal-or-equivalent-write-80297892` | `scripts/verify` plus WAL sidecar smoke | `docs/file_format.md` reference only if documentation remains current. |
| `REQ-9-recovery-must-be-idempotent-and-300531dc` | `incomplete_wal_entry_is_not_replayed_without_public_rollback_cli` and `committed_frame_after_incomplete_tail_cleanup_remains_replayable` | Crash matrix `CM-004` and `CM-005` if validated. |
| `REQ-9-checkpoint-or-log-truncation-must-d633d286` | `scripts/verify_crash_matrix` | `committed_wal_frame_ahead_of_page_store_fails_deterministically` and crash matrix `CM-001` through `CM-006`. |

## Data-Loss Risk Handling
The checkpoint/log-truncation requirement is the highest-risk row. Implementation must not infer this row from generic green tests. It must use a passing `scripts/verify_crash_matrix` run whose generated report demonstrates deterministic recovery or deterministic failure for the crash/interruption cases.

## Blocker Routing
If the crash matrix script is absent, fails, or does not directly cover checkpoint/log-truncation interruption safety, implementation writes a blocker in `final_review.md` and `requirement-evidence.md` and returns `blocking`. It must not weaken the requirement or mark the artifact gate complete.

## Non-Visual Evidence Design
This package has no rendered UI. Deterministic proof is command output, WAL/file state, crash matrix report content, and current SHA. Visual and UX proof layers are explicitly not applicable.
