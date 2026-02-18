# Ambrogio Specifications

This document describes the current implementation of Ambrogio.

## Overview

Ambrogio is a CLI tool with subcommands for managing todos, running pomodoro focus sessions, and chatting with a daily organiser via an LLM. Running without arguments starts the REPL chat interface.

## CLI Commands

```
ambrogio                        → REPL chat (default, requires LLM env vars)
ambrogio todos add 'buy milk'   → Append a todo to todos.md
ambrogio todos list              → Print open todos
ambrogio todos complete          → Interactive selection, mark as done
ambrogio pomodoro start          → Interactive todo selection, 25-min countdown
```

The `todos` and `start` subcommands only require `AMBROGIO_DAILY_ORGANISER_FILE` (via `FileConfig`). The REPL requires the full LLM configuration (via `Config`).

## Architecture

```
┌─────────────────────────────────────────────────────┐
│                    main.rs                          │
│           (CLI dispatch + REPL loop)                │
├──────────┬──────────┬──────────┬────────────────────┤
│  cli.rs  │ todo.rs  │pomodoro.rs│    chat.rs        │
│  (clap)  │ (store)  │ (timer)  │ (conversation)     │
├──────────┴──────────┴──────────┴────────────────────┤
│                   config.rs                         │
│         (Config + FileConfig from env)              │
├─────────────────────────────────────────────────────┤
│                    llm.rs                           │
│            (OpenAI-compatible API client)           │
└─────────────────────────────────────────────────────┘
```

## Modules

### `cli.rs`

Clap derive structs for CLI parsing.

**Types:**

- `Cli`: top-level parser with optional `Command`
- `Command`: `Todos { action }` or `Pomodoro { action }`
- `TodoAction`: `Add { description }`, `List`, `Complete`
- `PomodoroAction`: `Start`

No args (`None`) falls through to the REPL.

### `config.rs`

Handles configuration via environment variables.

**Environment Variables:**

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `AMBROGIO_LLM_API_KEY` | Yes (REPL only) | - | API key for the LLM provider |
| `AMBROGIO_LLM_URL` | Yes (REPL only) | - | Base URL of the OpenAI-compatible API |
| `AMBROGIO_LLM_MODEL` | Yes (REPL only) | - | Model name to use |
| `AMBROGIO_DAILY_ORGANISER_FILE` | Yes | - | Path to organiser file |
| `AMBROGIO_LLM_TIMEOUT` | No | `10` | Request timeout in seconds |

**Types:**

- `Config`: full LLM configuration (api_key, base_url, model, file_path, timeout) — used by REPL
- `FileConfig`: lightweight config with just `todos_path` — used by `todos` and `start` subcommands. Derives `todos_path` from the parent directory of `AMBROGIO_DAILY_ORGANISER_FILE`.

**Example Configurations:**

| Provider | URL | Model |
|----------|-----|-------|
| Groq | `https://api.groq.com/openai/v1` | `llama-3.3-70b-versatile` |
| OpenRouter | `https://openrouter.ai/api/v1` | `meta-llama/llama-3.3-70b-instruct` |
| OpenAI | `https://api.openai.com/v1` | `gpt-4o` |
| Ollama | `http://localhost:11434/v1` | `llama3` |

### `todo.rs`

File-backed todo store using markdown checkboxes.

**Types:**

- `Todo`: `{ description: String, done: bool }`
- `TodoStore`: wraps a `PathBuf`, provides `add()`, `load_all()`, `open_todos()`, `complete(index)`, `add_pomodoro(open_index, started_at, cancelled)`

**File Format (`todos.md`):**

```markdown
- [ ] open task
- [x] completed task
```

**Behavior:**

