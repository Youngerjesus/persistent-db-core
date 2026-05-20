## 2026-05-20T15:41:06Z - Archived Previous Latest Code Review

# Code Review

Verdict: FAIL

## Scope

Reviewed the current branch and worktree for `task-2026-05-20-23-32-28-v1-transaction-wal-current-artifact-evidence-refresh` against `main`.

- `git log --oneline main..HEAD`: no commits.
- `git diff --stat main...HEAD`: no tracked diff.
- `git diff --stat`: no tracked or staged diff.
- `git status --short`: the task package is untracked as `?? specs/v1-transaction-wal-current-artifact-evidence-refresh/`.
- Review surface: the untracked current-artifact package under `specs/v1-transaction-wal-current-artifact-evidence-refresh/`, including `qa_mapping.md`, `verify_evidence_contract.sh`, `evidence/*`, `final_review.md`, and phase review artifacts.
- QA context: `qa_mapping.md` defines task-scoped green as current SHA identity, command evidence, exact requirement-ID mapping, WAL sidecar/reopen proof, crash matrix proof, and non-visual evidence only.

No production Rust code, durable docs, or tracked test/script files changed against `main`.

## Specialist Routing

| Reviewer | Trigger | Status | Evidence Source | Accepted Finding IDs | Rejected Finding IDs | Skip/Fallback Reason |
| --- | --- | --- | --- | --- | --- | --- |
| `code-reviewer` | Correctness, completeness, spec mismatch, merge safety | invoked | Agent `019e45f7-e009-7d41-a7e5-4ad979cdf5e2`: reported untracked deliverable and stale QA-prep SSOT; reran `verify_evidence_contract.sh`, `scripts/verify_crash_matrix`, `cargo test --test wal_recovery`, and `scripts/verify` successfully. | `CR-001`, `CR-002` | none | n/a |
| `testing-reviewer` | Coverage, edge cases, validator strength, command evidence quality | invoked | Agent `019e45f7-e06a-7a70-9f73-ccd46c30eeaf`: reported build-provenance gap in canonical smoke evidence and weak provenance enforcement in validator. | `TR-001`, `TR-002` | none | n/a |
| `security-reviewer` | File/process boundaries, command transcripts, local path and sensitive data exposure | invoked | Agent `019e45f7-e0b0-7150-a9f5-e7bfb3b24c7c`: reported absolute local path leakage, validator bypassability, and direct `target/debug/db` smoke provenance risk. | `SEC-001`; `SEC-002` merged into `TR-002`; `SEC-003` merged into `TR-001` | none | n/a |
| `performance-reviewer` | Verification scripts and repeated heavyweight command evidence | invoked | Agent `019e45f7-e14b-7101-a22c-3f55207e067f`: no open performance findings; noted duplicated operator/runtime cost as residual only because the package does not wire the commands into automation. | none | none | n/a |
| `maintainability-reviewer` | Brittle structure, misleading evidence, stale report risk | fallback-applied | Runtime role unavailable; self-applied lens during report reconciliation. | `CR-002`, `TR-002` | none | Dedicated companion unavailable in this runtime. |
| `red-team-reviewer` | Additive bias, proxy-success evidence, stale/local-only artifact risk | fallback-applied | Runtime role unavailable; self-applied lens during report reconciliation. | `CR-001`, `TR-001`, `TR-002` | none | Dedicated companion unavailable in this runtime. |
| `database-reviewer` | Persistence boundary and durable state evidence | fallback-applied | Runtime role unavailable; self-applied lens against `tests/wal_recovery.rs`, `tests/crash_matrix.rs`, and current evidence package. | `TR-001` | none | Dedicated companion unavailable in this runtime; no product DB code diff was present. |
| `api-reviewer` | Endpoint/DTO/schema/transport contract changes | skipped | Self-applied routing check: no API surface or transport diff. | none | none | No API trigger. |
| `ui-ux-reviewer` | UI, layout, interaction, accessibility, visual evidence | skipped | Self-applied routing check: spec and contract mark visual/UX evidence not applicable. | none | none | No UI trigger. |

