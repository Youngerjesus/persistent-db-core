# 현재 SHA 기준 WAL 복구 증거 재검증

**Status**: APPROVED

## 메타데이터
- Run ID: 2026-05-18-00-55-20
- Task ID: task-2026-05-18-00-55-20-v1-wal-recovery-current-sha-proof
- Candidate rank: 1
- Target boundary: managed_repo
- Objective: 현재 SHA 기준 WAL 복구 증거 재검증
- Artifact: v1-wal-recovery-current-sha-proof

## 목표
- WAL recovery 기능은 최근 구현된 것으로 보이지만, 현재 artifact 판단에서는 prior manifest SHA `754958b37fd01f796b9d4f7522a2062b6e65abc5`가 현재 repo SHA `33b480cac6cf9d505a86eda4c149a4471454f11d`와 맞지 않아 `gate-v1-transactions-wal-recovery`가 닫히지 않습니다. 현재 task worktree HEAD에서 committed mutation 생존, uncommitted change 부재, incomplete trailing WAL entry 배제가 각각 다시 증명되지 않으면 V1 회복성 gate와 후속 crash matrix가 모두 불안정합니다.

## 지금 해야 하는 이유
- CLI, page storage, SQL, index gate는 Root Progress Projection에서 `projected_complete`이므로 오늘 후보에서 제외됩니다. 남은 high-priority recovery slice 중 WAL proof가 crash matrix보다 선행 조건이고, queue가 비어 있어 현재 SHA 검증을 가장 작은 독립 작업으로 handoff할 수 있습니다.

## 기대 산출물 변화
- 관리 레포의 `tests/wal_recovery.rs`, 관련 CLI smoke, WAL sidecar/recovery 문서 또는 evidence transcript를 현재 task worktree HEAD 기준으로 보강해 `req-v1-wal-recovery-proof`와 직접 매핑되는 deterministic recovery proof를 남깁니다.
- 최종 report 또는 evidence transcript는 `git rev-parse HEAD`, `git status --short`, 실행 명령, exit code, stdout, stderr, WAL sidecar 파일 상태를 함께 기록해야 합니다.

## 의도한 변경 대상
- tests/wal_recovery.rs
- docs/file_format.md
- docs/cli_contract.md
- src/main.rs
- src/lib.rs
- final report 또는 evidence transcript

## 관찰된 코드 맥락
- 이 섹션은 관찰 근거이며 구현 지시가 아닙니다. 실제 구현 전 worker는 최신 worktree에서 재검증해야 합니다.
- 관찰 기준 HEAD: 33b480cac6cf9d505a86eda4c149a4471454f11d
- Dirty state: none
- Raw evidence: review_loop/code_context.md
- 관련 파일 후보: tests/wal_recovery.rs, docs/file_format.md, docs/cli_contract.md, src/main.rs, src/lib.rs, src/storage.rs, specs/v1-transaction-wal-recovery/spec.md, specs/v1-transaction-wal-recovery/contracts.md, specs/v1-transaction-wal-recovery/qa_mapping.md, specs/v1-transaction-wal-recovery/impl_review.md

## Risk flags
- stale_evidence_sha_mismatch
- storage_recovery_semantics_boundary
- requires_current_sha_verification
- avoid_storage_format_change

## Daily Metric Loop
- Source: daily
- Final disposition: ready_for_handoff
- objective_plan_gap_fit: score=3
- causal_evidence_strength: score=3
- handoff_verifiability: score=3
- Constraint blockers: none

## 근거
- Root Progress Projection: artifact_status는 `open`이고 `gate-v1-transactions-wal-recovery`는 `missing satisfied requirement rows` 때문에 `open`입니다.
- Root Progress Projection: `req-v1-wal-recovery-proof`는 WAL replay가 committed changes survival과 uncommitted absence를 증명해야 하지만 현재 `open`입니다.
- Root Progress Projection: `gap-v1-transaction-wal-recovery`는 prior manifest repo SHA `754958b37fd01f796b9d4f7522a2062b6e65abc5`와 현재 repo SHA `33b480cac6cf9d505a86eda4c149a4471454f11d` 불일치로 `stale_needs_recheck`입니다.
- Current Plan: `gap-v1-transaction-wal-recovery`는 high priority이며 `metric-v1-recovery-correctness`와 `gate-v1-transactions-wal-recovery`에 연결됩니다.
- Current Artifact: `req-v1-wal-recovery-proof`는 recover 후 committed changes survival과 uncommitted changes absence를 요구합니다.
- Active Managed Repo Snapshot: queue는 비어 있고 git_status는 clean이며, 2026-05-18 history/progress는 minimal WAL recovery milestone이 추가되었다고 보고합니다.
- Root Progress Projection: CLI, disk storage, SQL, index gates는 `projected_complete`라 오늘 후보 범위에서 제외됩니다.
- src/storage.rs
- tests/wal_recovery.rs
- docs/file_format.md
- docs/cli_contract.md
- specs/v1-transaction-wal-recovery/spec.md
- specs/v1-transaction-wal-recovery/contracts.md
- specs/v1-transaction-wal-recovery/qa_mapping.md
- specs/v1-transaction-wal-recovery/impl_review.md
- specs/v1-transaction-wal-recovery/code_review.md
- specs/v1-transaction-wal-recovery/final_review.md
- specs/v1-transaction-wal-recovery/review_loop/metric_loop_evidence.md
- work_queue/progress.md
- autopilot/ssot/current-artifact.md
- autopilot/project_manager/tasks/tasks.json
- PageStore::open
- PageStore::append_record
- replay_wal
- append_wal_frame
- next_wal_frame_id
- wal_checksum
- wal_path
- committed_wal_replay_survives_reopen_via_cli
- incomplete_wal_entry_is_not_replayed_without_public_rollback_cli
- committed_frame_after_incomplete_tail_cleanup_remains_replayable
- committed_wal_frame_ahead_of_page_store_fails_deterministically
- autopilot/project_manager/tasks/tasks.json#task-2026-05-17-23-45-17-v1-transaction-wal-recovery:SUCCESS
- autopilot/project_manager/tasks/tasks.json#artifact_gate:gate-v1-transactions-wal-recovery
- autopilot/project_manager/tasks/tasks.json#artifact_requirement_ids:req-v1-wal-recovery-proof
- autopilot/project_manager/tasks/tasks.json#repo_after_sha:754958b37fd01f796b9d4f7522a2062b6e65abc5

