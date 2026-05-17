# Fresh Repair Baseline

## Trigger Finding

Latest `impl_brake_review.md` records `Fresh Repair Required: yes` with open verify-blocking findings IB-WAL-001 and IB-WAL-002, plus verify-risk IB-WAL-003.

## Root Cause

- IB-WAL-001: The evidence validator checked report shape but did not compare report identity fields against live `git rev-parse HEAD`, live `git status --short`, or the recorded scheduler result path. The report also recorded an expanded untracked-file list while naming the plain `git status --short` command.
- IB-WAL-002: The prior proof combined uncommitted-change absence with incomplete trailing WAL bytes. It did not independently exercise the complete rolled-back WAL frame path that V1 documents and `src/storage.rs` implements.
- IB-WAL-003: The report summarized command output without preserving full transcript artifacts or transcript paths.

## Changed-File Classification

- `tests/wal_recovery.rs`: test coverage.
- `specs/v1-wal-recovery-current-sha-proof/verify_evidence_contract.sh`: helper validator.
- `specs/v1-wal-recovery-current-sha-proof/qa_mapping.md`: task-scoped QA mapping artifact.
- `specs/v1-wal-recovery-current-sha-proof/final_report.md`: generated evidence report, to be regenerated for this retry.
- `specs/v1-wal-recovery-current-sha-proof/evidence/`: generated command transcripts for this retry.
- current retry `result.md`: generated scheduler outcome.

## Stale/Generated Evidence Cleanup

The prior `final_report.md` is stale for this retry because it records the previous implementation run id and a non-live dirty-state transcript. It will be replaced with fresh retry evidence. Command transcripts will be generated under `specs/v1-wal-recovery-current-sha-proof/evidence/impl_retry_0_resume_20260518_013345_481129_3fa984a5/`.

## Temporary Or Test-Only Workaround Cleanup

No product-model workaround is planned. The repair adds black-box WAL recovery coverage for an already documented complete rolled-back frame state rather than weakening tests or changing production behavior to satisfy evidence.

## Legitimate Helper Or Harness Components Retained Or Promoted

The existing direct WAL fixture harness in `tests/wal_recovery.rs` is retained because public CLI has no rollback or incomplete transaction command. The validator is promoted from shape-only checks to live provenance checks.

## Regenerated Evidence Plan

Run fresh commands from the current worktree: `git rev-parse HEAD`, `git status --short`, `cargo test --test wal_recovery`, `./scripts/verify`, canonical CLI smoke commands, and `bash specs/v1-wal-recovery-current-sha-proof/verify_evidence_contract.sh`. Record command transcripts and WAL sidecar states in the retry final report.

## Remaining Verify Risk

The retry must ensure the final report records the live plain `git status --short` output and that validator expectations do not silently accept stale report data.

## Next Brake Approval Target

`impl_brake_exec` should be able to confirm IB-WAL-001 and IB-WAL-002 are closed and decide whether transcript paths close IB-WAL-003.