## Findings

### CR-001 - Reviewed Deliverable Is Untracked

Severity: High

Status: Must Fix Now

Evidence:
- `git status --short` reports `?? specs/v1-transaction-wal-current-artifact-evidence-refresh/`.
- `git log --oneline main..HEAD` and `git diff --stat main...HEAD` are empty.
- `git ls-files specs/v1-transaction-wal-current-artifact-evidence-refresh` returns zero tracked files.
- `evidence/current-repo-sha.txt` records the same untracked package state at line 8.

Risk:
The task output would be dropped by any branch merge because the deliverable is not part of the branch. That directly blocks the current-artifact requirement rows from being available to the artifact matcher.

Required fix:
Add the intended task package files to the branch in the retry pass, then re-run the code review scope checks against the resulting tracked delta.

### TR-001 - Canonical WAL Smoke Is Not Build-Coupled

Severity: High

Status: Must Fix Now

Evidence:
- `evidence/wal-sidecar-smoke.md:11` and `evidence/wal-sidecar-smoke.md:22` record `target/debug/db exec ...` as the executed product commands.
- `evidence/requirement-evidence.md:16` and `evidence/requirement-evidence.md:17` map those direct binary commands into the requirement evidence.
- `final_review.md:31` and `final_review.md:32` rely on `wal-sidecar-smoke.md` for `REQ-8-committed-writes-survive-crash-and-35caf667` and `REQ-9-provide-wal-or-equivalent-write-80297892`.
- `impl_review.md:30` and `impl_review.md:31` later record a fresh `cargo run --quiet --bin db -- exec ...` smoke, but that build-coupled provenance is not reflected in the canonical smoke artifact or final review.

Risk:
The package can claim current-SHA product proof while the canonical smoke evidence may have exercised a stale `target/debug/db` binary. For an evidence-only task, the canonical artifact should carry the build-coupled invocation or explicit rebuild provenance.

Required fix:
Regenerate or amend the canonical WAL sidecar smoke evidence and requirement/final review references so the recorded product smoke is build-coupled, for example via `cargo run --bin db -- exec ...` or an explicit rebuild step tied to the recorded SHA.

### TR-002 - Provenance Validator Is Still Too Easy To Satisfy With Stale Text

Severity: Medium

Status: Must Fix Now

Evidence:
- `verify_evidence_contract.sh:31` through `verify_evidence_contract.sh:38` accept any `exit_code: 0` found within the next 12 lines after a matching command string.
- `verify_evidence_contract.sh:67` through `verify_evidence_contract.sh:76` only require path substrings in `current-repo-sha.txt`; they do not bind each required path to a corresponding `test -f` or `test -x` command with `exit_code: 0`.
- `qa_mapping.md` requires current SHA identity, dirty state, and required-file presence as task-scoped green for `WAL-CUR-002` and `T1`.

Risk:
A hand-written or stale provenance file can satisfy the validator more easily than the QA mapping implies. Since this package is evidence-only, the validator is a meaningful merge-quality surface, not just incidental test code.

Required fix:
Tighten `verify_evidence_contract.sh` so provenance and command evidence are structurally bound to their exact command rows, including required-file probe exit codes. Alternatively, add explicit negative validator fixtures proving stale or malformed provenance is rejected.

### SEC-001 - Machine-Local Absolute Path Leaks Into Package Artifact

Severity: Medium

Status: Must Fix Now

Evidence:
- `review_loop/code_context.md:4` records a machine-local repo root path; it should be redacted before commit.
- Repo policy says not to copy machine-specific paths into repo artifacts.

Risk:
If the untracked package is added as-is, it will commit a local username and workspace layout. This is not a secret, but it violates repo hygiene and makes the artifact less portable.

Required fix:
Redact or relativize the `repo_root` value before the package is added to the branch.

### CR-002 - Latest QA-Prep Review SSOT Is Stale

