# Ambrogio Specifications

This document describes the current implementation of Ambrogio.

## Overview

Ambrogio is a CLI chat tool that queries a daily organiser markdown file using an LLM. It provides a REPL interface for natural language questions about schedule, tasks, and meetings.

## Architecture

```
┌─────────────────────────────────────────────────────┐
│                    main.rs                          │
│                  (REPL loop)                        │
├─────────────────────────────────────────────────────┤
│                    chat.rs                          │
│         (conversation history + system prompt)      │
├─────────────────────────────────────────────────────┤
│                    llm.rs                           │
│            (OpenAI-compatible API client)           │
├─────────────────────────────────────────────────────┤
│                   config.rs                         │
│              (environment variables)                │
└─────────────────────────────────────────────────────┘
```

## Modules

### `config.rs`

Handles configuration via environment variables.

**Environment Variables:**

| Variable | Required | Description |
|----------|----------|-------------|
| `AMBROGIO_LLM_API_KEY` | Yes | API key for the LLM provider |
| `AMBROGIO_LLM_URL` | Yes | Base URL of the OpenAI-compatible API |
| `AMBROGIO_LLM_MODEL` | Yes | Model name to use |
| `AMBROGIO_DAILY_ORGANISER_FILE` | Yes | Path to organiser file |

**Types:**

- `Config` struct: holds validated configuration (api_key, base_url, model, file_path)

**Example Configurations:**

| Provider | URL | Model |
|----------|-----|-------|
| Groq | `https://api.groq.com/openai/v1` | `llama-3.3-70b-versatile` |
| OpenRouter | `https://openrouter.ai/api/v1` | `meta-llama/llama-3.3-70b-instruct` |
| OpenAI | `https://api.openai.com/v1` | `gpt-4o` |
| Ollama | `http://localhost:11434/v1` | `llama3` |

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

Entry point with REPL loop.

**Flow:**

1. Load config from environment
2. Read organiser file
3. Initialize chat manager
4. Start REPL with rustyline
5. Process user input until quit

**Commands:**

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

## Limitations

- No streaming responses (waits for full response)
- No persistent chat history across sessions
- Organiser file is loaded once at startup (changes require restart)
- No syntax validation of organiser file format
