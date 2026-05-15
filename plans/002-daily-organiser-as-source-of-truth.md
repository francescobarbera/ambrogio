# 002 - Daily organiser as source of truth

**Status:** accepted
**Date:** 2026-05-15

## Context

Tasks currently live in a separate `todos.md` file (sibling of `AMBROGIO_DAILY_ORGANISER_FILE`), grouped under `## Project` headers. The user has shifted their workflow: every morning they manually copy the previous day's unfinished tasks into today's `# YYYY-MM-DD` section of the organiser, under a `## Todos:` block. The deliberate manual step is the planning ritual — it forces re-evaluation. With that workflow, the separate `todos.md` and the project concept no longer earn their keep: there is only ever one list (today's), and the grouping that matters is the date.

## Decision

**Make the daily organiser file the single source of truth for tasks, scoped to today's `# YYYY-MM-DD` / `## Todos:` block.**

`## Logs:` is never read or written. Drop the `projects`, `note`, and `tasks delete` commands — they no longer fit the model. The detailed command surface and file-format rules live in `docs/SPECIFICATIONS.md`; this ADR records *why* and the structural choices.

Structural choices:

- **Free functions over a store struct.** Replace `TodoStore` with free functions in `src/daily.rs` taking `(path, today: NaiveDate, ...)`. Deterministic tests for free; no clock hidden in a constructor.
- **`FileConfig` removed.** With `todos_path` gone it collapses to a one-field wrapper; replace with `organiser_path_from_env() -> Result<PathBuf>`.
- **Internal seam: pure parse/edit helpers, thin I/O wrappers.** Each public function reads, delegates the markdown transform to a pure helper, then writes — mirrors the existing `find_open_todo_line` / `write_lines` pattern in `todo.rs`. Keeps tests fast and the byte-exactness guarantees verifiable without a tempfile.
- **Today-section creation only by `tasks add`.** `list`/`complete`/`pomodoro` return "No tasks for today." when the section is missing, preserving the planning-ritual feedback signal. `add` scaffolds **only** `# YYYY-MM-DD\n## Todos:\n- [ ] foo\n` — the `## Logs:` header is the user's responsibility, the parser never requires it. New day sections are inserted at the top of the file.
- **Dual-form parsing is permanent.** Both `- [ ]` and dashless `[ ]` lines are read; modifications preserve the form and trailing whitespace of the line being touched. The pomodoro icon goes *before* any trailing whitespace (`[ ] task 🍅  `). `tasks add` writes the dashed form.
- **Pomodoro spacing rule.** First icon: ` 🍅` (one leading space) before any trailing whitespace. Subsequent icons: concatenate without a space. Detection: scan from end-of-text-content for an existing trailing `🍅` before trailing whitespace.
- **Day-section boundary is generic.** Parser stops at the next `##` header, next `# ` header, or EOF — `## Logs:` is not special-cased.
- **Cancelled pomodoro: do nothing.** `add_pomodoro` is never called on cancellation; this is a wiring property of `run_pomodoro` in `main.rs`, not a `daily.rs` behavior — so it's covered by code review, not by a brittle integration test.

## Alternatives considered

- **Keep `todos.md` and sync from organiser** — second source of truth, sync direction problem. Rejected.
- **"Today" = latest `# YYYY-MM-DD` header** — easier when the user forgets to plan, but defeats the planning ritual. Rejected.
- **Auto-create today's section on any read command** — same problem. Only `tasks add` creates it.
- **Normalise dashless lines on first write** — silently rewriting the user's file is more surprising than carrying a tiny dual-format branch. Rejected per user preference.
- **Keep `DailyTasks` struct for symmetry with `TodoStore`** — adds a clock dependency for no test or callsite benefit.
- **Strikethrough `~~...~~` for completion** — `[x]` is idiomatic markdown. Rejected.
- **Sub-bullet pomodoro lines with timestamps** — user explicitly chose accumulating icons; daily granularity is enough.
- **Integration-test the cancelled-no-write invariant at `run_pomodoro` level** — would require an injection seam for the timer/stdin/hooks. The invariant is "`add_pomodoro` is never called for cancellation"; code review is the right check, not a test. Rejected.
- **YAML frontmatter per global ADR template** — `plans/001-continuous-pomodoro-with-breaks.md` already uses the bold `**Status:**` / `**Date:**` style; staying consistent within the project wins over matching the global template verbatim.

## Implementation

Two commits, tidy-first. The first does the pure structural refactor that the behavior change depends on; the second delivers the new behavior. This avoids the rename-then-delete churn an alternative ordering would produce.

**[tidy] `refactor: collapse FileConfig to organiser_path_from_env free function`**

- [ ] 1. `src/config.rs`: replace `FileConfig` with `pub fn organiser_path_from_env() -> Result<PathBuf>`. Update the existing test to assert the returned path equals the env var verbatim. Update the two callers in `main.rs` (`run_tasks`, `run_pomodoro`) — at this point they still pass it to `TodoStore` via the old `todos_path` derivation, so reintroduce the parent-dir join inline at the callsite as a one-line bridge. No observable behavior change.
- [ ] 2. `cargo fmt && cargo clippy --all-targets -- -D warnings && cargo test`.

**[behavior] `feat: scope tasks to today's organiser section; drop projects/note/tasks-delete`**

- [ ] 3. Add `src/daily.rs`: free functions `today_open`, `add`, `complete`, `add_pomodoro` — each takes `path: &Path, today: NaiveDate, ...`. `///` doc-comment each. Internal helpers split pure (parse/edit `&str` → `String`) from I/O. Unit tests under `#[cfg(test)]` in the same file (see Test matrix).
- [ ] 4. `src/cli.rs`: delete `Projects` variant, `ProjectAction`, `Note` variant, `TaskAction::Delete`, and their aliases. Remove the now-obsolete positive parse tests for those variants (they'd fail to compile anyway) and add negative parse tests for the long forms **and** the aliases (`p list`, `n 'x'`, `t d`).
- [ ] 5. `src/main.rs`: rewrite `run_tasks` and `run_pomodoro` against `daily.rs` using `organiser_path_from_env()` directly (no `todos.md` bridge). Delete `run_projects`, `run_note`, `select_or_create_task`, and the `Delete` arm. `select_task` returns `Result<Option<(usize, String)>>` so the pomodoro loop's "no more tasks" path is an exhaustive `match` arm. Pomodoro loop steps each iteration: (a) work timer; (b) on completion: append `🍅` via `add_pomodoro`, then run `hooks::run("pomodoro", "stop")`; (c) break timer; (d) on break completion: run `hooks::run("break", "stop")`; (e) re-prompt today's open list. Any cancellation exits the loop; an empty open list at step (e) also exits cleanly.
- [ ] 6. Delete `src/todo.rs` and its `mod todo;` declaration (now unused after step 5).
- [ ] 7. Update `docs/SPECIFICATIONS.md`: command table, file format rules, today-scoped semantics. Update `README.md` examples. Update `plans/001-continuous-pomodoro-with-breaks.md` with a note that step 4's `select_or_create_task` branch is superseded by ADR 002.
- [ ] 8. `cargo fmt && cargo clippy --all-targets -- -D warnings && cargo test`.

**Test matrix for `daily.rs` (step 1)**

Each test injects `today: NaiveDate` for determinism; assertions on file content are byte-exact.

| # | Function | Case | Expected |
|---|----------|------|----------|
| 1 | `today_open` | missing file | `Ok(vec![])` |
| 2 | `today_open` | file without today's `# YYYY-MM-DD` | `Ok(vec![])` |
| 3 | `today_open` | today exists but no `## Todos:` block | `Ok(vec![])` |
| 4 | `today_open` | only `[x]` tasks today | `Ok(vec![])` |
| 5 | `today_open` | mixed dashed and dashless open lines | both returned, in file order |
| 6 | `today_open` | task-shaped line inside `## Logs:` | excluded (parser stops at `##`) |
| 7 | `today_open` | multi-day file | only today's open tasks returned |
| 8 | `add` | missing file | creates `# YYYY-MM-DD\n## Todos:\n- [ ] foo\n` (exact bytes; no `## Logs:`) |
| 9 | `add` | today's `# YYYY-MM-DD` exists, no `## Todos:` | inserts `## Todos:\n- [ ] foo\n` immediately after the day header |
| 10 | `add` | today's `## Todos:` exists and is populated | appends `- [ ] foo` at end of Todos block, before next `##`/`# `/EOF |
| 11 | `add` | no today section, file has older days | inserts new day section at top of file |
| 12 | `add` | file ends without trailing newline | writes correctly without introducing extraneous blank lines |
| 13 | `complete` | dashed line | `- [ ]` → `- [x]`; no other bytes change |
| 14 | `complete` | dashless line with trailing spaces | `[ ] foo  ` → `[x] foo  ` |
| 15 | `complete` | out-of-bounds index | `Err` containing `"out of bounds"` |
| 16 | `complete` | no today section | `Err` containing `"no tasks for today"` (or equivalent — must be distinguishable from OOB) |
| 17 | `add_pomodoro` | first icon on dashed line | `- [ ] foo` → `- [ ] foo 🍅` |
| 18 | `add_pomodoro` | first icon on dashless line with trailing spaces | `[ ] foo  ` → `[ ] foo 🍅  ` |
| 19 | `add_pomodoro` | second icon on dashed line | `- [ ] foo 🍅` → `- [ ] foo 🍅🍅` (no extra space) |
| 20 | `add_pomodoro` | third icon on dashed line | `- [ ] foo 🍅🍅` → `- [ ] foo 🍅🍅🍅` |
| 21 | `add_pomodoro` | second icon on dashless line with trailing spaces | `[ ] foo 🍅  ` → `[ ] foo 🍅🍅  ` |
| 22 | `add_pomodoro` | out-of-bounds index | `Err` containing `"out of bounds"` |
| 23 | `add_pomodoro` | no today section | `Err` containing `"no tasks for today"` |
| 24 | `complete` / `add_pomodoro` | today exists but `## Todos:` block absent | both `Err` containing `"no tasks for today"` |
| 25 | parser | boundary: today-then-EOF | Todos block terminates at EOF |
| 26 | parser | boundary: today followed by `# 2026-05-14` | Todos block terminates at next `# ` |
| 27 | parser | boundary: today followed by `## SomeOther:` | Todos block terminates at next `##` |

Env-var tests for `organiser_path_from_env` are gated by a process-local `Mutex<()>` from the start (`static ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());`). `cargo test` parallelises by default and `std::env::set_var` is not thread-safe — the mutex is mandatory, not conditional.

## Consequences

- Project concept removed; `todo.rs` (~743 lines incl. tests) replaced by `daily.rs` (~200 lines target). Simpler mental model, smaller surface.
- Manual planning friction is preserved: forgetting to add today's section means `list`/`pomodoro` say "no tasks". Intended feedback signal.
- Old `todos.md` on disk is ignored; the user hand-copies any unfinished tasks once. No migration tooling.
- Existing dashless lines in `daily_organiser.md` keep working; modifications are minimum-diff. Dashless is supported indefinitely.
- Pomodoro lines lose timestamps. Daily granularity is enough; the Logs block carries narrative.
- The `(read → pick index → write)` interactive flow is racy against concurrent external edits — same as the previous `TodoStore`, accepted trade-off for a single-user CLI.
- Generic day-section parser: future `## SubSection`s within a day will terminate the Todos block automatically; no parser change needed.
- This ADR continues `plans/001`'s bold-fields format rather than the global YAML template. Within-project consistency over global-template parity.

## Links

- Supersedes step 4's `select_or_create_task` branch in `plans/001-continuous-pomodoro-with-breaks.md` (the create-new-task flow). The continuous-loop, break-timer, and `select_task` pieces survive — though `select_task`'s signature changes from `(usize, Vec<Todo>)` to `Result<Option<(usize, String)>>`.
