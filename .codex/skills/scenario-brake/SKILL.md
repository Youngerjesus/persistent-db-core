---
name: scenario-brake
description: General-purpose pre-implementation review skill for pressure-testing whether a plan, spec, or design covers enough realistic scenarios. Use when the user wants a hard pass on missing scenarios, conflated paths, parameter variations, recovery paths, or whether a plan only covers the happy path.
---

# Scenario Brake

reference @./references/review-lenses.md

이 스킬은 구현 전에 계획안, 설계안, spec 초안을 대상으로 "이 문서가 실제로 충분한 시나리오를 커버하나"를 압박 검토하는 리뷰 스킬입니다.
테스트 종류를 늘리는 것이 목적이 아니라, 계획이 서로 다른 현실 경로를 하나의 문제로 뭉뚱그렸는지와 주요 파라미터를 바꿨을 때 빠지는 시나리오가 생기는지를 드러내는 것이 목적입니다.

## Trigger

다음 상황에서 이 스킬을 우선 고려합니다.

- 사용자가 "시나리오가 충분한지 봐줘", "happy path 말고 다른 경로를 더 생각해봐", "이 계획이 뭘 놓치고 있나?", "엣지 시나리오를 더 세게 봐줘" 같은 요청을 할 때
- 테스트 개수보다, 현실적인 변형 경로와 누락된 검증 대상을 더 중요하게 보고 싶을 때
- 같은 문제처럼 보이는 경로가 실제로는 다른 원인, 다른 진입 방식, 다른 복구 전략을 요구하는지 확인하고 싶을 때

다음 경우에는 다른 스킬이 더 적합할 수 있습니다.

- 방향 자체의 타당성과 대안을 검토하려면 `decision-brake`
- 구현이 이미 끝난 뒤 defect risk 와 test gap 을 보려면 `implementation-brake`
- spec/contracts 를 SSOT 수준으로 잠그려면 `spec-reviewer`

## Core Posture

- checklist 를 채우는 것이 아니라 scenario coverage 를 압박 검토합니다.
- 시나리오를 한 줄 사건으로 보지 않고, 그것을 구성하는 주요 파라미터를 먼저 봅니다.
- "엣지 케이스가 빠졌다" 같은 뭉뚱그린 말로 끝내지 않고, 무엇을 같은 경로로 잘못 취급했는지와 무엇을 분리해야 하는지 적습니다.
- 결과는 기본적으로 사용자에게 보고합니다. review 결과를 자동으로 spec/contract 에 반영하는 것을 기본값으로 두지 않습니다.

## Workflow

### 1. Ground the plan

먼저 아래를 짧게 고정합니다.

- 무엇을 계획 중인지
- 어떤 핵심 흐름을 닫으려는지
- 현재 문서가 이미 커버한다고 주장하는 시나리오가 무엇인지

필요하면 관련 문서, 코드, 과거 버그 맥락을 읽되, 중심은 현재 계획의 scenario coverage 입니다.

### 2. Identify scenario classes

현재 계획에서 중요한 scenario class 를 고릅니다.

- entry path
- actor
- state
- data shape
- dependency behavior
- recovery path
- environment/runtime context
- invariants
- observability

모든 class 를 기계적으로 다 쓰지 말고, 계획의 pass/fail 을 가장 많이 바꿀 것만 고릅니다.

### 3. Mutate key parameters

`references/review-lenses.md`를 참고해 baseline scenario 의 주요 파라미터를 식별하고 값을 바꿔 봅니다.

핵심 원칙은 다음과 같습니다.

- baseline scenario 를 먼저 한 문장으로 적습니다.
- 그 시나리오를 성립시키는 주요 파라미터를 3-7개 식별합니다.
- 파라미터를 하나씩 바꿔 변형 시나리오를 만듭니다.
- 필요한 경우 고위험 조합만 2개 이상 함께 바꿉니다.
- 새 시나리오가 기존 시나리오와 truly same path 인지, 별도 시나리오인지, 다른 시나리오의 선행/후속 경로인지 판정합니다.

### 4. Separate vs connect

특히 아래 질문을 우선합니다.

- 현재 계획이 서로 다른 진입 경로를 같은 시나리오로 뭉뚱그렸는가?
- 같은 실패 현상이 실제로는 다른 원인과 다른 복구 전략을 가지는가?
- 특정 시나리오의 후속 경로나 재진입 경로가 빠져 있는가?
- 한 파라미터 변화가 다른 scenario class 와 연결되며 전혀 다른 검증 전략을 요구하는가?

### 5. Recommend coverage additions

빠진 시나리오를 찾으면 아래를 함께 적습니다.

- 왜 기존 계획으로는 커버됐다고 보기 어려운지
- 어떤 변형 파라미터가 새 시나리오를 만들었는지
- 이후 어떤 수준의 검증이 필요한지
  - unit
  - integration
  - e2e
  - restart/replay
  - concurrency
  - manual ops check

## Output Shape

응답은 아래 순서를 기본으로 합니다.

1. **Scenario classes reviewed**
2. **Key parameters identified**
3. **Parameter mutations explored**
4. **Scenario links and separations**
5. **What the plan already covers**
6. **Missing or conflated scenarios**
7. **Highest-risk blind spots**
8. **Recommended scenario additions**
9. **Overall verdict**

판정은 반드시 아래 중 하나를 사용합니다.

- `[SCENARIOS SUFFICIENT]`
- `[SCENARIOS MISSING]`
- `[PLAN NEEDS REFRAME]`

## Compact Example

예:

- baseline: `resume 실패 후 같은 phase strict resume 시 no rollout found -> fresh session fallback`
- parameters: `persisted status`, `entry trigger`, `session lookup source`, `prior stale knowledge`
- mutation: `persisted status=FAILED`, `entry trigger=scheduler restart`, `lookup=find_session_id()`
- result: strict resume 시나리오와 다른 일반 재진입 경로가 생성되므로, stale session 재선택 위험은 별도 시나리오로 분리해야 함

이 예시는 상태 전이 사례를 보여주지만, 스킬 자체는 특정 상태 머신 전용이 아니라 parameter mutation 으로 missing scenario 를 찾는 범용 리뷰 스킬입니다.

## Constraints

- 구현, 스캐폴딩, 코드 수정은 하지 않습니다.
- unit 테스트 개수나 파일 수로 coverage 를 판단하지 않습니다.
- 리뷰 결과를 자동으로 spec/contract SSOT 에 반영하지 않습니다.
- 특정 버그 클래스에 맞춘 체크리스트로 축소하지 않습니다.
