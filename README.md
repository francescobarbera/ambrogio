# Ambrogio

<img src="ambrogio.png" alt="Ambrogio logo" width="200">

Your daily organiser assistant for the terminal. Manage tasks, projects, pomodoro sessions, and chat with your daily schedule via an LLM.

## Installation

```bash
cargo build --release
```

The binary will be at `./target/release/ambrogio`.

## Configuration

Set environment variables before running:

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `AMBROGIO_DAILY_ORGANISER_FILE` | Yes | - | Path to your organiser file |
| `AMBROGIO_LLM_API_KEY` | REPL only | - | API key for the LLM provider |
| `AMBROGIO_LLM_URL` | REPL only | - | Base URL of the OpenAI-compatible API |
| `AMBROGIO_LLM_MODEL` | REPL only | - | Model name to use |
| `AMBROGIO_LLM_TIMEOUT` | No | `10` | Request timeout in seconds |

Only `AMBROGIO_DAILY_ORGANISER_FILE` is required for task management, projects, notes, and pomodoro. The LLM variables are only needed for the chat REPL.

### Example providers

| Provider | URL | Example model |
|----------|-----|---------------|
| Groq | `https://api.groq.com/openai/v1` | `llama-3.3-70b-versatile` |
| OpenRouter | `https://openrouter.ai/api/v1` | `meta-llama/llama-3.3-70b-instruct` |
| OpenAI | `https://api.openai.com/v1` | `gpt-4o` |
| Ollama | `http://localhost:11434/v1` | `llama3` |

## Usage

### Tasks

Manage your task list. Tasks are grouped by project in a markdown file (`todos.md`).

```bash
ambrogio tasks add 'buy milk'     # Add a task (prompts for project)
ambrogio tasks list                # List open tasks grouped by project
ambrogio tasks complete            # Mark a task as done (interactive)
ambrogio tasks delete              # Remove a task and its sub-items (interactive)
```

### Projects

Organise tasks under projects.

```bash
ambrogio projects list             # List all projects
ambrogio projects add 'Work'       # Create a new project
ambrogio projects delete           # Delete a project and all its tasks (interactive)
```

### Notes

Attach notes to tasks. Notes appear as indented sub-items under the task.

```bash
ambrogio note 'call back tomorrow' # Add a note to a task (interactive)
```

### Pomodoro

25-minute focus sessions tied to a task. Completed pomodoros are recorded as sub-items.

```bash
ambrogio pomodoro start            # Start a pomodoro (interactive task selection)
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
- **09:00** work on god mode feature
- **12:30** lunch with Beatrice
- **14:30** work on GeoTech

you: quit
Goodbye!
```

### Aliases

All commands have short aliases for quick access:

| Command | Alias | Subcommand | Alias |
|---------|-------|------------|-------|
| `tasks` | `t` | `add` | `a` |
| `projects` | `p` | `list` | `l` |
| `pomodoro` | `pom` | `complete` | `c` |
| `note` | `n` | `delete` | `d` |
| | | `start` | `s` |

```bash
ambrogio t l                       # tasks list
ambrogio t a 'buy milk'            # tasks add 'buy milk'
ambrogio t c                       # tasks complete
ambrogio t d                       # tasks delete
ambrogio n 'some note'             # note 'some note'
ambrogio pom s                     # pomodoro start
ambrogio p l                       # projects list
```

## File formats

### Organiser file

```markdown
# 2026-01-23
**09:00** meeting with team
**12:30** lunch
**14:00** work on project [TODO]
**16:00** completed task [DONE]

# 2026-01-22
**09:00** another day...
```

### Task file (`todos.md`)

Located in the same directory as the organiser file.

```markdown
## Work
- [ ] open task
  - 🍅 2026-02-12 10:00
  - 🍅 2026-02-12 14:30 cancelled
  - 📝 important detail
- [x] completed task

## Personal
- [ ] buy milk
  - 📝 get oat milk
```

## Hooks

User-defined shell scripts that run on specific events.

| Hook path | Trigger |
|-----------|---------|
| `~/.config/ambrogio/hooks/pomodoro/stop.sh` | After a pomodoro completes (not on cancellation) |

Hooks are silent no-ops if the file doesn't exist. Non-zero exit codes print a warning but don't interrupt the main flow.
