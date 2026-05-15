# 계약

## 강한 제약
- 명시적으로 escalate되지 않으면 SSOT 또는 policy 파일을 변경하지 않습니다.
- 현재 queue와 worktree topology invariant를 유지해야 합니다.
- Protected areas: ssot/, policies/.

## 코드 맥락 사용 계약
- `review_loop/code_context.md`와 `관찰된 코드 맥락` 섹션은 관찰 근거이며 구현 지시가 아닙니다. 실제 구현 전 worker는 최신 worktree에서 재검증해야 합니다.
- Worker는 task worktree의 최신 HEAD, dirty/conflict 상태, 관련 파일 존재 여부를 확인한 뒤 구현해야 합니다.
- 관찰된 파일 목록은 탐색 시작점일 뿐이며 acceptance criteria나 scope를 대체하지 않습니다.

## 필수 산출물
- 생성 대상 코드 또는 문서: `src/main.rs`의 V1 `db` CLI dispatch 계약과 `docs/cli_contract.md`.
- 생성 대상 테스트 또는 verification output: `tests/cli_contract.rs`, `cargo test`, `cargo run --bin db -- --help`, `cargo run --bin db -- help`, `cargo run --bin db -- --unknown`, `cargo run --bin db -- open demo.db`의 supporting evidence.
- 생성 대상 리포트 업데이트: final report verification section, run report, episode entry, 실행 중 필요해지는 human-request escalation.

## 실패 조건
- spec_loop가 package를 승인하지 않으면 task는 미완료입니다.
- 두 번째 recovery attempt가 필요해지면 즉시 escalate합니다.

## Acceptance Evidence Contract
- Each Candidate Acceptance Criteria item must connect to test output, browser evidence, command output, manual review evidence, or an explicit blocker.
- Do not weaken, merge away, or replace candidate acceptance criteria with generic completion wording during spec hardening.
- `cargo test`가 성공하고 help 및 unsupported argument dispatch를 검증하는 자동 테스트가 포함됩니다.
- `cargo run --bin db -- --help`가 exit code `0`, 빈 stderr, 아래 `Required Help Output Core Lines`와 일치하는 stdout을 반환합니다.
- `cargo run --bin db -- help`가 exit code `0`, 빈 stderr, `db --help`와 동일한 stdout contract를 반환합니다.
- `cargo run --bin db -- --unknown`과 `cargo run --bin db -- open demo.db`는 exit code `2`, 빈 stdout, 아래 `Required Unsupported Error Format`과 일치하는 stderr를 반환합니다.
- `tests/cli_contract.rs`는 help output, `db help` alias, unsupported argument, unsupported reserved subcommand를 deterministic automated test로 검증합니다.
- `docs/cli_contract.md`는 현재 지원 범위, help stdout 핵심 행, exit code, unsupported stderr 형식, future command reservation, non-goal을 설명하며 storage, SQL, WAL 구현을 이번 범위에 포함하지 않습니다.
- 변경 범위는 Rust `db` binary contract와 smoke baseline에 한정되고 network service, multi-process behavior, distributed behavior를 추가하지 않습니다.

## Required Help Output Core Lines
- Help stdout은 다음 행을 순서대로 포함해야 합니다. 행 문구와 순서는 acceptance contract입니다.

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

## Required Unsupported Error Format
- Unsupported argument 또는 subcommand는 stderr에 다음 두 행을 반환해야 합니다.

```text
error: unsupported argument or command: <token>
hint: run 'db --help' for the supported V1 CLI contract.
```

- `<token>`은 첫 번째 unsupported token입니다.
- 예: `db --unknown`의 `<token>`은 `--unknown`입니다.
- 예: `db open demo.db`의 `<token>`은 `open`입니다.

## Required Verification Commands
- `cargo test`
- `cargo run --bin db -- --help`
- `cargo run --bin db -- help`
- `cargo run --bin db -- --unknown`
- `cargo run --bin db -- open demo.db`

## Required Evidence Paths
- `docs/cli_contract.md`
- `tests/cli_contract.rs`
- scheduler final report의 verification section
- scheduler run report 또는 task run artifact의 command output evidence

## 완료 정의
- 구현이 존재하거나 blocker가 해소되어야 합니다.
- Acceptance criteria가 충족되어야 합니다.
- Verification proof가 첨부되어야 합니다.
- Artifact delta가 report에 반영되어야 합니다.
