# 001 - Continuous Pomodoro with Breaks

**Status:** done
**Date:** 2026-04-03

## Context

The pomodoro command currently runs a single 25-minute session on a selected task and exits. Users want a continuous flow: pomodoro → break → pick next task → pomodoro → break → ... until they decide to stop with Ctrl+C.

## Decision

Turn `run_pomodoro()` into a loop with three phases: work, break, task selection. Changes span `pomodoro.rs` (add break timer) and `main.rs` (loop + task selection with "create new" option).

## Implementation

- [ ] 1. `pomodoro.rs`: Add `BREAK_DURATION` (5 min), rename `run()` to `run_timer()` taking duration + label + description, add `run()` and `run_break()` as thin wrappers
- [ ] 2. `pomodoro.rs`: Update tests for new constants/functions
- [ ] 3. `main.rs`: Extract task selection into `select_task()` that returns `(usize, Vec<Todo>)` — reusable between first pick and subsequent picks
- [ ] 4. `main.rs`: Add `select_or_create_task()` that shows open tasks + a "Create new task" option; if user picks "Create new", prompt for description and project, add to store, re-fetch open todos, return the new task's index
- [ ] 5. `main.rs`: Refactor `run_pomodoro()` into a loop: first iteration uses `select_task()`, subsequent iterations use `select_or_create_task()` after a break. Ctrl+C during pomodoro records as cancelled and exits. Ctrl+C during break exits immediately. Completed pomodoro records + runs hook, then starts break.
- [ ] 6. Update `docs/SPECIFICATIONS.md` with new behavior
- [ ] 7. Run `cargo test`, `cargo clippy`, `cargo fmt`

## Consequences

- Pomodoro sessions become continuous until user explicitly quits
- Users can create tasks inline without leaving the pomodoro flow
- Break timer reuses the same countdown display with a different emoji/label
- Ctrl+C always exits the entire session (no partial cancel of just a break)
