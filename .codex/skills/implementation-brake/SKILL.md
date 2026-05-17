---
name: implementation-brake
description: Post-implementation review skill for stress-testing a completed code change before calling it done. Use when code already exists and the user wants implementation-level gaps, regression risks, missing tests, rigidity, or unnecessary complexity identified and then fixed through tdd-workflow.
---

# Implementation Brake

reference @./references/review-lenses.md
reference @./references/fix-handoff.md

이 스킬은 이미 작성된 구현을 대상으로, "동작은 되는 것 같은데 진짜 안전하게 닫혔나"를 압박 검토하는 구현 전용 브레이크 스킬입니다.
기획/방향 검토가 아니라 코드, 테스트, 경계 조건, 회귀 위험, 불필요한 복잡도를 다시 보고, 기본적으로는 `tdd-workflow`를 통해 수정까지 이어갑니다.
최근 diff 가 없어도 리뷰할 수 있지만, 근거 없는 추측 finding 은 허용하지 않습니다. 현재 코드, spec/contracts, 호출 경로, 테스트, 로그, 문서 중 필요한 증거를 모아 "이 구현을 닫혔다고 판단할 수 있는가"를 봅니다.

## Trigger

다음 상황에서 이 스킬을 우선 고려합니다.

- 사용자가 "구현 허점 봐줘", "이 구현 브레이크 걸어줘", "코드 쪽에서 놓친 거 찾아줘", "구현 리뷰하고 바로 고쳐" 같은 요청을 할 때
- 구현은 끝났지만 ship 전에 correctness, regression, edge case, test gap 을 다시 압박 검토하고 싶을 때
- bounded feature, bugfix, refactor, review feedback 반영 뒤에 실제 구현 품질을 다시 확인하려고 할 때

다음 경우에는 다른 스킬이 더 적합할 수 있습니다.

- 방향 자체가 맞는지 의심되면 `decision-brake`
- 아직 무엇을 만들지 정하는 단계면 `office-hours`
- 구현 자체를 처음 시작하는 bounded code change 면 `tdd-workflow`
- 큰 멀티 마일스톤 구현 orchestration 이면 더 상위 planning 스킬

## Core Posture

- findings first 입니다. 칭찬보다 결함, 누락, 리스크를 먼저 냅니다.
- 리뷰는 실제 defect risk 중심이어야 합니다. 취향성 nitpick 은 지양합니다.
- 구현이 맞는지 판단할 때 코드와 검증 증거를 함께 봅니다. diff 는 가능한 증거 중 하나일 뿐입니다.
- accepted finding 은 수정, 검증, 인간 결정, 증거 수집 중 하나로 닫힐 수 있어야 합니다. 그 외는 finding 이 아니라 note 입니다.
- `must fix now`이고 observable behavior 로 환원 가능한 finding 만 `tdd-workflow`의 다음 failing test 로 넘깁니다.
- broad cleanup 을 핑계로 scope 를 늘리지 않습니다.

## Workflow

### 1. Ground the implementation

먼저 review mode 를 고르고 아래를 짧게 고정합니다.

- **recent diff review**: 최근 변경분 중심으로 구현 결함과 회귀 위험을 봅니다.
- **current-state audit**: 오래된/큰 구현을 현재 코드, spec/contracts, 호출 경로, 테스트, 로그, 문서 기준으로 검토합니다.
- 무엇을 구현한 것인지
- 어떤 동작/버그/리팩터링을 닫으려는 것인지
- 어떤 behavior/spec/contracts 기준으로 판단할 것인지
- 관련 코드 경계와 현재 테스트 상태가 무엇인지

`current-state audit` 는 대상 behavior, 기준 문서 또는 계약, 코드 경계를 먼저 고정할 수 있을 때만 사용합니다.
필요하면 관련 diff, 현재 코드, 테스트, 호출 경로, 로그, 문서를 읽고 현재 구현의 실제 shape 을 파악합니다.
diff 가 없거나 노이즈가 크면 현재 코드와 계약/호출 경로를 중심 증거로 삼습니다.

### 2. Produce findings first

`references/review-lenses.md`를 참고해 현재 구현에서 중요한 렌즈만 사용합니다.

각 finding 은 아래를 포함해야 합니다.

- finding kind
  - **behavior defect**: 구현이 계약, 의도, 호출자 기대와 다릅니다.
  - **missing behavior**: 필요한 동작, 실패 경로, 상태 처리, 복구 경로가 없습니다.
  - **verification gap**: 구현은 있어 보이나 테스트 또는 다른 검증 증거가 부족합니다.
  - **decision gap**: 구현 문제가 아니라 요구사항, 정책, 소유권, 위험 수용 판단이 필요합니다.
- 무엇이 약하거나 빠졌는지
- 어떤 의도/spec/contracts/호출자 기대와 충돌하는지
- 왜 실제 defect risk 인지
- 어떤 risk category 인지
  - correctness
  - regression
  - edge/failure path
  - test gap
  - maintainability
  - performance
  - security
  - architecture drift
