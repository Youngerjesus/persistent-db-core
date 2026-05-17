# Strict TDD Rules

## Core Principle

If you did not watch the test fail, you do not know whether it tests the intended behavior.

```
NO PRODUCTION CODE WITHOUT A FAILING TEST FIRST
```

This is not guidance. It is the gate.

## Red -> Green -> Refactor

### 1. Red

- Write one minimal test for one behavior.
- Use a specific test name that describes the outcome.
- Prefer real behavior over mocks unless external I/O makes that impossible.

Good:

```typescript
test("retries a transient failure three times", async () => {
  let attempts = 0;

  const operation = async () => {
    attempts += 1;
    if (attempts < 3) throw new Error("fail");
    return "success";
  };

  await expect(retryOperation(operation)).resolves.toBe("success");
  expect(attempts).toBe(3);
});
```

Bad:

```typescript
test("retry works", async () => {
  const operation = vi
    .fn()
    .mockRejectedValueOnce(new Error("fail"))
    .mockRejectedValueOnce(new Error("fail"))
    .mockResolvedValueOnce("success");

  await retryOperation(operation);
  expect(operation).toHaveBeenCalledTimes(3);
});
```

### 2. Verify Red

- Run the narrowest test command that proves the new test is active.
- Confirm the test fails, not errors.
- Confirm the failure reason matches the missing behavior, not a typo or setup issue.

If the test passes immediately, stop. You are testing the wrong thing or existing behavior. Change the test before touching production code.

### 3. Green

- Write the smallest amount of production code that makes the failing test pass.
- Do not generalize early.
- Do not add unrelated cleanup, abstractions, or optional flags.

### 4. Verify Green

- Re-run the targeted test.
- Run the relevant broader suite for the area you changed.
- Keep the output clean enough to trust.

If a broader test fails, fix the code now. Do not postpone it.

### 5. Refactor

- Refactor only after green.
- Remove duplication, improve names, or extract helpers without changing behavior.
- Re-run tests to prove the refactor preserved behavior.

## Rationalizations To Reject

- "This is too small to test."
- "I'll add the test after."
- "I already manually tested it."
- "The test already passes, so we're good."
- "I'll keep this implementation as reference."
- "Being pragmatic means skipping the ritual."

All of these weaken the proof chain. TDD without observed red is not TDD.

## Preferred Test Shape

- One behavior per test
- Clear, behavior-focused names
- Real inputs and outputs
- Edge cases expressed as separate tests
- Mocks only at genuine system boundaries

## Bugfix Pattern

1. Reproduce the bug with a failing test.
2. Watch it fail for the bug you intend to fix.
3. Implement the minimal fix.
4. Re-run the regression test and the relevant surrounding suite.

Never fix a bug first and add the regression test later.