Severity: Medium

Status: Must Fix Now

Evidence:
- `qa_prep_review.md:3` still says `Verdict: RETRY`.
- `qa_prep_review.md:12` through `qa_prep_review.md:21` list scaffold gaps that the current validator partly addresses at `verify_evidence_contract.sh:67`, `verify_evidence_contract.sh:78`, `verify_evidence_contract.sh:91`, and `verify_evidence_contract.sh:125`.
- There is no `qa_prep_review.history.md` preserving the stale review separately.

Risk:
Repo instructions treat latest review/report files as verifier or reviewer SSOT. Leaving an obsolete latest QA-prep review with open retry findings contradicts the current implementation package and creates ambiguous handoff state.

Required fix:
The owning QA-prep/review phase should archive the stale content to `qa_prep_review.history.md` and refresh `qa_prep_review.md`, or the retry pass should remove the contradiction by routing this artifact back to the owning phase.

## Must Fix Now

- `CR-001`: Track the task package so the deliverable is actually part of the branch.
- `TR-001`: Make the canonical WAL sidecar smoke evidence build-coupled or record explicit rebuild provenance in the canonical artifact path.
- `TR-002`: Strengthen provenance validation for command rows and required-file checks, or add negative validator evidence.
- `SEC-001`: Remove the machine-local absolute path from `review_loop/code_context.md`.
- `CR-002`: Resolve the stale `qa_prep_review.md` SSOT contradiction through the owning review/report workflow.

## Residual Risks

- The package repeats heavyweight verification commands. This is operator/runtime cost, not a CI regression, because the new package does not wire those commands into automation.
- No product-code regression was found because there is no product-code diff in this branch; the review risk is artifact mergeability and evidence provenance.
- The dedicated maintainability, red-team, and database companion roles were unavailable, so those lenses were self-applied and reflected in accepted findings above.

## Next Action

Route to `code_review_retry`. Do not mark the artifact gate complete until the untracked deliverable and evidence-provenance findings are fixed and the latest code review is refreshed.

## Updated At

2026-05-20T15:19:50Z

## 2026-05-20T15:43:54Z - Archived Code Review Verification Failure

# Code Review

Verdict: FAIL

## Scope

Verified the current branch and worktree for `task-2026-05-20-23-32-28-v1-transaction-wal-current-artifact-evidence-refresh` against `main` during Code Review Verification round 2.

- `git log --oneline main..HEAD`: no commits.
- `git diff --stat main...HEAD`: no committed diff.
- `git diff --cached --stat`: 28 staged files under `specs/v1-transaction-wal-current-artifact-evidence-refresh/`.
- `git diff --stat`: no unstaged tracked diff.
- `git status --short`: the task package files are staged as `A`, including `code_review.md`, `qa_prep_review.md`, `qa_prep_review.history.md`, `verify_evidence_contract.sh`, `evidence/*`, and review-loop artifacts.
- Product code/durable docs delta: none against `main`; verification surface is the staged current-artifact evidence package and latest review/report consistency.

The previously reported evidence defects appear repaired in the staged artifacts, but the latest code review report presented to this verification phase still contained stale `FAIL` findings and `Next Action` text that no longer matched the staged package state.

## Specialist Routing

