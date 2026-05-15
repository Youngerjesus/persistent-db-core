# V1 `db` CLI 계약 및 smoke 테스트 기반 확정

**Status**: APPROVED

## 메타데이터
- Run ID: 2026-05-15-16-06-54
- Task ID: task-2026-05-15-16-06-54-v1-bootstrap-cli-contract
- Candidate rank: 1
- Target boundary: managed_repo
- Objective: V1 `db` CLI 계약 및 smoke 테스트 기반 확정
- Artifact: v1-bootstrap-cli-contract

## 목표
- V1 DB의 storage, SQL, index, WAL 작업은 모두 실행 가능한 `db` binary 계약에 의존하지만, 현재 repo에는 CLI skeleton만 있고 help 출력, command dispatch, unsupported argument behavior, smoke test evidence가 아직 formalized되어 있지 않습니다.

## 지금 해야 하는 이유
- Current Objective가 CLI contract를 첫 dependency로 지정하고, Current Plan의 high-priority next candidate가 `gap-v1-bootstrap-cli-contract`이며, queue와 evidence cache 모두 중복 또는 진행 중 작업을 보고하지 않습니다.

## 기대 산출물 변화
- `db --help`의 안정적인 command surface, deterministic exit behavior, command dispatch tests, CLI contract docs, `cargo test` 및 `cargo run --bin db -- --help` 검증 기반이 managed repo에 생깁니다.

## 의도한 변경 대상
- src/main.rs
- Cargo.toml
- tests/cli_contract.rs
- docs/cli_contract.md
- route:db-help
- flow:cli-command-dispatch

## 관찰된 코드 맥락
- 이 섹션은 관찰 근거이며 구현 지시가 아닙니다. 실제 구현 전 worker는 최신 worktree에서 재검증해야 합니다.
- 관찰 기준 HEAD: f3fa75a95ba099d7145ab01175713b56664a25bb
- Dirty state: none
- Raw evidence: review_loop/code_context.md
- 관련 파일 후보: src/main.rs, Cargo.toml, docs/v1_spec.md, docs/history_archives/history.md, work_queue/progress.md, AGENTS.md, .codex/skills/spec-creator/SKILL.md, .codex/skills/spec-reviewer/SKILL.md

## Risk flags

## Daily Metric Loop
- Source: daily
- Final disposition: ready_for_handoff
- objective_plan_gap_fit: score=3
- causal_evidence_strength: score=3
- handoff_verifiability: score=3
- Constraint blockers: none

## 근거
- `ssot/current-objective.md`는 `metric-v1-cli-contract`를 첫 active success metric으로 두고, 모든 later gap이 runnable CLI contract에 의존한다고 명시합니다.
- `ssot/current-plan.md`는 `gap-v1-bootstrap-cli-contract`를 high priority로 두며 next candidate hint를 help output, command dispatch skeleton, smoke tests로 제시합니다.
- `ssot/current-artifact.md`의 `gate-v1-cli-smoke`는 `cargo test`, `cargo run --bin db -- --help`, command dispatch tests, CLI contract docs를 required evidence로 요구합니다.
- Root Progress Projection은 artifact_status가 `open`이고 `gate-v1-cli-smoke`에서 `req-v1-cli-help-smoke`, `req-v1-cli-dispatch-tests`가 open이라고 보고합니다.
- Active Managed Repo Snapshot은 repo가 clean이고 `Cargo.toml`, `Cargo.lock`, `src/main.rs`를 가진 Rust CLI skeleton 상태라고 보고합니다.
- Queue Snapshot은 `[]`이며 Gap Evidence Cache도 verified, queued, active task가 없다고 보고하므로 duplicate 또는 reserved work가 없습니다.
- AGENTS.md
- Cargo.toml
- src/main.rs
- docs/v1_spec.md
- docs/history_archives/history.md
- work_queue/progress.md
- project_manager/tasks/tasks.json
- project_manager/specs/v1-bootstrap-cli-contract/review_loop/design.md
- HELP
- main
- [[bin]] name = "db"
- db --help
- db help
- unsupported arguments
- project_manager/tasks/tasks.json: tasks=[]
- canonical project_manager/specs/v1-bootstrap-cli-contract/spec.md 없음
- canonical project_manager/specs/v1-bootstrap-cli-contract/contracts.md 없음

