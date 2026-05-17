---
name: plan-eng-review
description: Lightweight pre-implementation engineering review for short-cycle work. Use when the user already has direction plus a mini-plan and wants a hard check on scope, reuse, architecture, tests, and failure modes before coding. Conditionally invokes scenario-brake when state, path, or recovery complexity makes happy-path planning unsafe.
---

# Plan Eng Review

이 스킬은 짧은 주기 개발에서, 구현 직전에 "이 plan으로 바로 들어가도 되나"를 빠르게 압박 검토하는 엔지니어링 리뷰 스킬입니다.
풀 `specs/*` 파이프라인을 대체하지 않습니다.
이미 방향과 mini-plan 이 있는 상태에서 scope, reuse, architecture, test gap, failure mode 를 점검하는 preflight gate 역할을 합니다.

## Trigger

다음 상황에서 이 스킬을 우선 고려합니다.

- 사용자가 "plan 리뷰해줘", "엔지니어링 리뷰해줘", "구현 들어가기 전에 sanity check 해줘", "이대로 build 해도 되나?" 같은 요청을 할 때
- 짧은 주기 작업이지만 아래 조건 중 2개 이상이 보일 때
  - 3개 이상 파일 또는 2개 이상 모듈이 바뀐다
  - 외부 API, auth, payment, observability, state transition 이 들어간다
  - 기존 동작을 건드리는 회귀 위험이 있다
  - 구현 선택지가 2개 이상 남아 있다
  - E2E 또는 operator-facing verification 이 필요하다

다음 경우에는 다른 스킬이 더 적합할 수 있습니다.

- 방향 자체가 맞는지 의심되면 `decision-brake`
- UI/UX completeness 를 구현 전에 압박 검토하고 싶으면 `plan-design-review`
- 시나리오 누락이 핵심 우려면 `scenario-brake`
- 구현이 이미 끝났고 코드/테스트 허점을 다시 보고 싶으면 `implementation-brake`
- full `specs/*` SDD phase review 를 닫아야 하면 spec-local `plan_review.md` 흐름과 해당 phase 문서를 우선한다

다음 경우에는 이 스킬이 과합니다.

- 명백한 단일 버그 수정
- copy/style/rename 수준의 저위험 변경
- 기존 패턴 그대로의 단순 single-path 작업
- 테스트만 보강하는 작은 변경

## Core Posture

- 목적은 문서 절차가 아니라 구현 직전 engineering sanity check 입니다.
- 가장 먼저 "기존 것을 재사용할 수 있나"와 "범위를 더 줄일 수 있나"를 봅니다.
- review 는 findings first 이지만, nitpick 보다 실제 defect risk 와 scope waste 를 우선합니다.
- minimal diff, DRY, explicitness, edge-case thoughtfulness 를 기본값으로 둡니다.
- 결과는 기본적으로 대화에 보고합니다. 자동으로 `plan_review.md`, `TODOS.md`, 대시보드, telemetry 를 남기지 않습니다.

## Workflow

### 1. Ground the plan

먼저 아래를 짧게 고정합니다.

- 무엇을 만들거나 바꾸려는지
- 사용자에게 어떤 결과가 바뀌는지
- 현재 mini-plan 또는 관련 문서가 무엇인지
- 직접 SSOT 가 있다면 어디까지인지

필요한 만큼만 관련 코드, 기존 흐름, plan 문서를 읽습니다.
짧은 작업에서 과수집하지 않습니다.

### 2. Step 0: Scope challenge

반드시 아래 질문부터 봅니다.

- 이 문제를 이미 부분적으로 해결하는 기존 코드/flow 가 무엇인가
- 병렬 구현 대신 기존 output 을 capture 하거나 기존 adapter 를 재사용할 수 있는가
- stated goal 을 닫는 최소 변경 집합은 무엇인가
- 지금 plan 이 결과 대비 과하게 넓거나 깊은가

obvious leaner path 가 보이면 먼저 그 방향을 권고합니다.
이 단계의 목적은 좋은 architecture 를 칭찬하는 것이 아니라, 불필요한 작업을 제거하는 것입니다.

### 3. Architecture review

아래를 점검합니다.

- component boundary 와 ownership
- dependency direction 과 coupling
- data flow 와 integration point
- reversibility 와 rollback friendliness
- single point of failure 또는 awkward coordination point

