# QA Prep Verification Review History: v1-wal-recovery-current-sha-proof

## qa_prep_verify_1_fresh_20260518_011613_293816_72193f1c

# QA Prep Verification Review: v1-wal-recovery-current-sha-proof

Verdict: RETRY

## Scope

Reviewed `qa_mapping.md`, `tasks.md`, `spec.md`, `contracts.md`, `plan.md`, `design.md`, current WAL test/doc context, and the generated red scaffold `verify_evidence_contract.sh` for implementation-readiness.

## Findings

### Must Fix 1: Canonical latest QA review report was missing before verification

The phase prompt identified `specs/v1-wal-recovery-current-sha-proof/qa_prep_review.md` as a canonical phase artifact, but it did not exist at verification start. A completed QA-prep handoff should include the latest QA review/verdict context, not only `qa_mapping.md` and self-check prose inside that mapping.

Required repair:
- Produce and retain the canonical latest QA-prep review report as part of QA prep output.
- Keep current open QA-prep findings in `qa_prep_review.md`; move stale/resolved history to a history file only if this report is refreshed later.

### Must Fix 2: Evidence scaffold is too weak to close task-scoped green criteria

`qa_mapping.md` defines concrete task-scoped green criteria, but `verify_evidence_contract.sh` only checks broad keyword presence and one generic zero exit code. A superficial `final_report.md` could pass the scaffold while missing or weakening required acceptance evidence.

Required repair:
- Strengthen `verify_evidence_contract.sh` or add an equivalent task-scoped validator so it rejects a final report that lacks distinct command records for:
  - `git rev-parse HEAD` with exit `0`, stdout SHA, and stderr;
  - `git status --short` with exit `0`, stdout, and stderr;
  - `cargo test --test wal_recovery` with exit `0`;
  - `./scripts/verify` with exit `0`;
  - create/insert CLI smoke with exit `0`, stdout exactly empty, and stderr exactly empty;
  - reopen/select CLI smoke with exit `0`, stderr exactly empty, and stdout exactly `id|name\n1|ada\n2|bea\n`.
- Require two separate WAL sidecar state records: immediately after create/insert and immediately after reopen/select. Each record must include existence and byte length; if complete frames are retained, the validator or review checklist must require non-empty sidecar evidence.
- Require explicit evidence text for all four WAL recovery test names currently mapped in T2, including separate rationale for direct WAL fixture coverage of uncommitted/incomplete state.
- Require explicit mapping to `gap-v1-transaction-wal-recovery`, `gate-v1-transactions-wal-recovery`, and `req-v1-wal-recovery-proof`.

### Must Fix 3: Provenance wording hardcodes a QA-prep run id as current-run source

`qa_mapping.md` states `Current-run id source: scheduler metadata active_run_id=qa_prep_exec_fresh_20260518_011312_890598_7123d5c2`. The implementation evidence will be produced in a later implementation run, so hardcoding the QA-prep run id risks stale provenance in the final evidence contract.

Required repair:
- Reword the provenance contract so implementation evidence records the active implementation run id and scheduler result path observed at implementation time.
- Keep the QA-prep run id only as historical QA-prep provenance if needed, not as the current-run source for future implementation evidence.

## Passing Checks

- `qa_mapping.md` maps all `tasks.md` entries T1 through T6.
- Preferred commands are concrete and repo-root runnable.
- Scenario coverage is not happy-path only; it includes stale SHA, uncommitted/incomplete WAL bytes, duplicate replay/idempotence, ahead-of-store failure, WAL sidecar state capture, temp path trust boundary, and retry/re-entry artifact reuse.
- `verify_evidence_contract.sh` is red before implementation because `final_report.md` is absent, which is directionally correct for QA prep.
- Current `tests/wal_recovery.rs`, `docs/file_format.md`, and `docs/cli_contract.md` are not obviously contradictory with the spec, contract, plan, or design.

## Next Action

Return to QA prep. Repair the review artifact and scaffold/provenance issues above, then rerun QA-prep verification. Implementation should not start until the QA contract rejects incomplete evidence rather than merely checking broad report keywords.
