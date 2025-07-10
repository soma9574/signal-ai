# Senator Budd Signal Chatbot

This project provides a Signal chatbot that answers in the persona of **Senator Ted Budd** to help Vice Admiral Mitch Bradley prepare for confirmation hearings.

## Tech Stack

* **Rust** backend using Axum for HTTP, `sqlx` for SQLite, and a background worker that calls Anthropic **Claude 3 Sonnet** model ("Sonnet 4" tier).
* **Effect-TS** React front-end.
* **SQLite** for chat history (single file database).
* **Signal** transport via `signald`.
* Deployed on **Railway** via GitHub integration.

## Local Setup

```
# Back-end
cd backend
cargo run
```

Front-end instructions will follow once the React project is scaffolded.

## Environment Variables

| Variable | Description |
| -------- | ----------- |
| `ANTHROPIC_API_KEY` | API key for Claude Sonnet |
| `DATABASE_URL` | SQLite file path (optional, defaults to `sqlite:chat_history.db`) |
| `SIGNAL_PHONE_NUMBER` | Phone number registered with Signal |

## Signal Integration

The backend includes two Signal client implementations:

1. **SignalCliClient** (default): Uses `signal-cli` command-line tool
2. **SignaldClient**: Uses `signald` daemon via Unix socket

The backend automatically:
- Polls for incoming Signal messages every 10 seconds
- Responds as Senator Ted Budd using Claude Sonnet
- Stores all conversations in SQLite
- Provides REST endpoints for manual message sending

### Prerequisites

Install `signal-cli`:
```bash
# On macOS
brew install signal-cli

# Register your phone number (one-time setup)
signal-cli -a +YOUR_PHONE_NUMBER register
signal-cli -a +YOUR_PHONE_NUMBER verify CODE_FROM_SMS
```

## License

MIT 