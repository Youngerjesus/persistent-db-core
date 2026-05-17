# Debug Report Template

Use this template while investigating and before proposing a fix.

```text
Symptom:
- What failed, where, and how it surfaced

Expected:
- What should have happened instead

Reproduced:
- Yes / No / Partial
- Exact reproduction method or why reproduction is currently unreliable

Evidence:
- Error messages, logs, traces, diffs, screenshots, or working-vs-broken comparisons

Current hypothesis:
- One explicit candidate root cause

Likely category:
- implementation defect
- spec mismatch
- test defect
- environment or config drift
- observability gap
- data issue
- external dependency issue
- unknown

Next step:
- One experiment, check, or verification action

Fix gate:
- Blocked until root cause is evidenced
- Open only after evidence clearly favors the current hypothesis
```

## Notes

- Do not list multiple hypotheses unless the evidence truly leaves you with more than one viable explanation.
- If more than one remains viable, keep the list short and say what evidence would eliminate each one.
- `Likely category` is a working classification, not a final verdict.
- If the issue is connected to a spec in this repository, include the relevant spec or contract reference in `Evidence`.