## 범위
- 범위 포함: 현재 SHA 기준 WAL recovery proof를 닫기 위한 `tests/wal_recovery.rs`, WAL sidecar/recovery 문서, 필요한 경우의 CLI smoke evidence 보강.
- 범위 포함: final report 또는 evidence transcript에 현재 managed repo SHA, dirty state, verification command output, CLI smoke output, WAL sidecar 파일 상태를 연결합니다.
- 범위 제외: unrelated breadth features, network service, multi-process concurrency, distributed storage, query optimization, UI route, browser/DOM/screenshot evidence.
- 범위 제외: public CLI stdout, stderr, exit code 변경. 단, 실제 결함 수정을 위해 계약 변경이 불가피하면 worker는 구현을 멈추고 spec/contract conflict로 보고해야 합니다.

## 수용 기준
- 선택된 candidate에 대한 구체적인 artifact delta가 존재해야 합니다.
- Daily metric loop evidence가 spec package와 일관되어야 합니다.
- 최종 리포트에 verification evidence가 연결되어야 합니다.

## Candidate Acceptance Criteria
- `cargo test --test wal_recovery`가 현재 task worktree HEAD에서 통과하고, committed mutation이 별도 `db exec` process의 reopen/replay 뒤 살아남는 케이스를 검증합니다.
- uncommitted change absence는 별도 deterministic scenario로 검증해야 합니다. 공개 CLI에 rollback 또는 uncommitted transaction command가 없으면 test fixture가 WAL bytes를 직접 작성할 수 있지만, final evidence는 그 fixture가 V1에서 관찰 가능한 uncommitted WAL state를 대표하는 이유를 기록해야 합니다.
- incomplete trailing WAL entry exclusion은 uncommitted change absence와 별도 deterministic scenario로 검증해야 합니다. 증거는 incomplete tail의 ghost row가 recovery 후 결과 rows 또는 storage row set에 나타나지 않고, 향후 replay 가능한 WAL sidecar 상태가 유지되거나 cleanup되었음을 기록해야 합니다.
- `./scripts/verify`가 통과해 fmt, clippy, full test suite, `db --help` smoke를 함께 보장합니다.
- CLI smoke transcript는 temp DB path를 생성한 뒤 `cargo run --bin db -- exec "$DB_PATH" "CREATE TABLE users (id INT, name TEXT); INSERT INTO users VALUES (1, 'ada'); INSERT INTO users VALUES (2, 'bea');"`를 실행하고, exit code `0`, stdout `""`, stderr `""`를 기록해야 합니다.
- CLI smoke transcript는 별도 process에서 `cargo run --bin db -- exec "$DB_PATH" "SELECT * FROM users;"`를 실행하고, exit code `0`, stderr `""`, stdout exactly `id|name\n1|ada\n2|bea\n`를 기록해야 합니다.
- CLI smoke transcript는 create/insert 후 `$DB_PATH.wal` 존재 여부와 byte length, reopen select 후 `$DB_PATH.wal` 존재 여부와 byte length를 기록해야 합니다. 구현이 complete WAL frames를 retained sidecar로 유지한다면 sidecar는 존재하고 non-empty여야 합니다.
- 최종 report 또는 evidence transcript는 `git rev-parse HEAD`, `git status --short`, `cargo test --test wal_recovery`, `./scripts/verify`, CLI smoke command의 실행 명령과 stdout/stderr 또는 transcript path를 기록해야 합니다.
- 최종 evidence는 `gap-v1-transaction-wal-recovery`, `gate-v1-transactions-wal-recovery`, `req-v1-wal-recovery-proof`에 명시적으로 매핑됩니다.

## 검증 계획
- 실행할 commands: `cargo test --test wal_recovery`, `./scripts/verify`, Candidate Acceptance Criteria에 명시된 CLI smoke commands.
- 기대 증거: run report 또는 final report, current HEAD SHA, dirty state, command output, CLI smoke transcript, WAL sidecar file-state summary, gate/requirement mapping.
- 브라우저, DOM, screenshot, rendered route state, UX design review evidence는 이 작업의 검증 범위가 아닙니다.

## 리스크 및 에스컬레이션
- 알려진 리스크: scheduler 또는 spec hardening이 초안을 거절할 수 있습니다.
- 이후 review 또는 execution이 명시적으로 escalate할 때만 사람 승인이 필요합니다.
