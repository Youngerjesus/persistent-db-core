# Review Lenses

이 문서는 `decision-brake`의 긴 체크리스트가 아니라, 빠르게 방향에 브레이크를 거는 기본 질문과 심층 사고 프레임의 경계선을 정의합니다.
모든 질문을 다 쓰는 것이 목적이 아닙니다.
현재 결정의 품질을 가장 크게 바꿀 질문만 골라 사용합니다.

## Base Brake Questions

기본 `decision-brake`는 아래 질문을 우선 사용합니다.

### 1. Problem Legitimacy

- 이 결정은 진짜 문제를 푸는가, 아니면 표면 증상을 만지는가?
- 지금 결정을 내려야 하는 이유가 실제로 존재하는가?
- urgency 가 착시일 가능성은 없는가?

### 2. Logic and Evidence

- 결론이 전제보다 앞서가고 있지 않은가?
- 핵심 가정이 증명되지 않았는데 이미 사실처럼 취급되고 있지 않은가?
- 실제 증거와 추정이 구분되어 있는가?

### 3. Leaner Alternative

- 이 접근이 다른 접근보다 실제로 나은 이유가 있는가?
- 더 싸고 빠르고 reversible 한 대안이 있는데 무시하고 있지 않은가?
- 무언가를 더하는 대신 제거해서 같은 목적을 달성할 수 없는가?

### 4. Failure and Cost

- 실패하면 어디서 가장 먼저 무너지는가?
- 사용자, 운영, 일정, 품질, 신뢰 측면에서 가장 비싼 리스크는 무엇인가?
- 초기 구현보다 유지비가 더 큰 선택은 아닌가?

### 5. Commitment Level

- 이 방향을 지금 고정할 만큼 근거가 충분한가?
- 아직 불확실성이 높은데 too-early optimization 또는 too-early standardization 을 하고 있지 않은가?
- reversible 하게 나눠서 실행할 수는 없는가?

### 6. Scope Targeting

- 성공해도 의미 있는 진전, 학습, 증거, 사용자 가치를 만들 만큼 충분히 큰가?
- 너무 커서 실패 확률, 검증 지연, 조정 비용이 커지는 범위는 아닌가?
- 가장 작은 "이길 수 있는" 스코프는 무엇인가?

### 7. Execution Readiness

- 입력, 완료 기준, 검증 방법, owner 가 실행 가능한 수준으로 닫혀 있는가?
- 구현자가 새 제품/기술 결정을 내려야만 진행할 수 있는 빈칸이 남아 있지 않은가?
- missing evidence 나 human input 때문에 실행보다 clarification 이 먼저인가?

## When Thinker Depth Matters

아래 신호가 보이면 `decision-brake-thinker`를 통한 심층 사고를 고려합니다.

- 기본 브레이크만으로는 판단 근거가 얕다
- 문제 구조가 복합적이라 다른 프레임으로 다시 봐야 한다
- 2차 효과, 시스템 상호작용, 장기 비용이 중요하다
- 문제를 더 근본 단위로 다시 쪼개야 한다

## When Reviewer Depth Matters

아래 신호가 보이면 `decision-brake-reviewer`를 통한 독립 2차 검토를 고려합니다.

- 되돌리기 어려운 고비용 결정이다
- 현재 팀이나 제안자의 확신이 지나치게 강하다
- 메인 판단이나 thinker / explorer 산출물의 과신 위험이 있다
- 사용자가 생각의 외주나 독립된 시각을 명시적으로 원한다

## When Explorer Depth Matters

아래 신호가 보이면 `decision-brake-explorer`를 통한 발산 대안 탐색을 고려합니다.

- 현재안을 개선하는 것만으로는 사고가 좁아질 위험이 있다
- 사용자가 새로운 사고법, 완전히 다른 대안, 근본 원인 재탐색을 원한다
- 현재 문제 정의 자체가 틀렸을 가능성을 별도로 탐색해야 한다
- 현재안과 다른 leverage point, 제거 전략, 우회 전략을 비교해야 한다

## When Lens-Owner Depth Matters

아래 신호가 보이면 해당 렌즈 전용 에이전트를 고려합니다.

