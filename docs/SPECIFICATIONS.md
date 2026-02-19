# Ambrogio Specifications

This document describes the current implementation of Ambrogio.

## Overview

Ambrogio is a CLI tool with subcommands for managing todos, projects, running pomodoro focus sessions, and chatting with a daily organiser via an LLM. Running without arguments starts the REPL chat interface.

## CLI Commands

```
ambrogio                            → REPL chat (default, requires LLM env vars)
ambrogio projects list               → List all projects
ambrogio projects add 'Work'         → Create a new project
ambrogio projects delete             → Interactive project deletion with confirmation
ambrogio todos add 'buy milk'        → Add a todo (prompts for project selection)
ambrogio todos list                  → Print open todos grouped by project
ambrogio todos complete              → Interactive selection, mark as done
ambrogio pomodoro start              → Interactive todo selection, 25-min countdown
```

The `todos`, `projects`, and `pomodoro` subcommands only require `AMBROGIO_DAILY_ORGANISER_FILE` (via `FileConfig`). The REPL requires the full LLM configuration (via `Config`).

## Architecture

```
┌─────────────────────────────────────────────────────┐
│                    main.rs                          │
│           (CLI dispatch + REPL loop)                │
├──────────┬──────────┬───────────┬───────────────────┤
│  cli.rs  │ todo.rs  │pomodoro.rs│    chat.rs        │
│  (clap)  │ (store)  │ (timer)   │ (conversation)    │
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
- `Command`: `Todos { action }`, `Projects { action }`, or `Pomodoro { action }`
- `TodoAction`: `Add { description }`, `List`, `Complete`
- `ProjectAction`: `List`, `Add { name }`, `Delete`
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
- `FileConfig`: lightweight config with just `todos_path` — used by `todos`, `projects`, and `pomodoro` subcommands. Derives `todos_path` from the parent directory of `AMBROGIO_DAILY_ORGANISER_FILE`.

**Example Configurations:**

| Provider | URL | Model |
|----------|-----|-------|
| Groq | `https://api.groq.com/openai/v1` | `llama-3.3-70b-versatile` |
| OpenRouter | `https://openrouter.ai/api/v1` | `meta-llama/llama-3.3-70b-instruct` |
| OpenAI | `https://api.openai.com/v1` | `gpt-4o` |
| Ollama | `http://localhost:11434/v1` | `llama3` |

### `todo.rs`

File-backed todo store using markdown checkboxes, grouped by project.

**Types:**

- `Todo`: `{ description: String, done: bool, project: String }`
- `TodoStore`: wraps a `PathBuf`, provides project and todo management methods

**File Format (`todos.md`):**

Todos are grouped under `## Project Name` headers:

```markdown
## Work
- [ ] open task
  - 🍅 2026-02-12 10:00
  - 🍅 2026-02-12 14:30 cancelled
- [x] completed task

## Personal
- [ ] buy milk
```

Every todo must belong to a project. Todos without a `## ` header above them are ignored by `load_all()`.

**Project Methods:**

- `projects()` returns ordered list of project names from `## ` headers
- `add_project(name)` appends a `## name` header; creates file if missing; rejects duplicates
- `delete_project(name)` removes the project header and all its content (todos, pomodoros)

**Todo Methods:**

- `add(project, description)` inserts `- [ ] description` at the end of the named project section
- `load_all()` parses all `- [ ] ` and `- [x] ` lines with their project context, ignores pomodoro sub-items
- `open_todos()` returns only unchecked items with project info
- `complete(index)` rewrites the file, changing the nth open todo's `[ ]` to `[x]` (global index across all projects)
- `add_pomodoro(open_index, started_at, cancelled)` inserts a pomodoro entry under the nth open todo, after any existing pomodoro sub-items
- `print_open_todos()` prints open todos grouped by project with global sequential numbering

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
4. `projects` subcommand → `run_projects()` (loads `FileConfig`, operates on `TodoStore`)
5. `pomodoro start` → `run_pomodoro()` (loads `FileConfig`, selects todo, runs countdown, records pomodoro to `todos.md`)

**Interactive Flows:**

- `todos add`: prompts for project selection before adding the todo
- `todos complete`: displays todos grouped by project with global numbering, prompts for selection
- `projects delete`: prompts for project selection, then asks for `y/N` confirmation before deleting project and all its todos
- `pomodoro start`: displays todos grouped by project with global numbering, prompts for selection

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
## Work
- [ ] open task
  - 🍅 2026-02-12 10:00
  - 🍅 2026-02-12 14:30 cancelled
- [x] completed task

## Personal
- [ ] buy milk
- [ ] call dentist
```

**Projects** are `## ` headers. Every todo must belong to a project.

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
| dirs | 6 | Platform config directory resolution |

**Dev Dependencies:**

| Crate | Version | Purpose |
|-------|---------|---------|
| tempfile | 3 | Temp files for tests |

## Hooks

Ambrogio supports user-defined shell scripts that run on specific events.

**Location:** `~/.config/ambrogio/hooks/{feature}/{event}.sh`

**Behavior:**

- If the hook file doesn't exist, nothing happens (silent no-op)
- If it exists, it's executed via `sh` and its stdout/stderr are printed to the terminal
- If the script exits with a non-zero status, a warning is printed but the main flow is not interrupted
- No environment variables are passed to hooks

**Available Hooks:**

| Hook path | Trigger |
|-----------|---------|
| `pomodoro/stop.sh` | After a pomodoro completes successfully (not on cancellation) |

### `hooks.rs`

**Functions:**

- `run(feature, event)`: resolves and executes `~/.config/ambrogio/hooks/{feature}/{event}.sh`

## Limitations

- No streaming responses (waits for full response)
- No persistent chat history across sessions
- Organiser file is loaded once at startup (changes require restart)
- No syntax validation of organiser file format
