# Rust Newsletter

A newsletter backend built with **Rust** that allows authors to manage subscribers and send email newsletters.

## Features

- Author authentication
- Subscriber management
- Newsletter creation and management
- Email subscription and unsubscription
- Newsletter publishing
- Email delivery

## Tech Stack

- **Rust**
- **Axum** — Web framework
- **Tokio** — Async runtime
- **SQLx** — Database access
- **PostgreSQL** — Database
- **SMTP** — Email delivery

## API

### Authentication

```http
POST /auth/register
POST /auth/login
```

### Subscribers

```http
POST /subscriptions
POST /subscriptions/unsubscribe
```

### Newsletters

```http
POST   /newsletters
GET    /newsletters
GET    /newsletters/{id}
PUT    /newsletters/{id}
DELETE /newsletters/{id}
POST   /newsletters/{id}/publish
```

## Getting Started

```bash
git clone  https://github.com/ChifundoGKauwa/Email-News.git
cd newsletter

cargo run
```

The API will be available at:

```text
http://localhost:3000
```