## 범위
- In scope: selected candidate only.
- Out of scope: unrelated breadth features.

## 수용 기준
- 선택된 candidate에 대한 구체적인 artifact delta가 존재해야 합니다.
- Daily metric loop evidence가 spec package와 일관되어야 합니다.
- 최종 리포트에 verification evidence가 연결되어야 합니다.

## CLI observable contract
- `db --help`와 `db help`는 동일한 help contract를 노출해야 합니다.
- 두 help command는 exit code `0`, 빈 stderr, 그리고 아래 핵심 stdout 행을 순서대로 포함해야 합니다. 구현은 공백 정렬을 추가할 수 있지만 행의 문구와 순서는 바꿀 수 없습니다.

```text
db - deterministic single-process V1 database CLI
Usage:
  db --help
  db help
Supported commands:
  help        Print this help text.
Reserved future commands:
  open <path>
  exec <path> <sql>
  check <path>
  bench <path>
V1 bootstrap scope:
  This build only defines the CLI contract and smoke baseline.
  Storage pages, SQL execution, indexes, transactions, WAL, and recovery are not implemented in this slice.
Non-goals:
  No network server, multi-process concurrency, or distributed storage.
```

- `open`, `exec`, `check`, `bench`는 future command로만 예약됩니다. 이번 slice에서 실행 가능한 command로 구현하거나 storage, SQL, index, transaction, WAL, recovery 동작을 시작하면 안 됩니다.
- 지원하지 않는 argument 또는 subcommand는 exit code `2`, 빈 stdout, 그리고 아래 형식의 stderr를 반환해야 합니다.

```text
error: unsupported argument or command: <token>
hint: run 'db --help' for the supported V1 CLI contract.
```

- `<token>`은 사용자가 전달한 첫 번째 unsupported token입니다. 예: `db --unknown`은 `<token>`이 `--unknown`이고, `db open demo.db`는 `<token>`이 `open`입니다.

## Candidate Acceptance Criteria
- `cargo test`가 성공하고 help 및 unsupported argument dispatch를 검증하는 자동 테스트가 포함됩니다.
- `cargo run --bin db -- --help`가 exit code `0`, 빈 stderr, 문서화된 `CLI observable contract`의 핵심 stdout 행과 일치하는 output을 반환합니다.
- `cargo run --bin db -- help`가 `db --help`와 동일한 help contract를 exit code `0`으로 반환합니다.
- `cargo run --bin db -- --unknown`과 `cargo run --bin db -- open demo.db`는 exit code `2`, 빈 stdout, 문서화된 unsupported stderr 형식을 반환하고 `tests/cli_contract.rs`로 검증됩니다.
- `docs/cli_contract.md`는 현재 지원 범위, help stdout 핵심 행, exit code, unsupported stderr 형식, future command reservation, non-goal을 설명하며 storage, SQL, WAL 구현을 이번 범위에 포함하지 않습니다.
- 변경 범위는 Rust `db` binary contract와 smoke baseline에 한정되고 network service, multi-process behavior, distributed behavior를 추가하지 않습니다.

## 검증 계획
- 필수 command: `cargo test`
- 필수 smoke command: `cargo run --bin db -- --help`
- 필수 help alias smoke command: `cargo run --bin db -- help`
- 필수 unsupported argument 검증: `cargo run --bin db -- --unknown`
- 필수 unsupported reserved subcommand 검증: `cargo run --bin db -- open demo.db`
- 필수 automated test target: `tests/cli_contract.rs`
- 필수 문서 산출물: `docs/cli_contract.md`
- 최종 리포트는 위 command의 exit code, stdout/stderr 요약, `tests/cli_contract.rs` 존재 여부, `docs/cli_contract.md` 존재 여부를 verification evidence로 연결해야 합니다.
- 기대 증거: scheduler terminal result, final report verification section, command output evidence, test output evidence, artifact delta summary.

## 리스크 및 에스컬레이션
- 알려진 리스크: scheduler 또는 spec hardening이 초안을 거절할 수 있습니다.
- 이후 review 또는 execution이 명시적으로 escalate할 때만 사람 승인이 필요합니다.