- disposition 이 무엇인지
- 무엇을 바꿔야 하거나 어떤 증거/결정이 필요한지

`missing scenario`, `conflated scenario`, `follow-up required`는 top-level finding kind 로 쓰지 않습니다.
scenario 누락이나 경로 뭉개짐은 `scenario/stateful path` 렌즈로 설명하고, 후속 조치는 disposition 으로 표현합니다.

### 3. Decide what must be fixed now

finding 을 아래 disposition 으로 분리합니다.

- **must fix now**: ship 전에 닫아야 하는 결함 또는 test gap
- **can defer**: 지금 닫지 않아도 되는 개선 또는 cleanup
- **blocked on evidence**: 재현, 테스트, 로그, 코드 확인이 먼저 필요함
- **blocked on human decision**: 요구사항, 정책, 소유권, 위험 수용 판단이 먼저 필요함

기본값은 보수적입니다. correctness, regression, missing failure-path coverage 는 defer 하지 않습니다.
`blocked on evidence` 와 `blocked on human decision` 은 production patch 로 바로 넘기지 않습니다.

### 4. Check scenario/stateful coverage when needed

모든 implementation review 에 `scenario-brake` 전체 workflow 를 적용하지는 않습니다.
다만 아래 신호가 있으면 scenario lens 를 선택적으로 사용해 구현 후 결함과 검증 공백을 찾습니다.

- state transition, retry, resume, restart, replay
- scheduler, background job, operator/manual entry
- external dependency, timeout, partial failure, delayed consistency
- idempotency, ownership, ordering, uniqueness, stale exclusion
- 운영자가 로그/상태를 보고 후속 조치를 판단해야 하는 경로

이때 필요한 렌즈만 고릅니다.

- Entry
- Actor
- State
- Data
- Dependency
- Recovery
- Environment
- Invariant
- Observability

새 시나리오 체계를 설계하거나 계획 자체의 scenario coverage 를 다시 짜야 하면 이 스킬 안에서 해결하지 말고 `scenario-brake`로 넘깁니다.

### 5. Hand off into TDD repair

사용자가 review-only 를 명시하지 않았다면, `must fix now`이고 observable behavior 로 환원 가능한 finding 만 `references/fix-handoff.md` 규칙으로 `tdd-workflow`에 연결합니다.

핵심 원칙은 다음과 같습니다.

- finding 하나를 behavior 하나로 환원합니다.
- 가장 작은 failing test 를 먼저 씁니다.
- production code 는 red 확인 뒤에만 수정합니다.
- targeted test 후 관련 상위 스위트까지 다시 검증합니다.
- `blocked on evidence` 는 먼저 증거 수집으로 닫습니다.
- `blocked on human decision` 은 인간 결정 전까지 코드 수정으로 넘기지 않습니다.

### 6. Re-review after fixes

수정이 끝나면 다시 구현 검토 posture 로 돌아와 아래를 확인합니다.

- 원래 finding 이 실제로 닫혔는가
- 수정이 다른 리스크를 새로 만들지 않았는가
- broader verification 이 충분했는가

### 7. Decide ship readiness

`[SHIP]` 은 단순히 "리뷰 finding 이 없다"가 아니라 현재 구현을 닫아도 된다는 authoritative ship-readiness verdict 입니다.
`[SHIP]` 전에 아래 증거를 명시적으로 확인합니다.

- acceptance criteria, spec, contracts, 또는 호출자 기대가 어떤 증거로 닫혔는가
- 관련 targeted test, broader suite, browser/e2e, 로그, 수동 검증 중 무엇을 실행했고 결과가 무엇인가
- 구현 중 plan/spec drift 가 생겼는가, 생겼다면 현재 ship decision 과 충돌하지 않는가
- 남은 residual risk 는 무엇이고 왜 현재 ship 을 막지 않는가
- 남은 follow-up 이 있다면 현재 acceptance 와 분리 가능하며 별도 작업으로 미뤄도 되는가

이 중 하나라도 missing, contradicted, 또는 판단 불가능하면 `[SHIP]` 대신 `[FIX BEFORE SHIP]` 또는 `[NEEDS REWORK]` 를 사용합니다.
commit, report, documentation sync, progress update, PR, merge 는 이 스킬의 책임이 아닙니다. `[SHIP]` 이후 운영적 마감은 `closeout` 같은 별도 workflow 로 넘깁니다.

## Output Shape

응답은 아래 순서를 기본으로 합니다.

1. **Implementation under review**
2. **Findings**
3. **What must be fixed now**
4. **Open blockers**
5. **What can be deferred**
6. **Fix plan**
7. **Verification result**
8. **Ship-readiness verdict**

판정은 반드시 아래 중 하나를 사용합니다.

- `[SHIP]`
- `[FIX BEFORE SHIP]`
- `[NEEDS REWORK]`

## Companion Agent Routing

