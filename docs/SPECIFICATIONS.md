# Ambrogio Specifications

This document describes the current implementation of Ambrogio.

## Overview

Ambrogio is a CLI tool for managing today's tasks inside a markdown daily-organiser file, running pomodoro focus sessions, and chatting with the organiser via an LLM. Running without arguments starts the REPL chat interface.

Tasks are scoped to **today's** date section in the organiser file. The user manually copies unfinished tasks from the previous day each morning — that planning step is intentional and the tool does not automate it. If today's section is missing, `list`, `complete`, and `pomodoro` print "No tasks for today." rather than creating the section silently. Only `tasks add` creates the section.

## CLI Commands

```
ambrogio                             → REPL chat (default, requires LLM env vars)
ambrogio tasks add 'buy milk'        → Append a task to today's ## Todos: block
ambrogio tasks list                  → Print today's open tasks
ambrogio tasks complete              → Interactive selection, flip [ ] → [x]
ambrogio pomodoro start              → Interactive selection, 25-min countdown loop
```

**Aliases:**

| Command | Alias | Subcommand | Alias |
|---------|-------|------------|-------|
| `tasks` | `t` | `add` | `a` |
| `pomodoro` | `pom` | `list` | `l` |
| | | `complete` | `c` |
| | | `start` | `s` |

Examples: `ambrogio t l` = `ambrogio tasks list`, `ambrogio t a 'buy milk'` = `ambrogio tasks add 'buy milk'`.

The `tasks` and `pomodoro` subcommands require `AMBROGIO_DAILY_ORGANISER_FILE`. The REPL additionally requires the LLM configuration.

## Architecture

```
┌─────────────────────────────────────────────────────┐
│                    main.rs                          │
│           (CLI dispatch + REPL loop)                │
├──────────┬──────────┬───────────┬───────────────────┤
│  cli.rs  │ daily.rs │pomodoro.rs│    chat.rs        │
│  (clap)  │ (today)  │ (timer)   │ (conversation)    │
├──────────┴──────────┴───────────┴───────────────────┤
│              hooks.rs     │      config.rs          │
│         (event scripts)   │  (env configuration)    │
├───────────────────────────┴─────────────────────────┤
│                    llm.rs                           │
│            (OpenAI-compatible API client)           │
└─────────────────────────────────────────────────────┘
```

## Modules

### `cli.rs`

Clap derive structs for CLI parsing.

**Types:**

- `Cli`: top-level parser with optional `Command`
- `Command`: `Tasks { action }` or `Pomodoro { action }`
- `TaskAction`: `Add { description }`, `List`, `Complete`
- `PomodoroAction`: `Start`

No args (`None`) falls through to the REPL.

### `config.rs`

Handles configuration via environment variables.

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `AMBROGIO_LLM_API_KEY` | Yes (REPL only) | - | API key for the LLM provider |
| `AMBROGIO_LLM_URL` | Yes (REPL only) | - | Base URL of the OpenAI-compatible API |
| `AMBROGIO_LLM_MODEL` | Yes (REPL only) | - | Model name to use |
| `AMBROGIO_DAILY_ORGANISER_FILE` | Yes | - | Path to organiser file |
| `AMBROGIO_LLM_TIMEOUT` | No | `10` | Request timeout in seconds |

**Types and functions:**

- `Config`: full LLM configuration (api_key, base_url, model, file_path, timeout) — used by the REPL
- `organiser_path_from_env() -> Result<PathBuf>`: returns the organiser path; used by `tasks` and `pomodoro`

### `daily.rs`

File-backed task store scoped to today's `## Todos:` block inside the daily organiser. Free functions; no struct. The clock dependency is passed in as a `chrono::NaiveDate` parameter to keep tests deterministic.

**Public API:**

| Function | Behavior |
|----------|----------|
| `today_open(path, today)` | Returns the open `[ ]` task descriptions in today's Todos block, in file order. Both dashed (`- [ ]`) and dashless (`[ ]`) forms are read. Empty vec if file, today, or Todos block is missing. |
| `add(path, today, description)` | Appends `- [ ] description` to today's Todos block. Creates the file, today's day section, or the Todos block if missing. New day sections are inserted at the top of the file. |
| `complete(path, today, index)` | Flips `[ ]` → `[x]` on the `index`-th open task. Preserves line form and trailing whitespace. Errors `"no tasks for today"` or `"index N out of bounds"`. |
| `add_pomodoro(path, today, index)` | Appends `🍅` to the `index`-th open task before any trailing whitespace. If a `🍅` already trails (modulo whitespace), the new icon concatenates with no separator. |

