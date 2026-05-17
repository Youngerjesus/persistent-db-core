# Scenario Review Lenses

모든 렌즈를 다 쓰지 말고, 현재 계획의 scenario coverage 품질을 가장 많이 바꿀 렌즈만 선택합니다.

각 렌즈에서는 아래 공통 절차를 수행합니다.

1. baseline scenario 를 한 문장으로 적습니다.
2. 이 시나리오를 구성하는 주요 파라미터를 3-7개 식별합니다.
3. 각 파라미터의 가능한 값 또는 사건 변형을 적습니다.
4. 한 번에 하나 또는 필요한 경우 소수의 조합만 바꿔 변형 시나리오를 만듭니다.
5. 각 변형이 기존 시나리오와 같은 취급이 가능한지, 별도 시나리오인지, 선행/후속 경로인지 판정합니다.
6. 계획에 없는 고위험 변형은 missing scenario 로 기록합니다.

## 1. Entry Lens

- 최초 진입, 재시도, 재개, 재시작 후 재진입, 수동 개입이 모두 truly same path 인가?
- entry trigger 가 바뀌면 lookup, validation, initialization 이 달라지는가?
- 첫 실행 기준으로만 설계되어 재진입 경로가 빠져 있지 않은가?

## 2. Actor Lens

- 사용자, 스케줄러, 백그라운드 잡, 운영자, 동시 실행 주체가 같은 invariant 를 공유하는가?
- actor 가 바뀌면 권한, 타이밍, 재시도, 복구 방식이 달라지는가?
- 계획이 한 actor 기준 assumptions 를 다른 actor 에도 그대로 적용하고 있지 않은가?

## 3. State Lens

- initial, in-progress, partial success, failed, already-completed, stale 판정 상태가 truly same handling 인가?
- 같은 요청이라도 시작 상태가 다르면 다른 후속 경로가 생기는가?
- 상태 기록이 남은 뒤 다음 실행이 같은 lookup/transition 을 타도 안전한가?

## 4. Data Lens

- empty, malformed, duplicate, outdated, partially persisted, conflicting snapshot 이 다른 시나리오를 만드는가?
- 데이터 shape 가 바뀌면 처리 로직뿐 아니라 복구 경로도 달라지는가?
- "입력이 이상하다"를 한 시나리오로 뭉뚱그리면 안 되는가?

## 5. Dependency Lens

- dependency success, timeout, partial response, delayed consistency, no result 가 같은 실패로 묶이면 안 되는가?
- 외부 시스템 응답 차이가 state write, retry policy, fallback behavior 를 바꾸는가?
- dependency failure 를 한 종류의 error 로만 보고 downstream 차이를 놓치고 있지 않은가?

## 6. Recovery Lens

- retry, fallback, recreate, rollback, skip, terminal fail, no-op 가 truly interchangeable 한가?
- 한 번 실패한 뒤의 rescue path 가 다음 실행의 일반 경로와 충돌하는가?
- 복구 성공만 보고 복구 실패 후 후속 경로를 놓치고 있지 않은가?

## 7. Environment Lens

- process restart, cold start, config drift, feature flag difference, browser/device/context 차이가 같은 로직 경로를 유지하는가?
- 메모리 기반 가정이 사라지면 다른 시나리오가 생기는가?
- 운영 환경 변화가 stale reference, duplicate selection, replay risk 를 만드는가?

## 8. Invariant Lens

- idempotency, uniqueness, ownership, ordering, stale exclusion, monotonic progression 이 실제로 유지되는가?
- 같은 entity 가 다시 선택되거나 다시 처리되면 별도 시나리오가 되는가?
- 계획이 happy path invariants 만 적고 failure/re-entry invariants 는 비워 두고 있지 않은가?

## 9. Observability Lens

- 같은 현상이 로그, 메트릭, 상태에서는 다른 원인으로 구분 가능한가?
- 시나리오를 분리해야 하는데 관측 가능성 부족 때문에 한 문제처럼 보이진 않는가?
- 운영자가 후속 조치를 결정할 만큼 failure mode 가 드러나는가?

## Suggested Review Moves

- 가장 중요한 baseline scenario 하나를 먼저 잡고 파라미터를 뽑습니다.
- "엣지 케이스가 더 있나?" 대신 "이 시나리오를 성립시키는 변수는 무엇인가?"를 먼저 묻습니다.
- 파라미터를 바꿨을 때 새 경로가 생기면, 같은 문제인지 다른 문제인지 강제로 판정합니다.
- 분리된 시나리오마다 필요한 검증 수준을 같이 적습니다.
- 마지막에는 계획이 충분한지, 빠졌는지, 재구성이 필요한지 명확한 verdict 를 냅니다.
