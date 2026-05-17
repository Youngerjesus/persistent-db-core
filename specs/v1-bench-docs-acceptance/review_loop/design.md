# V1 benchmark lower bounds 및 acceptance docs 고정

## 정규화된 후보
- Rank: 1
- Feature slug: v1-bench-docs-acceptance
- Target boundary: managed_repo
- Selection type: evidence_backed_feature_improvement
- Confidence: high

## 문제 정의
V1의 핵심 구현, 복구, crash, invariant, differential evidence는 대부분 충족되었지만, benchmark lower bounds와 launch gate별 acceptance documentation이 없어 최종 artifact completion 판단이 열려 있습니다.

## 지금 해야 하는 이유
Root Progress Projection에서 `gate-v1-bench-docs-acceptance`만 open이고, Queue Snapshot은 비어 있으며, current plan도 같은 gap을 다음 V1 acceptance evidence 후보로 지정합니다.

## 기대 산출물 변화
managed repo에 runnable benchmark verification surface, benchmark lower-bound 문서, V1 acceptance guide를 추가하고, 해당 문서가 모든 launch gate와 evidence requirement를 현재 증거에 매핑하도록 만듭니다.

## 의도한 변경 대상
- scripts/verify_bench_acceptance
- docs/benchmarks.md
- docs/v1_acceptance.md
- docs/v1_spec.md
- docs/cli_contract.md
- tests/

## Risk flags
- benchmark_environment_variability
- docs_must_map_only_verified_evidence
- avoid_unproven_performance_claims

## 근거
- Root Progress Projection: `artifact_status=open`, `gate-v1-bench-docs-acceptance` status=`open`, missing_requirement_ids=`req-v1-benchmark-lower-bounds`, `req-v1-acceptance-docs`.
- Current Artifact: `req-v1-benchmark-lower-bounds`는 benchmark output, benchmark command, lower-bound documentation을 요구합니다.
- Current Artifact: `req-v1-acceptance-docs`는 V1 usage와 acceptance docs가 각 launch gate를 evidence에 매핑해야 한다고 정의합니다.
- Current Plan: `gap-v1-bench-docs-acceptance`는 `metric-v1-acceptance-evidence`와 `gate-v1-bench-docs-acceptance`에 연결되어 있으며 next candidate hint가 benchmark harness, lower-bound docs, acceptance guide입니다.
- Active Managed Repo Snapshot: `git_status=clean`, Queue Snapshot=`[]`.
- Managed repo progress: 다음 작은 handoff 후보로 benchmark/acceptance docs를 언급하며, 기존 SQL, recovery, check, differential baseline 위에서 진행 가능하다고 기록합니다.