- 스코프가 너무 작아 무가치하거나 너무 커서 실패할 위험이 있으면 `decision-brake-scope-targeter`
- "무엇을 할지"는 좋아 보이지만 지금 바로 handoff 가능한지 애매하면 `decision-brake-readiness-reviewer`
- CAO candidate review 에서 handoff impact, missing input, evidence gap 이 verdict 를 바꿀 수 있으면 `decision-brake-readiness-reviewer`

## Thinker Deepening Angles

아래 사고법은 주로 `decision-brake-thinker`가 심층 사고에 사용합니다.

### First Principles

- 지금 제안은 가정 위에 올라가 있는가, 아니면 더 근본 단위까지 내려갔는가?
- 익숙한 관습을 제거하면 무엇이 남는가?

### Systems Thinking

- 이 선택이 다른 시스템 요소와 어떻게 상호작용하는가?
- 국소 최적화가 전체 최적화를 해치고 있지 않은가?
- 어떤 피드백 루프나 전이효과가 생기는가?

### Second-Order Thinking

- 당장의 이득 뒤에 오는 다음 비용은 무엇인가?
- 성공했을 때조차 운영 비용이나 복잡도가 커지는가?

### Inversion

- 이 결정을 통해 실패하려면 어떤 일이 벌어져야 하는가?
- 스스로 함정에 빠지게 만드는 선택은 무엇인가?

## Reviewer Focus

`decision-brake-reviewer`는 thinker 처럼 사고를 확장하는 역할이 아니라, 아래를 검토하는 데 집중합니다.

- 메인 판단의 논리적 비약
- thinker / explorer 산출물의 과장, 과신, 근거 부족
- 여전히 남아 있는 더 단순한 대안
- 제거하거나 보류해야 할 요소
- 지금 결정을 고정하는 것의 비용

## Explorer Focus

`decision-brake-explorer`는 현재안을 보완하는 역할이 아니라, 아래를 탐색하는 데 집중합니다.

- 현재안과 다른 문제 정의
- 대안적 root-cause 가설
- 비자명한 해결 전략
- 다른 leverage point
- 현재안 대비 얻는 것과 잃는 것
- 메인 판단이 종합해야 할 가장 강한 divergent path

## Scope Targeter Focus

`decision-brake-scope-targeter`는 현재안의 품질보다 스코프 조준을 검토합니다.

- too small: 성공해도 무가치하거나 증거가 약한 범위
- too large: 실패 확률, 조정 비용, 검증 지연이 큰 범위
- winnable target: 가장 작은 의미 있는 승리 지점
- scope tradeoff: 줄이면 잃는 것과 키우면 감당해야 할 것

## Readiness Reviewer Focus

`decision-brake-readiness-reviewer`는 방향의 매력보다 실행 준비도를 검토합니다.

- required inputs, owner, acceptance criteria, verification method
- unresolved decisions that would leak to the implementer
- missing evidence, protected area, human input, handoff blockers
- handoff impact: ready, changes-needed, clarification-needed, escalation-needed

## Suggested Review Moves

- 가장 치명적인 가정 1개를 먼저 겨냥합니다.
- "무엇을 더할까?" 대신 "무엇을 제거하면 같은 효과가 나나?"를 먼저 묻습니다.
- 현재안, 더 lean 한 안, 더 flexible 한 안을 나란히 둡니다.
- 스코프가 핵심이면 too small / too large / winnable target 을 함께 봅니다.
- 실행 준비도가 핵심이면 handoff 가능한지와 구현자에게 새 결정이 새는지 먼저 봅니다.
- 구조가 복합적이면 thinker 를 붙입니다.
- 현재안 밖의 근본 대안이 필요하면 explorer 를 붙입니다.
- 스코프 조준이 verdict 를 바꿀 수 있으면 scope-targeter 를 붙입니다.
- 실행 가능성이나 handoff impact 가 verdict 를 바꿀 수 있으면 readiness-reviewer 를 붙입니다.
- 되돌리기 어렵거나 편향 위험이 크면 reviewer 를 붙입니다.
- 사용자가 "심층 리뷰", "생각의 외주", "깊게 생각해봐", "다른 시각까지 검토해줘" 같은 표현으로 명시적으로 요청하면 thinker 와 reviewer 를 모두 붙입니다.
- 사용자가 "다른 대안", "새로운 사고법", "근본 원인", "현재안 밖의 선택지" 같은 표현으로 명시적으로 요청하면 explorer 를 붙입니다.
- 마지막에는 반드시 명확한 brake level 을 냅니다.