- `add()` creates the file if missing, appends `- [ ] description`
- `load_all()` parses all `- [ ] ` and `- [x] ` lines, ignores everything else
- `open_todos()` returns only unchecked items
- `complete(index)` rewrites the file, changing the nth open todo's `[ ]` to `[x]`
- `add_pomodoro(open_index, started_at, cancelled)` inserts a pomodoro entry under the nth open todo, after any existing pomodoro sub-items
- `print_open_todos()` prints numbered list of open todos

### `pomodoro.rs`

Countdown timer for focus sessions.

**Constants:**

- `POMODORO_DURATION`: 25 minutes

**Types:**

- `Outcome`: enum with `Completed` and `Cancelled` variants

**Functions:**

- `run(description)`: starts a 25-minute countdown, updating the terminal every second with `MM:SS - description`. Plays terminal bell (`\x07`) on completion. Ctrl+C cancels. Returns `Outcome::Completed` or `Outcome::Cancelled`.
- `format_countdown(duration)`: formats a `Duration` as `MM:SS`

### `llm.rs`

HTTP client for OpenAI-compatible chat completion APIs.

**Types:**

- `Message`: role + content (system/user/assistant)
- `LlmClient`: makes API requests

**API Format:**

- Endpoint: `{base_url}/chat/completions`
- Auth: Bearer token via `Authorization` header
- Request: `{ model, messages }`
- Response: `{ choices: [{ message: { content } }] }`

### `chat.rs`

Manages conversation state and system prompt.

**Types:**

- `ChatManager`: holds LLM client, system prompt, and message history

**System Prompt:**

Includes:
- Role description (personal assistant)
- Organiser format explanation
- Current date (for relative date calculations)
- Full organiser content

**Behavior:**

- Maintains conversation history for multi-turn chat
- Prepends system prompt to every request
- Appends user and assistant messages to history

### `main.rs`

Entry point with CLI dispatch.

**Flow:**

1. Parse CLI args with clap
2. No subcommand → `run_repl()` (loads full `Config`, reads organiser, starts REPL)
3. `todos` subcommand → `run_todos()` (loads `FileConfig`, operates on `TodoStore`)
4. `pomodoro start` → `run_pomodoro()` (loads `FileConfig`, selects todo, runs countdown, records pomodoro to `todos.md`)

**REPL Commands:**

- `quit`, `exit`, `q`: exit the program
- Ctrl+C, Ctrl+D: exit the program
- Any other input: send to LLM

## Organiser File Format

Expected markdown structure:

```markdown
# YYYY-MM-DD
**HH:MM** description
**HH:MM** task description [TODO]
**HH:MM** completed task [DONE]

# YYYY-MM-DD
...
```

**Conventions:**

- Dates as H1 headers: `# 2026-01-23`
- Time-based entries: `**HH:MM** description`
- Open tasks: `[TODO]` suffix
- Completed tasks: `[DONE]` suffix
- Free-form notes allowed between entries

## Todo File Format

Located in the same directory as the organiser file, named `todos.md`.

```markdown
- [ ] open task
  - 🍅 2026-02-12 10:00
  - 🍅 2026-02-12 14:30 cancelled
- [x] completed task
```

**Pomodoro entries** are indented sub-items under their todo. Format: `  - 🍅 YYYY-MM-DD HH:MM [cancelled]`. Absence of `cancelled` means the pomodoro ran to completion. Pomodoro lines are ignored by `load_all()` and `open_todos()`.

## Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| tokio | 1 | Async runtime |
| reqwest | 0.11 | HTTP client |
| serde | 1 | Serialization |
| serde_json | 1 | JSON handling |
| rustyline | 14 | REPL with history |
| anyhow | 1 | Error handling |
| chrono | 0.4 | Date/time for system prompt |
| clap | 4 | CLI subcommand parsing |

**Dev Dependencies:**

| Crate | Version | Purpose |
|-------|---------|---------|
| tempfile | 3 | Temp files for tests |

## Limitations

- No streaming responses (waits for full response)
- No persistent chat history across sessions
- Organiser file is loaded once at startup (changes require restart)
- No syntax validation of organiser file format
