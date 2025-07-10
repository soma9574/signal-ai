# Senator Budd Signal Chatbot

This project provides a Signal chatbot that answers in the persona of **Senator Ted Budd** to help Vice Admiral Mitch Bradley prepare for confirmation hearings.

## Tech Stack

* **Rust** backend using Axum for HTTP, `sqlx` for Postgres, and a background worker that calls Anthropic **Claude 3 Sonnet** model ("Sonnet 4" tier).
* **Effect-TS** React front-end.
* **PostgreSQL** for chat history.
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
| `DATABASE_URL` | Postgres connection string |
| `SIGNAL_SERVICE_URL` | URL of signald instance |

## License

MIT 