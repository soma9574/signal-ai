# Senator Budd Signal Chatbot

This project provides a Signal chatbot that answers in the persona of **Senator Ted Budd** to help Vice Admiral Mitch Bradley prepare for confirmation hearings.

## 🚀 Quick Start

**See [SETUP.md](SETUP.md) for complete step-by-step instructions.**

### TL;DR
1. Install `signal-cli` and register a phone number
2. Get an Anthropic API key 
3. Set environment variables (`ANTHROPIC_API_KEY`, `SIGNAL_PHONE_NUMBER`)
4. Run `cargo run` in the `backend/` directory
5. Text the bot and get responses from "Senator Budd"

## Tech Stack

* **Rust** backend using Axum for HTTP, `sqlx` for SQLite, and a background worker that calls Anthropic **Claude 3 Sonnet** model ("Sonnet 4" tier).
* **SQLite** for chat history (single file database).
* **Signal** transport via `signald`.
* Deployed on **Railway** via GitHub integration.

## Features

- 📱 **Signal Integration** - Text the bot directly from any phone
- 🤖 **Senator Ted Budd Persona** - Responses as the Senator using Claude Sonnet
- 💾 **Chat History** - All conversations saved to SQLite database  
- 🔍 **Health Monitoring** - `/health` endpoint shows system status
- 🪵 **Comprehensive Logging** - Detailed logs for easy debugging
- ⚡ **Real-time Responses** - 10-second polling for new messages

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

## API Endpoints

- `POST /chat` - Web chat interface (JSON)
- `POST /signal/send` - Send Signal messages manually
- `GET /health` - System health check

## Usage for Admiral Bradley

Once set up:
1. Text the registered Signal number
2. Ask questions about military affairs, confirmation hearings, etc.
3. Receive responses from "Senator Budd" within 10 seconds
4. Review conversation history via logs or database

## License

MIT 