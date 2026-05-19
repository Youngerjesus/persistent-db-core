# Research: v1-section14-benchmark-acceptance

## 목적
승인된 Section 14 계약을 바꾸지 않고 100000-row public `db bench` acceptance evidence를 구현하기 위한 기술 선택과 위험 제어를 확정한다.

## 결정 1: benchmark core는 `src/bench.rs` 라이브러리 모듈로 둔다
- Decision: deterministic fixture 생성, 측정, evidence JSON 작성, hard-fail 검증은 `persistent_db_core::bench`에 구현하고 `src/main.rs`의 `db bench` dispatch가 이를 호출한다.
- Rationale: `db bench`가 public CLI여야 하므로 script-only runner는 계약 위반이다. 동시에 benchmark 로직을 `main.rs`에 넣으면 테스트와 hard-fail regression을 고정하기 어렵다.
- Consequences: `src/lib.rs`에 `pub mod bench;`를 추가한다. 모듈 API는 public CLI와 tests가 공유하되 Section 14 범위 밖 일반 benchmark framework로 확장하지 않는다.

## 결정 2: evidence JSON은 std-only serializer로 고정한다
- Decision: 새 JSON crate를 추가하지 않고 benchmark evidence 구조체를 직접 JSON string으로 직렬화한다.
- Rationale: repo 규칙은 V1에서 task-level 이유 없는 dependency 확장을 피하라고 한다. Evidence schema가 고정되어 있어 small escaping helper로 충분하다.
- Consequences: 문자열 필드는 JSON escaping helper를 반드시 통과한다. Tests는 top-level required field와 sentinel string을 black-box 또는 string-level로 검증한다.

## 결정 3: indexed-vs-scan comparison은 harness explicit scan mode로 만든다
- Decision: indexed path는 existing SQL executor/query planner path를 사용하고, scan comparison은 benchmark harness 안에서 동일 in-memory deterministic fixture를 explicit full-scan evaluator로 계산한다. 두 path의 result count와 deterministic hash가 일치해야 한다.
- Rationale: public SQL contract는 eligible secondary predicates가 full-scan으로 fallback되는 것을 금지한다. Index를 비활성화하는 public SQL knob을 새로 추가하면 scope가 넓어진다.
- Consequences: evidence의 `index_use_evidence.observed_access_path`는 existing `plan_query_path_for_test` 또는 equivalent internal planning proof에서 `SecondaryIndexEquality`/`SecondaryIndexRange`를 얻어 기록한다. Explicit scan evaluator는 latency lower-bound 비교와 result hash comparison에만 사용된다.

## 결정 4: runtime cap과 flakiness는 retry 없이 hard fail한다
- Decision: `runtime_cap_seconds`는 300 이하로 기록하고 초과, missing field, non-positive metric, speedup threshold 미달, retry-needed state를 모두 hard fail로 처리한다.
- Rationale: 계약이 flakiness retry 필요 상태를 hard fail로 명시한다.
- Consequences: script와 CLI는 재시도하지 않는다. 실패 시 `DB_BENCH: FAIL check=<check_id> reason=<reason>` 또는 `BENCH_ACCEPTANCE: FAIL check=<check_id> reason=<reason>`로 종료한다.

## 결정 5: recovery evidence는 10000 committed insert workload를 별도 DB에서 측정한다
- Decision: benchmark dataset DB와 별도로 recovery DB를 만들고 10000 committed transactions 뒤 close/reopen timing, WAL bytes, recovered count, representative lookup을 기록한다.
- Rationale: Section 14 recovery bound는 100000-row query benchmark와 별개로 `committed_transaction_count=10000`을 고정한다.
- Consequences: `recovery_ms <= max(2000, wal_file_bytes / 4096)`를 benchmark core와 verifier script 양쪽에서 검증한다.

## 결정 6: docs는 older 1k benchmark evidence를 대체하지 않고 Section 14 current evidence로 분리한다
- Decision: `docs/benchmarks.md`, `docs/performance_report.md`, `docs/v1_acceptance.md`, `docs/bug_diary.md`, `docs/cli_contract.md`는 Section 14 artifact IDs와 `target/bench_acceptance/section14-benchmark-acceptance.json`를 명시한다.
- Rationale: 기존 `v1-bench-docs-acceptance` evidence는 1k lower-bound 중심이라 이번 current artifact를 satisfied로 만들 수 없다.
- Consequences: 기존 historical claims는 필요 시 과거 evidence로 남기되, current acceptance row는 Section 14 command/evidence path만 통과 근거로 사용한다.

## 결정 7: `INTEGER`는 Section 14에 필요한 좁은 SQL type alias로 추가한다
- Decision: benchmark fixture를 내부 storage bypass로 만들지 않고 public SQL path에서 `INTEGER`를 `INT`와 동일한 integer type alias로 파싱한다. `INT`의 기존 출력/저장/semantic behavior는 유지한다.
- Rationale: `contracts.md`가 table schema와 `primary_key_type="INTEGER"`를 고정한다. 내부 fixture helper로 우회하면 public `db bench`가 실제 contracted SQL schema를 생성했다는 증거가 약해지고, SQL executor/parser drift를 숨길 수 있다.
- Consequences: 구현 범위는 `src/sql.rs`의 type parser, `tests/sql_exec.rs` 또는 focused SQL parser/CLI regression, `docs/sql_subset.md`와 `docs/cli_contract.md`의 type spelling 문서 업데이트로 제한한다. `INTEGER`는 `CREATE TABLE` type spelling alias로만 추가하며 projection, affinity, other SQLite type behavior, storage format rewrite는 하지 않는다.

## 결정 8: final command traceability는 verifier script가 같은 evidence file을 갱신한다
- Decision: `db bench`는 benchmark 수행 직후 같은 evidence path에 `commands.db_bench.status="pass"`와 `commands.verify_bench_acceptance.status="pending"`을 기록하고 `DB_BENCH` sentinel을 출력한다. `scripts/verify_bench_acceptance`는 public `db bench`를 실행한 뒤 evidence를 검증하고, 성공한 경우 같은 JSON file을 원자적으로 갱신해 `commands.verify_bench_acceptance.status="pass"`와 `BENCH_ACCEPTANCE` sentinel, final `result="pass"`를 기록한다.
- Rationale: `db bench` 실행 시점에는 verifier script가 아직 통과하지 않았으므로 verifier pass를 미리 쓰면 overclaim이다. 하지만 final acceptance evidence에는 두 command의 pass traceability가 필요하다.
- Consequences: implementation tests distinguish pre-verifier and post-verifier evidence lifecycle. The script must revalidate the updated JSON after writing the verifier command entry so it does not invalidate the `DB_BENCH` path or schema contract.

## 결정 9: hard-fail rejection은 validator negative test로 증명한다
- Decision: benchmark module exposes a narrow validation function used by tests to validate `index_use_evidence` and `hard_fail_checks`. A focused regression constructs evidence where an eligible equality or range query has `observed_access_path="full_scan"` and asserts non-zero/failure with stable `check_id`.
- Rationale: passing evidence alone can be optimistic. `FAIL-14-5` requires proof that the rejection mechanism fails closed when eligible indexed paths are reported or observed as full scans.
- Consequences: `tests/bench_acceptance.rs` includes both positive evidence checks and negative validator tests for `indexed_equality_no_full_scan` and `indexed_range_no_full_scan`. The script continues to validate file evidence; it does not need a public failure-injection CLI mode.

## 미해결 질문
없음. 구현자가 새 product decision 없이 실행 가능하다.
