# Verification Checklist

Before marking work complete, confirm all of the following:

- A failing test exists for each new behavior or bugfix.
- Each new test was executed and observed failing before implementation.
- The failure reason matched the missing behavior, not setup noise.
- Production code was written only after red.
- The implementation was the minimum needed to reach green.
- The targeted test passed after the implementation.
- The relevant broader suite was re-run after green.
- Refactors, if any, happened only after green and stayed green.
- Tests exercise behavior more than mocks.
- Edge cases and error paths were covered where they materially affect the change.
