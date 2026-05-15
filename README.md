# Ambrogio

<img src="ambrogio.png" alt="Ambrogio logo" width="200">

Your daily organiser assistant for the terminal. Manage today's tasks, run pomodoro sessions, and chat with your daily schedule via an LLM.

## Installation

```bash
cargo build --release
```

The binary will be at `./target/release/ambrogio`.

## Configuration

Set environment variables before running:

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `AMBROGIO_DAILY_ORGANISER_FILE` | Yes | - | Path to your daily organiser file |
| `AMBROGIO_LLM_API_KEY` | REPL only | - | API key for the LLM provider |
| `AMBROGIO_LLM_URL` | REPL only | - | Base URL of the OpenAI-compatible API |
| `AMBROGIO_LLM_MODEL` | REPL only | - | Model name to use |
| `AMBROGIO_LLM_TIMEOUT` | No | `10` | Request timeout in seconds |

Only `AMBROGIO_DAILY_ORGANISER_FILE` is required for task management and pomodoro. The LLM variables are only needed for the chat REPL.

### Example providers

| Provider | URL | Example model |
|----------|-----|---------------|
| Groq | `https://api.groq.com/openai/v1` | `llama-3.3-70b-versatile` |
| OpenRouter | `https://openrouter.ai/api/v1` | `meta-llama/llama-3.3-70b-instruct` |
| OpenAI | `https://api.openai.com/v1` | `gpt-4o` |
| Ollama | `http://localhost:11434/v1` | `llama3` |

## Usage

### Tasks

All task commands operate on **today's** `## Todos:` block inside the daily organiser file. You decide each morning which tasks belong to today by manually copying unfinished items from the previous day's section — that planning step is intentional. If today's section is missing, `list`, `complete`, and `pomodoro` print "No tasks for today." Only `tasks add` creates the section.

```bash
ambrogio tasks add 'buy milk'     # Append - [ ] buy milk to today's Todos
ambrogio tasks list                # List today's open tasks
ambrogio tasks complete            # Flip [ ] → [x] on a chosen task
```

### Pomodoro

25-minute focus sessions tied to one of today's open tasks. Each completed pomodoro appends a `🍅` to the task line. After each completed pomodoro a 5-minute break starts, then you pick the next task. Ctrl+C exits.

```bash
ambrogio pomodoro start
```

### Chat REPL

Run without arguments to start an interactive chat with your daily organiser.

```bash
ambrogio
```

```
Ambrogio - Your daily organiser assistant
Type 'quit' or 'exit' to leave

you: What do I have to do today?

ambrogio: Based on your organiser for today:
- applicare le modifiche suggerite da Raffaele
- finire la configurazione di aws-infrastructure
- fixare i problemi di GeoTech

you: quit
Goodbye!
```

### Aliases

| Command | Alias | Subcommand | Alias |
|---------|-------|------------|-------|
| `tasks` | `t` | `add` | `a` |
| `pomodoro` | `pom` | `list` | `l` |
| | | `complete` | `c` |
| | | `start` | `s` |

```bash
ambrogio t l                       # tasks list
ambrogio t a 'buy milk'            # tasks add 'buy milk'
ambrogio t c                       # tasks complete
ambrogio pom s                     # pomodoro start
```

## Organiser file format

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

- Day headers: `# YYYY-MM-DD` (one hash, then space, then ISO date).
- Today's tasks live under `## Todos:`. Other subsections (such as `## Logs:`) are never read or written by `ambrogio`.
- Tasks may be written as `- [ ] foo` (what `tasks add` writes) or `[ ] foo  ` (manual style, dashless). Both forms are recognised; modifications preserve the line's leading form and trailing whitespace.
- Pomodoros accumulate as `🍅` characters at the end of the task text, before any trailing whitespace.

## Hooks

User-defined shell scripts that run on specific events.

| Hook path | Trigger |
|-----------|---------|
| `~/.config/ambrogio/hooks/pomodoro/stop.sh` | After a pomodoro completes (not on cancellation) |
| `~/.config/ambrogio/hooks/break/stop.sh` | After a break completes (not on cancellation) |

Hooks are silent no-ops if the file doesn't exist. Non-zero exit codes print a warning but don't interrupt the main flow.
