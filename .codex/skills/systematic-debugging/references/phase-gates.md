# Phase Gates

Use these gates to decide whether you are ready to move forward or must return to investigation.

## Phase 1: Root Cause Investigation

Required before moving on:
- You can reproduce the issue or explain why it is currently non-deterministic
- You captured the exact symptom and expected behavior
- You checked the most relevant recent code, config, environment, or dependency changes
- You gathered evidence at the failing boundary or nearest observable boundary
- You can state the most likely failing component, assumption, or interaction

Do not advance if:
- You only know where it failed, not why
- You are still guessing between multiple plausible causes
- You have no hard evidence yet

## Phase 2: Pattern Analysis

Required before moving on:
- You found a working example, reference path, or expected baseline when one exists
- You compared broken and working cases across code, config, inputs, and execution path
- You listed the differences that could plausibly explain the symptom
- You checked whether the issue is tied to code, tests, environment, data, contracts, or external systems

Do not advance if:
- You are assuming a pattern without reading the working path fully
- You are ignoring a meaningful difference because it seems unlikely

## Phase 3: Hypothesis and Testing

Required before moving on:
- You stated one hypothesis in plain language
- You defined one experiment that can falsify that hypothesis
- You defined the expected observation before running it
- The experiment result narrowed the cause rather than broadening the uncertainty

Do not advance if:
- You ran multiple experiments at once
- You changed production code before proving what you were testing
- The experiment result is ambiguous and you have not re-framed the hypothesis

## Phase 4: Implementation

Required before calling the work done:
- You created a failing reproduction at the right layer for the issue
- The reproduction passed after the fix
- Relevant broader verification ran cleanly
- The fix addresses the evidenced cause, not only the visible symptom
- Temporary debugging instrumentation was removed or intentionally promoted to durable observability

Do not advance if:
- You cannot show red then green, or equivalent before/after proof
- The broader checks now fail
- You are bundling unrelated cleanup with the fix

## Architecture Brake

Stop and question the architecture when:
- Each attempted fix reveals a new hidden dependency or shared-state issue
- Fixes keep moving across layers without stabilizing the system
- The smallest plausible fix still requires wide refactoring
- The same class of issue keeps reappearing after local fixes

At that point, do not continue patching. Reframe the problem as a design issue.
