# Research

## Question
What is the minimum implementation evidence delta needed to refresh `gate-v1-indexes` for `REQ-7-implement-integer-primary-key-as-9c698e08` at the current managed repo SHA without expanding primary-index scope?

## Findings
- `src/index.rs` already defines `PrimaryIndex` over `std::collections::BTreeMap<i64, usize>` with duplicate-aware `insert`, exact `get`, `remove`, `ordered_positions`, `len`, and `is_empty`.
- `src/sql.rs` already has primary-key catalog metadata, rebuild-on-open paths, duplicate validation, exact primary-key lookup, and primary-key ordered scan paths. Implementation must revalidate current behavior rather than assume the observed context is sufficient.
- `tests/primary_index.rs` already contains primitive tests, ordered-position tests, reopen/rebuild SQL behavior, persisted duplicate fixture helpers, and row-only catalog compatibility coverage.
- `tests/sql_exec.rs` already contains broad SQL CLI behavior tests and `primary_key` filtered tests from the earlier primary-index slice. The current contract requires exact scenario-level evidence, including a combined SQL input, restart/reopen on the same database path, duplicate insert preservation, and a valid persisted duplicate-row fixture.
- `docs/v1_acceptance.md` currently maps `gate-v1-indexes` to legacy `req-v1-primary-index-proof`, not the current artifact requirement id `REQ-7-implement-integer-primary-key-as-9c698e08`.
- `scripts/verify` is the required baseline command. `scripts/verify_primary_index_acceptance` is absent and should be added as a focused acceptance command if implementation proceeds.
- The existing `tests/primary_index.rs` constant for invalid SQL storage stderr currently shows the generic `unknown record tag` text in the excerpt. The implementation phase must verify whether persisted duplicate primary-key reopen now returns the contract-required duplicate-primary-key invalid-storage message and repair tests/source only if needed.

## Decisions
| Decision | Rationale |
|---|---|
| Treat this as current-artifact evidence refresh, not a new index feature. | The approved spec says earlier primary-index implementation and tests exist; the blocker is current artifact digest/current-SHA traceability. |
| Keep `artifact_requirement_ids` limited to `REQ-7-implement-integer-primary-key-as-9c698e08`. | The contract explicitly forbids claiming `REQ-7-create-index-must-create-disk-3b71a7dc`, `REQ-7-insert-update-and-delete-must-997871f9`, or `EVID-7-validate-index-invariants-for-uniqueness-2d153f8e`. |
| Use focused black-box integration tests for CLI stdout/stderr/exit-code evidence. | The acceptance criteria specify exact CLI observable behavior and same-path reopen behavior. |
| Use direct fixture records only for the persisted duplicate-primary-key invariant. | The contract requires valid SQL catalog and row records with duplicate primary key `2`; malformed tags or broken prefixes cannot substitute. |
| Add a narrow `scripts/verify_primary_index_acceptance`. | It gives the scheduler a task-specific command and matches the intended touch list without replacing baseline `scripts/verify`. |
| Update `docs/v1_acceptance.md` only for traceability after command evidence exists. | The durable acceptance guide should cite the current SHA, requirement id, final evidence path, and commands, while avoiding broader gate completion claims. |

## Rejected Options
| Option | Reason |
|---|---|
| Edit `spec.md` or `contracts.md`. | The approved package is frozen by the SDD autopilot workflow. |
| Change `ssot/` or `policies/`. | Protected by the contract and not required for this evidence refresh. |
| Broaden into SQL schema acceptance or secondary index requirements. | The spec explicitly selected only the primary-index current-artifact requirement and excluded conflicting SQL acceptance candidates. |
| Use generic `scripts/verify` output alone as final evidence. | The contract requires scenario mapping, exact commands, current SHA, and final review evidence in the feature evidence path. |
| Replace valid persisted duplicate-row evidence with corrupt-record fixtures. | The contract explicitly rejects malformed tag/prefix/length substitutes. |

## Open Implementation Risks
- Current tests may pass older primary-index behavior while missing the exact combined SQL input, exact stderr, or valid duplicate-row reopen fixture required by this package.
- The persisted duplicate fixture may currently report a generic invalid-storage error; implementation must make the failure label/message match the contract if the focused test fails.
- `docs/v1_acceptance.md` must not imply unrelated index requirements are complete.
