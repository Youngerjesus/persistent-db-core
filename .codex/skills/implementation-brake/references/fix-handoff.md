# Fix Handoff to `tdd-workflow`

이 문서는 implementation review finding 을 실제 수정 작업으로 넘길 때의 고정 규칙입니다.

## Principle

review 는 끝이 아니라 입력입니다.
must-fix finding 은 가능한 한 빨리 `tdd-workflow`의 다음 failing test 로 환원해야 합니다.

## Handoff Steps

1. finding 하나를 고릅니다.
2. 그 finding 을 observable behavior 한 문장으로 다시 씁니다.
3. 그 behavior 를 깨뜨리는 가장 작은 failing test 를 먼저 작성합니다.
4. red 를 실제로 확인합니다.
5. 그 테스트를 통과시키는 최소 production change 만 적용합니다.
6. targeted test 를 다시 실행합니다.
7. 관련 상위 테스트 스위트까지 넓혀 회귀를 확인합니다.
8. 원래 finding 이 실제로 닫혔는지 다시 review posture 로 확인합니다.

## Translation Pattern

finding:
"빈 입력에서 예외 대신 잘못된 기본값이 반환될 수 있다"

behavior target:
"빈 입력일 때 함수는 기본값을 삼키지 않고 명시적 실패를 반환해야 한다"

next TDD step:
"빈 입력 케이스를 재현하는 failing test 를 먼저 추가한다"

## Non-Negotiables

- failing test 없이 직접 production code 를 고치지 않습니다.
- 여러 finding 을 한 번에 묶어 대규모 수정으로 넘기지 않습니다.
- refactor 는 green 이후에만 합니다.
- broader verification 없이 finding closed 를 선언하지 않습니다.
