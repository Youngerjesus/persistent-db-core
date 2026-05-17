# Research: v1-bench-docs-acceptance

## 목적
승인된 contract를 바꾸지 않고 V1 benchmark lower-bound evidence와 acceptance documentation을 구현할 수 있는 최소 설계를 확정한다.

## 결정 1: benchmark runner는 repo-local Bash script로 구현한다
- Decision: `scripts/verify_bench_acceptance`는 Bash로 작성하고, timing, temp DB lifecycle, environment collection, JSON emission을 script 안에서 처리한다.
- Rationale: 기존 `scripts/verify`, `scripts/verify_crash_matrix`, `scripts/verify_differential_property`가 repo-local script evidence 패턴을 사용한다. 새 Rust binary 또는 CLI subcommand를 추가하지 않아도 contract의 runnable benchmark surface를 충족한다.
- Consequences: JSON escaping과 timing arithmetic은 Bash에서 조심해야 한다. 필요하면 embedded one-shot Python을 JSON emission에만 사용할 수 있지만, benchmark execution itself는 반드시 `cargo run --quiet --bin db -- exec <temp-db> <sql>` 기반이어야 한다.

## 결정 2: benchmark workload는 two-scenario evidence로 고정한다
- Decision: 구현은 `bench_insert_1k`와 `bench_reopen_select_1k`만 측정한다.
- Rationale: 이번 spec은 V1 full performance suite가 아니라 lower-bound acceptance evidence를 고정하는 좁은 task다. `docs/v1_spec.md`의 historical `db bench` performance gate는 아직 reserved future CLI로 남는다.
- Consequences: `docs/benchmarks.md`는 1,000-row acceptance lower bound만 주장해야 하며 100k, secondary index, recovery proportionality, arbitrary hardware performance를 완료로 주장하면 안 된다.

## 결정 3: pass/fail은 minimum measured iteration으로만 판단한다
- Decision: warmup 1회는 pass/fail에서 제외하고 measured 3회 모두 threshold 이상이어야 한다. `observed_min_rows_per_second`가 threshold 이상일 때만 scenario pass다.
- Rationale: contract가 평균, 중앙값, 최선값 통과를 금지한다.
- Consequences: output summary는 best/average를 강조하지 않는다. JSON의 `observed_min_rows_per_second`가 authoritative lower-bound value다.

## 결정 4: acceptance guide는 evidence map이며 progress projection이 아니다
- Decision: `docs/v1_acceptance.md`는 current-artifact launch gates와 requirement ids를 모두 행으로 매핑하되, evidence가 없는 항목은 `blocked` 또는 `out_of_scope_for_this_task`로 명시한다.
- Rationale: contract가 progress projection만으로 완료 처리하는 것을 금지한다.
- Consequences: `req-v1-secondary-index-proof`는 현재 progress상 missing evidence이므로 완료로 표기하면 안 된다. 이번 task의 직접 완료 대상은 `req-v1-benchmark-lower-bounds`와 `req-v1-acceptance-docs`다.

## 결정 5: CLI contract update는 최소 참조만 허용한다
- Decision: `docs/cli_contract.md`는 `bench <path>` reserved future command 상태를 유지한다. 변경이 필요하면 benchmark evidence가 script-only임을 참조하는 문구로 제한한다.
- Rationale: user-facing CLI 확장은 out of scope다.
- Consequences: `src/main.rs`와 CLI dispatch tests는 benchmark command를 추가하지 않는다. 기존 `bench_reserved_future_command_remains_unsupported` 성격의 test는 유지되어야 한다.

## 미해결 질문
없음. 구현자가 새 product decision 없이 실행 가능하다.