**Parsing rules:**

- Today's section starts at the line `# YYYY-MM-DD` matching the passed `today` and ends at the next `# ` header or EOF.
- The `## Todos:` block starts at `## Todos:` inside today's day section and ends at the next `##`/`# ` header or EOF.
- `## Logs:` and other day sections are never read or modified.

### `pomodoro.rs`

Countdown timer for focus sessions.

**Constants:** `POMODORO_DURATION` = 25 min, `BREAK_DURATION` = 5 min.

**Types:** `Outcome { Completed, Cancelled }`.

**Functions:** `run(description)`, `run_break()`, `run_timer(duration, emoji, description)`, `format_countdown(duration)`. Plays a terminal bell on completion; Ctrl+C cancels.

### `llm.rs` / `chat.rs`

HTTP client for OpenAI-compatible chat completion APIs, plus conversation state. Unchanged from prior versions; see source for details.

### `main.rs`

Entry point with CLI dispatch.

1. Parse CLI args.
2. No subcommand → `run_repl()` (loads full `Config`, reads organiser, starts REPL).
3. `tasks` → `run_tasks()` (loads `organiser_path_from_env()`, dispatches add/list/complete on `daily`).
4. `pomodoro start` → `run_pomodoro()` (selects a task, runs the work/break loop, appends `🍅` on each completed pomodoro).

**Pomodoro loop iteration:**

1. Work timer (25 min).
2. If completed: `daily::add_pomodoro` then `hooks::run("pomodoro", "stop")`.
3. Break timer (5 min).
4. If completed: `hooks::run("break", "stop")`.
5. Re-prompt today's open list.

Any cancellation exits the loop. An empty open list at step 5 also exits cleanly with a message.

## Organiser File Format

```markdown
# 2026-05-15
## Todos:
- [ ] applicare le modifiche suggerite da Raffaele
- [ ] finire la configurazione di aws-infrastructure 🍅🍅
- [x] fixare i problemi di GeoTech 🍅🍅🍅
## Logs:
**08:30** [post](...) ...

# 2026-05-14
...
```

**Conventions:**

- Day headers: `# YYYY-MM-DD` (one hash).
- Today's tasks go under `## Todos:`. Other subsections (`## Logs:`, etc.) are ignored by the task tools.
- Tasks may be written as `- [ ] foo` (dashed, what `tasks add` writes) or `[ ] foo  ` (dashless, the user's manual style). Both are read; modifications preserve the line's original form and trailing whitespace.
- Pomodoros accumulate as `🍅` characters at the end of the task text, before any trailing whitespace.
- Logs entries use `**HH:MM** description` and are free-form; they are never read or written by the task tools.

## Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| tokio | 1 | Async runtime |
| reqwest | 0.13 | HTTP client (rustls) |
| serde | 1 | Serialization |
| serde_json | 1 | JSON handling |
| rustyline | 18 | REPL with history |
| anyhow | 1 | Error handling |
| chrono | 0.4 | Date / time |
| clap | 4 | CLI parsing |
| dirs | 6 | Platform config directory resolution |

**Dev Dependencies:** `tempfile = 3`.

## Hooks

Ambrogio supports user-defined shell scripts that run on specific events.

**Location:** `~/.config/ambrogio/hooks/{feature}/{event}.sh`.

**Behavior:** missing hook = silent no-op; existing hook runs via `sh`; non-zero exit prints a warning but does not interrupt the main flow; no env vars are passed.

| Hook path | Trigger |
|-----------|---------|
| `pomodoro/stop.sh` | After a pomodoro completes successfully (not on cancellation) |
| `break/stop.sh` | After a break completes successfully (not on cancellation) |

## Limitations

- No streaming responses (waits for full response).
- No persistent chat history across sessions.
- Organiser file is loaded once at startup for the REPL (changes require restart).
- No syntax validation of the organiser file format.
- The `(read → pick → write)` interactive flow is racy against concurrent external edits to the organiser; accepted trade-off for a single-user CLI.