| Reviewer | Trigger | Status | Evidence Source | Accepted Finding IDs | Rejected Finding IDs | Skip/Fallback Reason |
| --- | --- | --- | --- | --- | --- | --- |
| `code-reviewer` | Correctness, completeness, spec mismatch, merge safety | evidence-verified | Archived latest report in `code_review.history.md`; current `git status --short`, `git diff --cached --stat`, and staged artifact checks. | `CRV-001` | none | No new subagent review was started in this verify phase per instruction. |
| `testing-reviewer` | Coverage, validator strength, command evidence quality | evidence-verified | `bash verify_evidence_contract.sh`, `cargo test --test wal_recovery`, `scripts/verify_crash_matrix`, and `scripts/verify` all exited 0. | `CRV-001` | none | No new subagent review was started in this verify phase per instruction. |
| `security-reviewer` | Local path leakage, sensitive/control-plane identity exposure, command provenance | evidence-verified | Implementation evidence no longer contains machine-local paths or direct binary smoke commands. | none | none | No new subagent review was started in this verify phase per instruction. |
| `performance-reviewer` | Added automation cost or heavyweight commands wired into baseline | evidence-verified | Staged package only adds evidence artifacts and a task-local validator; no product baseline script change. | none | none | No new subagent review was started in this verify phase per instruction. |
| `maintainability-reviewer` | Stale SSOT/report drift risk | fallback-applied | Self-applied verify lens against latest report consistency and staged artifact state. | `CRV-001` | none | Dedicated companion unavailable/not invoked in this verify phase. |
| `red-team-reviewer` | Proxy-success or stale-artifact risk | fallback-applied | Self-applied verify lens; validator and smoke provenance repairs passed, but latest report drift remains. | `CRV-001` | none | Dedicated companion unavailable/not invoked in this verify phase. |
| `database-reviewer` | Persistence boundary and WAL recovery evidence | fallback-applied | `cargo test --test wal_recovery` and `scripts/verify_crash_matrix` passed; no product DB code diff. | none | none | Dedicated companion unavailable/not invoked in this verify phase. |
| `api-reviewer` | Endpoint/DTO/schema/transport contract changes | skipped | No API surface or transport diff. | none | none | No API trigger. |
| `ui-ux-reviewer` | UI, layout, interaction, accessibility, visual evidence | skipped | Rust CLI evidence-only task; spec/contract mark visual and UX evidence not applicable. | none | none | No UI trigger. |

## Findings

### CRV-001 - Latest Code Review Report Was Not Refreshed After Repairs

Severity: Medium

Status: Must Fix Now

Evidence:
- The latest `code_review.md` presented to this verification phase still had `Verdict: FAIL`, stale `Must Fix Now` entries, and `Next Action` routing to `code_review_retry`.
- Current staged state contradicted that stale report: `git status --short` listed the task package as staged `A`, not untracked `??`, and `git diff --cached --stat` reported the package delta.
- Current canonical WAL smoke evidence recorded build-coupled product commands, and `verify_evidence_contract.sh` rejected direct binary smoke commands in smoke and requirement evidence.
- `review_loop/code_context.md` recorded `repo_root: <redacted-managed-repo-root>`.
- `qa_prep_review.md` recorded `Verdict: PASS`, and `qa_prep_review.history.md` preserved the stale QA-prep retry report.
- Verification evidence passed: `bash specs/v1-transaction-wal-current-artifact-evidence-refresh/verify_evidence_contract.sh`, `cargo test --test wal_recovery`, `scripts/verify_crash_matrix`, and `scripts/verify` all exited 0.

Risk:
The code review SSOT still routed the task as failed even though the staged artifact package appeared to have repaired the accepted findings. That report drift made the review trail unreliable and violated the phase requirement that latest `Must Fix Now` and `Next Action` match actual state.

Required fix:
Run the code-review retry/report-refresh path and replace the latest `code_review.md` with a current PASS report if the repair state remains as verified here. Preserve this verification failure in `code_review.history.md` before writing the refreshed latest report.

## Must Fix Now

- `CRV-001`: Refresh the latest code review SSOT so `Verdict`, `Must Fix Now`, and `Next Action` match the current staged artifact state.

## Residual Risks

- No product-code regression was found because there is no product-code diff in this branch.
- Python checks (`pytest`, `ruff`, `mypy`) are not applicable: no tracked Python files or Python tool configuration files were found by the applicability scan.
- The full baseline `scripts/verify` includes long-running benchmark acceptance paths; they passed in this verification run.

## Next Action

Route to `code_review_retry` for report refresh only. Do not mark the artifact gate complete while the latest code review SSOT still carries the stale failure trail.

## Updated At

2026-05-20T15:41:06Z
