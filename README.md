# Ambrogio

A terminal-based assistant that helps you query your daily organiser using an LLM.

## Installation

```bash
cargo build --release
```

The binary will be at `./target/release/ambrogio`.

## Configuration

Set environment variables before running:

| Variable | Required | Description |
|----------|----------|-------------|
| `AMBROGIO_LLM_API_KEY` | Yes | API key for the LLM provider |
| `AMBROGIO_LLM_URL` | Yes | Base URL of the OpenAI-compatible API |
| `AMBROGIO_LLM_MODEL` | Yes | Model name to use |
| `AMBROGIO_DAILY_ORGANISER_FILE` | Yes | Path to your organiser file |

### Example providers

| Provider | URL | Example model |
|----------|-----|---------------|
| Groq | `https://api.groq.com/openai/v1` | `llama-3.3-70b-versatile` |
| OpenRouter | `https://openrouter.ai/api/v1` | `meta-llama/llama-3.3-70b-instruct` |
| OpenAI | `https://api.openai.com/v1` | `gpt-4o` |
| Ollama | `http://localhost:11434/v1` | `llama3` |

## Usage

```bash
export AMBROGIO_LLM_API_KEY="your-api-key"
export AMBROGIO_LLM_URL="https://api.groq.com/openai/v1"
export AMBROGIO_LLM_MODEL="llama-3.3-70b-versatile"
export AMBROGIO_DAILY_ORGANISER_FILE="./daily_organiser.md"
./target/release/ambrogio
```

Example session:

```
Ambrogio - Your daily organiser assistant
Type 'quit' or 'exit' to leave

you: What do I have to do today?

ambrogio: Based on your organiser for today:
- **09:00** work on god mode feature
- **12:30** lunch with Beatrice
- **14:30** work on GeoTech

you: Do I have open TODOs?

ambrogio: Yes, you have 7 open TODOs:
- ABIT: backlog for security updates
- GeoTech: Share text includes extra
- GeoTech: Surface values wrong or missing
...

you: quit
Goodbye!
```

## Organiser format

Ambrogio expects a markdown file with this structure:

```markdown
# 2026-01-23
**09:00** meeting with team
**12:30** lunch
**14:00** work on project [TODO]

# 2026-01-22
**09:00** completed task [DONE]
```

- Dates: `# YYYY-MM-DD`
- Scheduled items: `**HH:MM** description`
- Open tasks: `[TODO]`
- Completed tasks: `[DONE]`