짧은 주기 작업에서는 top 3-5 issue 만 남깁니다.
문제마다 "무엇이 깨질 수 있는지"와 "더 단순한 대안이 있는지"를 함께 적습니다.

### 4. Test review

plan 기준으로 main codepath 와 user flow 를 추적합니다.
가능하면 compact ASCII diagram 으로 요약합니다.

반드시 확인할 것:

- happy path 외 분기
- existing behavior change 에 대한 regression test 필요 여부
- unit / integration / E2E 중 어떤 수준이 맞는지
- test gap 이 실제 user-visible failure 로 이어지는지

test review 는 "테스트가 있으면 됨"이 아니라, 이 plan 으로 구현했을 때 무엇이 증명돼야 하는지 보는 단계입니다.

### 5. Failure-mode review

2-5개의 realistic production failure 를 적습니다.
각 failure 마다 아래를 본다.

- plan 이 이를 예방하는가
- plan 이 실패를 처리하는가
- user 또는 operator 가 명확히 볼 수 있는가
- silent failure 로 끝날 가능성이 있는가

test 없음 + handling 없음 + silent failure 인 경우는 강하게 지적합니다.

### 6. Conditional scenario-brake hook

기본값은 `scenario-brake`를 자동 실행하지 않는 것입니다.
아래 신호가 보일 때만 추가 호출 또는 권고합니다.

- 의미 있는 state 가 3개 이상 있다
- 동일 흐름에 entry path 가 2개 이상 있다
- retry, resume, refresh, back, reopen 같은 re-entry 경로가 중요하다
- auth, payment, quota, session-expiry 같은 조건 분기가 핵심이다
- UI 와 backend 가 다른 타이밍으로 실패할 수 있다
- `partial`, `stale`, `empty`, `delayed` 같은 상태가 제품적으로 중요하다

이 경우 `scenario-brake`의 역할은 test count 확대가 아니라, plan 이 서로 다른 현실 경로를 같은 것으로 뭉뚱그렸는지 압박 검토하는 것입니다.

위 신호가 없으면 scenario review 를 생략하고, 왜 happy-path bias 위험이 낮다고 판단했는지 한 줄로 적습니다.

### 7. Final recommendation

마지막에는 반드시 아래 중 하나로 판정합니다.

- `GO`
- `GO WITH CHANGES`
- `STOP`

최종 답변에는 아래를 포함합니다.

- review 대상
- scope cut recommendation 여부
- top issues
- top missing tests
- top failure modes
- `scenario-brake`를 skipped / recommended / used 중 무엇으로 처리했는지

## Output Shape

응답은 아래 순서를 기본으로 합니다.

1. **Plan under review**
2. **Scope challenge**
3. **Architecture findings**
4. **Test findings**
5. **Failure modes**
6. **Scenario review decision**
7. **Recommendation**

필요할 때만 마지막에 **Open questions** 를 붙입니다.
질문은 구현 방향을 바꾸는 tradeoff 가 있을 때만 남깁니다.

## Interaction Rules

- 기본은 한 번의 통합 리뷰 응답입니다.
- issue 마다 무조건 질문 루프를 돌리지 않습니다.
- genuine tradeoff 가 있을 때만 사용자의 결정을 요청합니다.
- 사용자 질문이 없더라도 recommendation 은 명확하게 냅니다.
- review 를 핑계로 새로운 artifact 체계를 들이밀지 않습니다.

## Relationship To Other Skills

- `decision-brake`: 방향, 문제정의, 대안 선택이 흔들릴 때
- `scenario-brake`: scenario coverage, re-entry, recovery, path separation 이 흔들릴 때
- `plan-design-review`: 구현 전 plan 의 UI/UX completeness 와 design decision gap 을 점검할 때
- `plan-eng-review`: 방향은 대체로 정해졌고, 구현 직전 engineering preflight 가 필요할 때

이 스킬은 `decision-brake`와 `scenario-brake` 사이에 끼어드는 것이 아니라, 짧은 주기 구현의 마지막 preflight review 역할을 맡습니다.

## Constraints

- full gstack-style orchestration 을 복제하지 않습니다.
- telemetry, session state, CLAUDE routing, dashboard, global review log 를 만들지 않습니다.
- 자동 문서 갱신을 기본값으로 두지 않습니다.
- full `specs/*` review workflow 를 대체하지 않습니다.
- 작은 작업을 불필요하게 무겁게 만들지 않습니다.
