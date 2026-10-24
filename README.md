# Email News

Rust backend for managing authors, newsletters, and subscriber confirmation.

## Features

- Author CRUD
- Newsletter CRUD
- Subscriber registration flow with pending confirmation
- JWT-based verification link for subscription confirmation
- Email validation before subscriber creation

## Stack

- Rust
- Axum
- Tokio
- SeaORM
- PostgreSQL
- JWT via `jsonwebtoken`
- SMTP email delivery via `lettre`

## API docs

See the full frontend API guide here:

- [docs/API.md](docs/API.md)

## Configuration

Keep local values in the ignored root `.env` file. The app loads it for local
development, and Docker excludes it from the image and build context. Do not
commit `.env` or place secret values in source files.

Required settings:

- `DATABASE_URL`: PostgreSQL connection string
- `JWT_SECRET`: random secret of at least 32 bytes

Optional settings:

- `PORT`: listen port (defaults to `8004`; Render supplies this automatically)
- `APP_BASE_URL`: public base URL used in subscription confirmation links
- `SMTP_HOST`, `SMTP_PORT`, `SMTP_USERNAME`, `SMTP_PASSWORD`, `EMAIL_FROM`

Render does not read your local `.env` file. Add the required settings under the
Render service's Environment page; keep those production values out of Git.

## Local development

```bash
cargo run
```

Optional port override:

```bash
PORT=8002 cargo run
```

Default local base URL:

```text
http://127.0.0.1:8004
```

## Deploy on Render

Create a PostgreSQL database, then create a **Web Service** from this
repository and choose **Docker** as the runtime. Add `DATABASE_URL` and
`JWT_SECRET` to the service's Environment settings. Set `APP_BASE_URL` to the
public `https://` URL Render assigns to the web service so confirmation emails
contain reachable links. Add SMTP settings if you want the app to send those
emails.

The container listens on `0.0.0.0:$PORT`; it accepts Render's assigned port.
On startup it runs the SeaORM migrations, then starts the API. The service
needs a PostgreSQL database reachable from Render.

## Docker Compose

Build and start the API with Docker Compose. Compose reads `DATABASE_URL`,
`JWT_SECRET`, and the optional SMTP settings from the root `.env` file:

```bash
docker compose up --build
```

The container applies database migrations before starting the API at
`http://localhost:8004`. The database must be reachable from the Docker host.
