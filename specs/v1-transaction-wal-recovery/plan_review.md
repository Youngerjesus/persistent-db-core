# Plan Verification Review: v1-transaction-wal-recovery

## Verdict
ready_for_reverification

## Original Verdict
retry

## Retry 1 Resolution
All `Must Fix Before Implementation` items below have been addressed in the planning artifacts. The review is ready for plan verification to rerun.

## Reason
The package has the right scope, artifacts, and acceptance mapping, but the implementation plan is not yet decision-complete for the core WAL contract. Implementation would still need to choose important recovery semantics before tests, code, and file-format documentation can be written consistently.

## Must Fix Before Implementation

- [x] Freeze the WAL frame layout in `design.md` or `plan.md` before implementation starts.
  - Current issue: `design.md` lists candidate fields and says the final byte layout will be documented later, but the deterministic Scenario B fixture and `docs/file_format.md` contract depend on the exact framing.
  - Required detail: specify magic/version bytes, frame id or transaction id field width, state marker values, payload kind values, payload length encoding, payload bytes, validation/checksum strategy, and how incomplete trailing bytes are detected.

- [x] Choose one replay idempotence/checkpoint strategy instead of leaving alternatives.
  - Current issue: `design.md` lists durable applied markers, WAL truncation/checkpoint, or frame ids with persisted applied-state metadata as acceptable options, and `tasks.md` repeats this as an implementation choice.
  - Required detail: state the chosen strategy, when replay applies frames, when WAL is truncated or retained, how repeated opens avoid duplicate SQL records, and what file-state evidence the implementation report must capture.

- [x] Make the Scenario B fixture path executable from the chosen layout.
  - Current issue: `tasks.md` says fixture helpers are added only after layout finalization, so T1 cannot be implemented deterministically from the current plan.
  - Required detail: describe whether Scenario B will be CLI-backed or storage-level, which helper/API will write the fixture, and how the committed `1|ada` row is represented so replay proves committed WAL application while the incomplete/rollback `9|ghost` is excluded.

## Already Satisfactory

- Canonical `spec.md` and `contracts.md` are present and approved.
- The package correctly keeps public transaction SQL, multi-process concurrency, network behavior, and broad crash-matrix work out of scope.
- Required verification commands are consistently carried through: `cargo test`, `cargo test --test wal_recovery`, and `./scripts/verify`.
- `docs/cli_contract.md` is correctly conditional on public CLI behavior changes.