`implementation-brake`가 최종 ship-readiness verdict 를 소유합니다.
Companion reviewer 는 병렬 판정자가 아니라 evidence input 입니다. companion 이 `[SHIP]` 의견을 내도 참고 신호일 뿐이며, final `[SHIP]`, `[FIX BEFORE SHIP]`, `[NEEDS REWORK]` verdict 는 항상 main `implementation-brake`가 냅니다.
`closeout`은 `[SHIP]` 이후의 얇은 운영적 마감 executor 이며, full implementation review 를 `closeout`으로 옮기지 않습니다.

### Companion Roles

- `implementation-brake-reviewer`: high-risk implementation review companion 입니다. stateful/recovery/shared-contract risk, edge/failure path, verification gap, concrete complexity risk 를 봅니다. red-team 성격의 additive bias, silent failure, cross-category integration gap 렌즈도 이 agent 가 흡수하되, 모든 finding 은 concrete defect risk, verification gap, 또는 ship-readiness risk 로 환원되어야 합니다.
- `code-reviewer`: merge-gate companion 입니다. correctness, regression, completeness, merge safety 에 집중하며, implementation-brake-specific reviewer 와 독립된 좁은 렌즈입니다. merge-risk 가 있는 non-trivial diff 에서 별도 read-only evidence input 으로 사용할 수 있습니다.
- `performance-reviewer`: conditional specialist 입니다. query, loop, async/request path, rendering, bundle, unbounded data, cache/queue/resource-risk 같은 concrete performance trigger 가 있을 때만 호출합니다. 성능 리스크가 없는 diff 에서는 호출하지 않습니다.

### When to Invoke

아래 중 하나라도 해당하면 `implementation-brake-reviewer`를 read-only companion 으로 호출하는 것을 기본값으로 삼습니다.

- scheduler, retry, resume, restart, queue, persistence, policy, auth 경계가 바뀐 경우
- 외부 side effect, stateful/recovery behavior, shared contract, 다중 모듈 변경이 포함된 경우
- non-trivial `must fix now` 반영 후 재검토가 필요한 경우
- main reviewer 가 구현자와 같아서 단일 시선의 `[SHIP]` 판단이 위험한 경우
- 사용자가 독립된 implementation review pass 를 명시적으로 원한 경우

아래 신호가 있으면 `code-reviewer`를 merge-gate companion 으로 호출합니다.

- 요구사항 충족 여부, 기존 호출자 회귀, 누락된 상태/분기 처리, merge safety 를 별도 시선으로 확인해야 하는 경우
- diff 가 작더라도 실패 시 사용자 동작이나 scheduler/task 상태를 잘못 바꿀 수 있는 경우
- companion output 을 바탕으로 final verdict 를 강화해야 하지만 full implementation-brake companion 까지는 필요하지 않은 경우

아래 신호가 있으면 `performance-reviewer`를 conditional companion 으로 호출합니다.

- query shape, pagination, filtering, sorting, batching, or indexing assumption 변경
- loop, nested collection work, unbounded data, repeated scan/sort/filter 경로
- async/request path, blocking I/O, network fan-out, retry/polling/resource-sensitive background path
- frontend render-time work, large list/chart rendering, animation, unstable references, bundle-heavy imports

아래처럼 좁고 저위험인 단일 경로 변경에서는 optional specialist routing 없이 main `implementation-brake`가 직접 종료할 수 있습니다.

- main pass 가 diff, contract, targeted test 를 직접 확인할 수 있음
- state/recovery/shared-contract 변화가 없음
- merge-risk 또는 performance-risk trigger 가 없음
- 남은 finding 이 없거나 separable follow-up 으로 명확히 분리됨

런타임 정책, 도구 제한, 사용자 승인 조건 때문에 companion delegation 이 불가능하면 main `implementation-brake`가 필요한 렌즈를 직접 적용합니다.
이 경우 review 결과에 companion fallback 이었음을 기록하고, 생략 사유와 적용한 렌즈를 남깁니다.

### Reconcile Companion Findings

companion finding 은 병렬 verdict 가 아니라 main `implementation-brake`가 조정해야 하는 evidence input 입니다.
각 finding 은 아래 중 하나로 정리합니다.

- **accepted + ship blocker**: `must fix now`
- **accepted but separable**: `can defer`
- **plausible but unproven**: `blocked on evidence`
- **requirement/policy judgment needed**: `blocked on human decision`
- **rejected**: code, spec, contract, test, log 같은 구체 증거로 reject reason 기록

unresolved accepted companion finding 이 하나라도 있으면 `[SHIP]`을 낼 수 없습니다.
companion 이 직접 수정하거나 finding 을 닫지 않습니다. 수정은 기존처럼 `tdd-workflow` repair 흐름으로 넘깁니다.

## Constraints

- pre-implementation 의사결정 검토를 대신하지 않습니다. 그 경우 `decision-brake`를 사용합니다.
- review-only 가 아닌 이상 findings 보고에서 끝내지 않습니다.
- failing test 없는 무차별 patching 을 허용하지 않습니다.
- review cleanup 을 핑계로 큰 리팩터링을 밀어 넣지 않습니다.
