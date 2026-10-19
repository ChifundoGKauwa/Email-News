# Email News API Documentation

This document describes the backend API contract for the frontend team.

## Base URL

The app binds to the local loopback address. The default port is `8001`, but it can be overridden by the `PORT` environment variable.

Examples:

- Default: `http://127.0.0.1:8001`
- Custom: `http://127.0.0.1:8002`

## Authentication policy

For the frontend contract, the API is intentionally kept open for normal CRUD usage.

- `POST /auth/register` is public.
- `POST /auth/login` is public.
- `GET /authors`, `POST /authors`, `PUT /authors/{id}`, `DELETE /authors/{id}` are public.
- `GET /newsletters`, `POST /newsletters`, `PUT /newsletters/{id}`, `DELETE /newsletters/{id}` are public.
- `POST /subscribers` and `GET /subscribers/confirm/{token}` are public.
- The app generates JWTs for successful author login and for subscriber confirmation, but the current frontend-facing contract does not require a bearer token for normal CRUD operations.

## Common response patterns

### Success responses

Most successful responses return the created or updated object as JSON.

```json
{
  "id": "uuid-string",
  "name": "Jane Doe",
  "email": "jane@example.com"
}
```

### Error responses

- `400 Bad Request`: invalid body, invalid email, or invalid JWT
- `401 Unauthorized`: invalid or missing credentials when auth is being checked
- `404 Not Found`: entity not found
- `409 Conflict`: duplicate record, such as duplicate email
- `500 Internal Server Error`: server-side failure

## Auth endpoints

### 1) Register author

```http
POST /auth/register
Content-Type: application/json
```

Request body:

```json
{
  "name": "Jane Doe",
  "email": "jane@example.com",
  "password": "StrongPass!123",
  "bio": "Newsletter author"
}
```

Required fields:

- `name`: string
- `email`: valid email string
- `password`: string, minimum length 8
- `bio`: optional string

Success response:

```json
{
  "token": "eyJhbGciOiJIUzI1NiJ9...",
  "author": {
    "id": "uuid-string",
    "name": "Jane Doe",
    "email": "jane@example.com",
    "bio": "Newsletter author",
    "created_at": "2026-10-03T12:00:00"
  }
}
```

### 2) Login author

```http
POST /auth/login
Content-Type: application/json
```

Request body:

```json
{
  "email": "jane@example.com",
  "password": "StrongPass!123"
}
```

Required fields:

- `email`: valid email
- `password`: non-empty string

Success response:

```json
{
  "token": "eyJhbGciOiJIUzI1NiJ9...",
  "author": {
    "id": "uuid-string",
    "name": "Jane Doe",
    "email": "jane@example.com",
    "bio": "Newsletter author",
    "created_at": "2026-10-03T12:00:00"
  }
}
```

## Authors

### 1) Get all authors

```http
GET /authors
```

Success response:

```json
[
  {
    "id": "a1f1d378-1e64-4e4d-ab40-1d1f65d39c9f",
    "name": "Jane Doe",
    "email": "jane@example.com",
    "bio": "Tech writer",
    "created_at": "2026-10-03T12:00:00"
  }
]
```

### 2) Get author by id

```http
GET /authors/{id}
```

### 3) Create author directly

```http
POST /authors
Content-Type: application/json
```

Request body:

```json
{
  "name": "Jane Doe",
  "email": "jane@example.com",
  "password": "StrongPass!123",
  "bio": "Tech writer"
}
```

Success response:

```json
{
  "id": "a1f1d378-1e64-4e4d-ab40-1d1f65d39c9f",
  "name": "Jane Doe",
  "email": "jane@example.com",
  "bio": "Tech writer",
  "created_at": "2026-10-03T12:00:00"
}
```

### 4) Update author

```http
PUT /authors/{id}
Content-Type: application/json
```

Request body: all fields are optional.

```json
{
  "name": "Jane Smith",
  "email": "jane.smith@example.com",
  "bio": "Senior tech writer"
}
```

### 5) Delete author

```http
DELETE /authors/{id}
```

Returns:

- `204 No Content` on success
- `404 Not Found` if not found

## Newsletters

### 1) Get all newsletters

```http
GET /newsletters
```

Success response:

```json
[
  {
    "id": "6f8d3d8d-2a7e-4bdd-9f6c-287e0d5d132a",
    "author_id": "a1f1d378-1e64-4e4d-ab40-1d1f65d39c9f",
    "title": "Weekly update",
    "content": "Hello from the team!",
    "image_url": null,
    "published": false,
    "created_at": "2026-10-03T12:00:00",
    "updated_at": "2026-10-03T12:00:00"
  }
]
```

### 2) Get newsletter by id

```http
GET /newsletters/{id}
```

### 3) Create newsletter

```http
POST /newsletters
Content-Type: application/json
```

Request body:

```json
{
  "author_id": "a1f1d378-1e64-4e4d-ab40-1d1f65d39c9f",
  "title": "Weekly update",
  "content": "Hello from the team!",
  "image_url": "https://cdn.example.com/abcd1234.jpg"
}
```

### 4) Update newsletter

```http
PUT /newsletters/{id}
Content-Type: application/json
```

Request body: all fields are optional.

```json
{
  "title": "Updated weekly update",
  "content": "This is the new newsletter content",
  "published": true,
  "image_url": "https://cdn.example.com/updated.jpg"
}
```

### 5) Delete newsletter

```http
DELETE /newsletters/{id}
```

Returns:

- `204 No Content` on success
- `404 Not Found` if not found

## Subscribers

### 1) Get all subscribers

```http
GET /subscribers
```

Success response:

```json
[
  {
    "id": "d2ac90a0-2b37-4d72-8b6a-9c75f3ce3a5d",
    "email": "user@example.com",
    "subscribed_at": "2026-10-03T12:00:00",
    "name": "Test User",
    "status": "pending_confirmation",
    "confirmation_token": "jwt-token-string"
  }
]
```

### 2) Get subscriber by id

```http
GET /subscribers/{id}
```

### 3) Create subscriber

```http
POST /subscribers
Content-Type: application/json
```

Request body:

```json
{
  "email": "user@example.com",
  "name": "Test User"
}
```

Behavior:

- Validates email format.
- Rejects invalid emails with `400 Bad Request`.
- Rejects duplicates with `409 Conflict`.
- Stores the record with `status = pending_confirmation`.
- Generates a JWT confirmation token.
- Sends a confirmation email with a clickable link.

Success response:

```json
{
  "id": "d2ac90a0-2b37-4d72-8b6a-9c75f3ce3a5d",
  "email": "user@example.com",
  "subscribed_at": "2026-10-03T12:00:00",
  "name": "Test User",
  "status": "pending_confirmation",
  "confirmation_token": "eyJhbGciOiJIUzI1NiJ9..."
}
```

### 4) Confirm subscriber email

```http
GET /subscribers/confirm/{token}
```

On success:

```json
{
  "message": "Subscription confirmed successfully.",
  "email": "user@example.com",
  "status": "confirmed"
}
```

On invalid token:

```json
{
  "message": "Invalid or expired confirmation token."
}
```

### 5) Delete subscriber

```http
DELETE /subscribers/{id}
```

Returns:

- `204 No Content` on success
- `404 Not Found` if not found

## Subscription confirmation flow

1. Frontend sends `POST /subscribers` with the subscriber email and name.
2. Backend stores the user with `status = pending_confirmation`.
3. Backend creates a JWT confirmation token.
4. Backend emails the confirmation link to the subscriber.
5. User clicks the confirmation URL.
6. Backend validates the token and updates the user status to `confirmed`.

Example confirmation link:

```text
http://127.0.0.1:8001/subscribers/confirm/{token}
```

## Environment variables

```bash
DATABASE_URL=postgresql://user:password@localhost:5432/email_news
PORT=8002
JWT_SECRET=your-secret-key
APP_BASE_URL=http://127.0.0.1:8002
SMTP_HOST=smtp.example.com
SMTP_PORT=587
SMTP_USERNAME=your-user
SMTP_PASSWORD=your-password
EMAIL_FROM=no-reply@example.com
```

For Gmail, use the Gmail SMTP server and a Google App Password instead of your normal account password:

```bash
SMTP_HOST=smtp.gmail.com
SMTP_PORT=587
SMTP_USERNAME=you@gmail.com
SMTP_PASSWORD=abcdefghijklmnop
EMAIL_FROM=you@gmail.com
```
