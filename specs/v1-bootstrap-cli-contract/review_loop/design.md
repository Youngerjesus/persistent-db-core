# V1 `db` CLI 계약 및 smoke 테스트 기반 확정

## 정규화된 후보
- Rank: 1
- Feature slug: v1-bootstrap-cli-contract
- Target boundary: managed_repo
- Selection type: current_objective_blocker
- Confidence: high

## 문제 정의
V1 DB의 storage, SQL, index, WAL 작업은 모두 실행 가능한 `db` binary 계약에 의존하지만, 현재 repo에는 CLI skeleton만 있고 help 출력, command dispatch, unsupported argument behavior, smoke test evidence가 아직 formalized되어 있지 않습니다.

## 지금 해야 하는 이유
Current Objective가 CLI contract를 첫 dependency로 지정하고, Current Plan의 high-priority next candidate가 `gap-v1-bootstrap-cli-contract`이며, queue와 evidence cache 모두 중복 또는 진행 중 작업을 보고하지 않습니다.

## 기대 산출물 변화
`db --help`의 안정적인 command surface, deterministic exit behavior, command dispatch tests, CLI contract docs, `cargo test` 및 `cargo run --bin db -- --help` 검증 기반이 managed repo에 생깁니다.

## 의도한 변경 대상
- src/main.rs
- Cargo.toml
- tests/cli_contract.rs
- docs/cli_contract.md
- route:db-help
- flow:cli-command-dispatch

## Risk flags
- 없음

## 근거
- `ssot/current-objective.md`는 `metric-v1-cli-contract`를 첫 active success metric으로 두고, 모든 later gap이 runnable CLI contract에 의존한다고 명시합니다.
- `ssot/current-plan.md`는 `gap-v1-bootstrap-cli-contract`를 high priority로 두며 next candidate hint를 help output, command dispatch skeleton, smoke tests로 제시합니다.
- `ssot/current-artifact.md`의 `gate-v1-cli-smoke`는 `cargo test`, `cargo run --bin db -- --help`, command dispatch tests, CLI contract docs를 required evidence로 요구합니다.
- Root Progress Projection은 artifact_status가 `open`이고 `gate-v1-cli-smoke`에서 `req-v1-cli-help-smoke`, `req-v1-cli-dispatch-tests`가 open이라고 보고합니다.
- Active Managed Repo Snapshot은 repo가 clean이고 `Cargo.toml`, `Cargo.lock`, `src/main.rs`를 가진 Rust CLI skeleton 상태라고 보고합니다.
- Queue Snapshot은 `[]`이며 Gap Evidence Cache도 verified, queued, active task가 없다고 보고하므로 duplicate 또는 reserved work가 없습니다.
