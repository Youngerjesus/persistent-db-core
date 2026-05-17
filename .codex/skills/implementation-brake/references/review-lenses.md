# Implementation Review Lenses

모든 렌즈를 다 쓰지 말고, 현재 구현의 pass/fail 을 가장 많이 바꿀 렌즈만 선택합니다.

## 1. Correctness

- 의도한 동작을 실제로 만족하는가?
- happy path 는 맞더라도 branch 하나가 틀어질 가능성은 없는가?
- state transition, return value, side effect 중 하나라도 어긋나는가?

## 2. Regression Risk

- 이번 수정이 기존 동작을 깨뜨릴 가능성이 어디에 있는가?
- 변경 경계가 생각보다 넓은데 검증 범위가 너무 좁지는 않은가?
- 기존 테스트가 이번 회귀를 실제로 잡을 수 있는가?

## 3. Edge Cases and Failure Paths

- nil, empty, error, timeout, invalid input, concurrent path 가 빠져 있지 않은가?
- 실패 시 반환/로그/상태가 일관적인가?
- 호출자는 이 실패를 안전하게 다룰 수 있는가?

## 4. Test Gap

- 핵심 동작을 증명하는 테스트가 없는가?
- 테스트가 너무 implementation-detail 에 묶여 있는가?
- failure-path, regression-path, boundary-case coverage 가 빠져 있지 않은가?

## 5. Rigidity

- 지금 구현이 필요 이상으로 경직되어 미래 선택지를 닫는가?
- 너무 이른 추상화, 너무 이른 일반화가 들어갔는가?
- 특정 케이스에만 맞는 구조를 공용 경로에 박아 넣었는가?

## 6. Addition Bias in Code

- 같은 목적을 더 적은 branch, 더 적은 abstraction, 더 적은 moving part 로 달성할 수 없는가?
- 새 타입, helper, layer, config 가 실제 가치보다 많지 않은가?
- "나중에 필요할 것 같아서" 추가한 구조가 있는가?

## 7. Maintainability

- 읽는 사람이 쉽게 오해할 수 있는 흐름인가?
- 함수 책임이 과하고 경계가 흐려졌는가?
- 수정 포인트가 여러 군데로 퍼져 있어 다음 변경 비용이 커졌는가?

## 8. Performance and Security

- 반복 호출, 불필요한 query, payload inflation, retry amplification 이 생기지 않는가?
- 입력 검증, 권한 경계, 민감 데이터 처리가 느슨하지 않은가?

## Suggested Review Moves

- defect risk 가 가장 큰 finding 부터 적습니다.
- "이상하다"가 아니라 "어디서 왜 깨질 수 있는지"를 적습니다.
- accepted finding 은 즉시 하나의 failing test 후보로 바꿉니다.
- defer 가능한 항목과 ship blocker 를 섞지 않습니다.
