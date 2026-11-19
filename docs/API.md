# Email News API Contracts

This document covers the original author, newsletter, and subscriber routes and
the reader account and interaction routes added later. The route inventory at
the end lists every registered endpoint.

## Base URL

The app binds to `0.0.0.0` and uses the `PORT` environment variable. The local default is `8004`; Render supplies `PORT` at runtime.

Examples:

- Local: `http://127.0.0.1:8004`
- Custom: use the value of `PORT`.

## Authentication policy

`POST /auth/register` and `POST /auth/login` are public and keep the existing
`{ "token": "…", "author": { … } }` success response. Send that JWT on
protected API requests using `Authorization: Bearer <token>`.

- Public reads: `GET /authors`, `GET /authors/{id}`, `GET /newsletters`, and
  `GET /newsletters/{id}`.
- Author administration: `POST /authors`, `PUT /authors/{id}`, and
  `DELETE /authors/{id}` require authentication. Update/delete are limited to
  the author represented by the token.
- Newsletter writes: `POST /newsletters`, `PUT /newsletters/{id}`, and
  `DELETE /newsletters/{id}` require authentication. Create requires the
  request's `author_id` to match the token, and update/delete are limited to
  the resource owner. Cross-author resources return `404`.
- Subscriber subscription remains public: `POST /subscribers` and
  `GET /subscribers/confirm/{token}`.
- Subscriber administration: `GET /subscribers`, `GET /subscribers/{id}`, and
  `DELETE /subscribers/{id}` require an author bearer token whose author UUID is
  listed in `SUBSCRIBER_ADMIN_AUTHOR_IDS`. The variable is a comma-separated
  allowlist; if unset, subscriber administration returns `403`.
- Reader registration and login are public through `/readers/register` and
  `/readers/login`. Reader access is separate from author access.
- Reader interaction mutations and reader bookmarks/highlights require a reader
  bearer token. Public clap totals and comment lists do not require one.

Missing, malformed, expired, and invalid bearer tokens receive `401`. Reader
tokens cannot authorize author resources, and author tokens cannot authorize
reader interaction resources.

Access tokens expire after 12 hours. A token for a deleted author or reader is
rejected. All JSON request bodies use `Content-Type: application/json`.

## Common response patterns

### Success responses

Most successful responses return the created or updated object as JSON. JSON
successes use `200 OK`; successful deletion and interaction removal use
`204 No Content`.

```json
{
  "id": "uuid-string",
  "name": "Jane Doe",
  "email": "jane@example.com"
}
```

### Error responses

- `400 Bad Request`: malformed JSON syntax, invalid UUID path parameter, or
  field values rejected by endpoint validation
- `401 Unauthorized`: missing or invalid credentials on a protected route
- `403 Forbidden`: authenticated caller is not permitted to perform the action
- `404 Not Found`: entity not found or a cross-author resource hidden by the API
- `409 Conflict`: duplicate record, such as duplicate email
- `415 Unsupported Media Type`: a JSON endpoint is called without a JSON
  content type
- `422 Unprocessable Entity`: JSON fields are missing or have the wrong type
- `500 Internal Server Error`: server-side failure

Most error responses contain an empty body. The subscription confirmation route
returns a JSON `message` for its success and token errors.

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

- `name`: non-empty string
- `email`: valid email string
- `password`: string, at least 8 bytes after trimming leading/trailing spaces
- `bio`: optional string

Email is trimmed and lowercased. Passwords are trimmed before hashing and
login, so leading/trailing spaces are not significant. Duplicate email
addresses return `409 Conflict`; invalid fields return `400 Bad Request`.

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

Required fields are a valid `email` and a non-empty `password` after trimming.

Invalid credentials return `401 Unauthorized`. Invalid email syntax or an empty
password returns `400 Bad Request`.

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

### 3) Get author session

```http
GET /auth/session
Authorization: Bearer <author-token>
```

Returns the authenticated author's profile, including their email. Deleted or
invalid accounts return `401 Unauthorized`.

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
    "bio": "Tech writer",
    "created_at": "2026-10-03T12:00:00"
  }
]
```

Public author reads return profile fields and omit email addresses. The
authenticated author session and authentication responses still include the
current user's email.

### 2) Get author by id

```http
GET /authors/{id}
```

The response uses the same public profile fields as `GET /authors` and omits
the author's email address.

### 3) Create author directly

```http
POST /authors
Authorization: Bearer <author-token>
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

This route requires a valid author token. The returned object has the same
fields as the authenticated author's response in the registration section.

`name` must not be empty, `email` must be valid, and `password` must contain at
least 8 bytes after trimming. Duplicate emails return `409 Conflict`.

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
Authorization: Bearer <author-token>
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

An updated name cannot be empty. An updated email must be valid and is
normalized to lowercase; a duplicate email returns `409 Conflict`.
The caller may update only their own author ID; another author's ID returns
`404 Not Found`. Omitted or `null` fields are left unchanged.

Success returns the updated author object, including its email.

### 5) Delete author

```http
DELETE /authors/{id}
Authorization: Bearer <author-token>
```

Returns:

- `204 No Content` on success
- `404 Not Found` if not found

An author can delete only their own account. Deleting an account also deletes
its newsletters and associated story interactions through database cascades.

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
    "published": true,
    "created_at": "2026-10-03T12:00:00",
    "updated_at": "2026-10-03T12:00:00"
  }
]
```

Public newsletter reads include published newsletters only. Drafts return
`404` from `GET /newsletters/{id}` and are omitted from `GET /newsletters`.

### 2) Get newsletter by id

```http
GET /newsletters/{id}
```

Returns the newsletter object shown above. Drafts and unknown IDs both return
`404 Not Found`.

### 3) Create newsletter

```http
POST /newsletters
Authorization: Bearer <author-token>
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

The `author_id` must match the bearer token's author ID; a mismatch returns
`404 Not Found`. `title` and `content` must be non-empty after trimming.
Creation always starts with `published: false` and returns the created
newsletter object with `200 OK`.

### 4) Update newsletter

```http
PUT /newsletters/{id}
Authorization: Bearer <author-token>
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

All fields are optional; omitted or `null` fields are unchanged. A non-null
`image_url` replaces the current value; this API does not currently clear an
image URL. `title` and `content`, when supplied, must be non-empty after
trimming. Only the newsletter owner can update it; cross-owner and missing IDs
return `404 Not Found`. Success returns the updated newsletter object.

### 5) Delete newsletter

```http
DELETE /newsletters/{id}
Authorization: Bearer <author-token>
```

Returns:

- `204 No Content` on success
- `404 Not Found` if not found

Only the newsletter owner can delete it; cross-owner IDs also return `404`.

## Subscribers

### 1) Get all subscribers

```http
GET /subscribers
Authorization: Bearer <allowlisted-author-token>
```

Returns confirmed subscribers only. Each response object contains `id`,
`email`, `subscribed_at`, `name`, and `status`; confirmation tokens are never
included. A valid author token not in `SUBSCRIBER_ADMIN_AUTHOR_IDS` receives
`403 Forbidden`.

Success response:

```json
[
  {
    "id": "d2ac90a0-2b37-4d72-8b6a-9c75f3ce3a5d",
    "email": "user@example.com",
    "subscribed_at": "2026-10-03T12:00:00",
    "name": "Test User",
    "status": "confirmed"
  }
]
```

### 2) Get subscriber by id

```http
GET /subscribers/{id}
Authorization: Bearer <allowlisted-author-token>
```

Returns one confirmed subscriber using the same object shape as the list route.
Pending or unknown IDs return `404 Not Found`.

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

- Trims and lowercases the email before validation and storage.
- Rejects invalid or empty emails with `400 Bad Request`.
- Rejects already confirmed addresses with `409 Conflict`.
- For an existing pending address, refreshes the name, generates a new
  confirmation token, and attempts to send a new confirmation email.
- Stores a new address with `status = pending_confirmation`.
- Generates a JWT confirmation token valid for 24 hours.
- Attempts delivery of a confirmation email with a clickable link when SMTP is
  configured. Delivery errors do not change the successful API response.
- If `SMTP_HOST` is unset, the confirmation link is written to the server log
  for local development instead of being emailed.

Success response:

```json
{
  "id": "d2ac90a0-2b37-4d72-8b6a-9c75f3ce3a5d",
  "email": "user@example.com",
  "subscribed_at": "2026-10-03T12:00:00",
  "name": "Test User",
  "status": "pending_confirmation"
}
```

The confirmation token is sent only in the email link and is never returned by
the API.

### 4) Confirm subscriber email

```http
GET /subscribers/confirm/{token}
```

The signed token expires after 24 hours and is one-time use. Resending a
confirmation link invalidates the previous token. The endpoint also requires
the token to match the subscriber's currently stored token.

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
Authorization: Bearer <allowlisted-author-token>
```

Returns:

- `204 No Content` on success
- `404 Not Found` if not found
- `403 Forbidden` if the author ID is not in `SUBSCRIBER_ADMIN_AUTHOR_IDS`

## Subscription confirmation flow

1. Frontend sends `POST /subscribers` with the subscriber email and name.
2. Backend stores the user with `status = pending_confirmation`.
3. Backend creates a JWT confirmation token.
4. Backend attempts to email the confirmation link (or logs it when SMTP is
   not configured).
5. User clicks the confirmation URL.
6. Backend validates the token and updates the user status to `confirmed`.

Example confirmation link:

The link uses `APP_BASE_URL` followed by `/subscribers/confirm/{token}`.

## Reader accounts

### Register reader

```http
POST /readers/register
Content-Type: application/json

{
  "name": "Jane Reader",
  "email": "reader@example.com",
  "password": "StrongPass!123"
}
```

`name` must not be empty, `email` must be valid, and `password` must contain at
least 8 bytes after trimming. Email and password are trimmed at registration
and login. `409 Conflict` is returned for an existing email.

Success response:

```json
{
  "token": "eyJhbGciOiJIUzI1NiJ9...",
  "reader": {
    "id": "9c6f8f73-1a49-4e63-a2d5-b9e4cb4e512e",
    "name": "Jane Reader",
    "email": "reader@example.com",
    "created_at": "2026-11-19T12:00:00"
  }
}
```

### Login reader

```http
POST /readers/login
Content-Type: application/json

{
  "email": "reader@example.com",
  "password": "StrongPass!123"
}
```

Success uses the same `{ "token", "reader" }` shape as registration.
`401 Unauthorized` is returned for invalid credentials.

### Get reader session

```http
GET /readers/session
Authorization: Bearer <reader-token>
```

Returns the authenticated reader object (`id`, `name`, `email`, and
`created_at`). A token for a deleted reader is rejected with `401 Unauthorized`.

## Reader interactions

Send reader bearer tokens on protected routes:

```http
Authorization: Bearer <reader-token>
```

Story IDs and resource IDs are UUID strings. Public clap totals and comment
lists, adding interactions, and listing highlights require the story to exist
and be published; otherwise they return `404 Not Found`. Deleting an existing
owned interaction is allowed even if its story is unpublished. Comment bodies
and highlight text may contain at most 10,000 characters.

### Claps

Get the public clap count:

```http
GET /interactions/claps/{story_id}
```

Success response:

```json
{
  "story_id": "6f8d3d8d-2a7e-4bdd-9f6c-287e0d5d132a",
  "total": 12,
  "reader_clapped": false
}
```

`reader_clapped` is always `false` on this public endpoint; it does not identify
the caller or indicate whether a particular reader has clapped.

Each reader may record one clap for a story. Repeating the request is
idempotent.

```http
POST /interactions/claps/{story_id}
Authorization: Bearer <reader-token>
```

```http
DELETE /interactions/claps/{story_id}
Authorization: Bearer <reader-token>
```

Mutations return `204 No Content`. Removing a clap that does not exist returns
`404 Not Found`.

### Comments

List public comments:

```http
GET /interactions/comments/{story_id}
```

Success response:

```json
[
  {
    "id": "b5ed3d50-ea9d-43f3-9b56-86cf342d40e2",
    "story_id": "6f8d3d8d-2a7e-4bdd-9f6c-287e0d5d132a",
    "reader_id": "9c6f8f73-1a49-4e63-a2d5-b9e4cb4e512e",
    "reader_name": "Jane Reader",
    "body": "Great article!",
    "created_at": "2026-11-19T12:05:00",
    "updated_at": "2026-11-19T12:05:00"
  }
]
```

Create a comment:

```http
POST /interactions/comments/{story_id}
Authorization: Bearer <reader-token>
Content-Type: application/json

{
  "body": "Great article!"
}
```

Success returns the created comment object with `200 OK`. Empty or oversized
bodies return `400 Bad Request`.

Update or delete only your own comment. Other readers' comments and unknown IDs
return `404 Not Found`.

```http
PUT /interactions/comments/{id}
Authorization: Bearer <reader-token>
Content-Type: application/json

{
  "body": "Updated comment"
}
```

The update response has the same comment object shape as comment creation.

```http
DELETE /interactions/comments/{id}
Authorization: Bearer <reader-token>
```

Delete returns `204 No Content` on success. Updating or deleting a comment is
allowed while its story is unpublished, but only by the comment's owner.

### Bookmarks

List the authenticated reader's bookmarks:

```http
GET /interactions/bookmarks
Authorization: Bearer <reader-token>
```

Response:

```json
[
  {
    "id": "b5ed3d50-ea9d-43f3-9b56-86cf342d40e2",
    "story_id": "6f8d3d8d-2a7e-4bdd-9f6c-287e0d5d132a",
    "story_title": "Weekly update",
    "created_at": "2026-11-19T12:05:00"
  }
]
```

Add or remove a bookmark using the story ID:

```http
PUT /interactions/bookmarks/{story_id}
Authorization: Bearer <reader-token>
```

```http
DELETE /interactions/bookmarks/{story_id}
Authorization: Bearer <reader-token>
```

Both successful mutations return `204 No Content`. Adding an existing bookmark
is idempotent. Removing a bookmark that does not exist returns `404`. A
previously bookmarked story that is unpublished or no longer available is
shown with `"story_title": "Unavailable story"`.

### Highlights

List highlights for a published story:

```http
GET /interactions/highlights/{story_id}
Authorization: Bearer <reader-token>
```

Add a highlight. Offsets are nonnegative character offsets with an end greater
than the start.

```http
POST /interactions/highlights/{story_id}
Authorization: Bearer <reader-token>
Content-Type: application/json

{
  "text": "Highlighted passage",
  "start_offset": 12,
  "end_offset": 32
}
```

Success returns the created highlight:

```json
{
  "id": "b5ed3d50-ea9d-43f3-9b56-86cf342d40e2",
  "story_id": "6f8d3d8d-2a7e-4bdd-9f6c-287e0d5d132a",
  "text": "Highlighted passage",
  "start_offset": 12,
  "end_offset": 32,
  "created_at": "2026-11-19T12:05:00"
}
```

Delete only your own highlight:

```http
DELETE /interactions/highlights/{id}
Authorization: Bearer <reader-token>
```

Delete returns `204 No Content`; an unknown or another reader's highlight
returns `404 Not Found`. Invalid text or offsets return `400 Bad Request`.

## Route inventory

| Method | Path | Access | Purpose |
| --- | --- | --- | --- |
| `POST` | `/auth/register` | Public | Register an author and receive an author token |
| `POST` | `/auth/login` | Public | Log in an author |
| `GET` | `/auth/session` | Author token | Get the current author profile, including email |
| `GET` | `/authors` | Public | List public author profiles |
| `GET` | `/authors/{id}` | Public | Get a public author profile |
| `POST` | `/authors` | Author token | Create an author |
| `PUT` | `/authors/{id}` | Owner author token | Update own author profile |
| `DELETE` | `/authors/{id}` | Owner author token | Delete own author account |
| `GET` | `/newsletters` | Public | List published newsletters |
| `GET` | `/newsletters/{id}` | Public | Get a published newsletter |
| `POST` | `/newsletters` | Author token | Create a draft newsletter |
| `PUT` | `/newsletters/{id}` | Owner author token | Update or publish own newsletter |
| `DELETE` | `/newsletters/{id}` | Owner author token | Delete own newsletter |
| `POST` | `/subscribers` | Public | Subscribe an email address |
| `GET` | `/subscribers/confirm/{token}` | Confirmation token | Confirm a subscription |
| `GET` | `/subscribers` | Allowlisted author token | List confirmed subscribers |
| `GET` | `/subscribers/{id}` | Allowlisted author token | Get a confirmed subscriber |
| `DELETE` | `/subscribers/{id}` | Allowlisted author token | Delete a subscriber |
| `POST` | `/readers/register` | Public | Register a reader and receive a reader token |
| `POST` | `/readers/login` | Public | Log in a reader |
| `GET` | `/readers/session` | Reader token | Get the current reader profile |
| `GET` | `/interactions/claps/{story_id}` | Public | Get clap count for a published newsletter |
| `POST` | `/interactions/claps/{story_id}` | Reader token | Clap for a published newsletter |
| `DELETE` | `/interactions/claps/{story_id}` | Reader token | Remove own clap |
| `GET` | `/interactions/comments/{story_id}` | Public | List comments for a published newsletter |
| `POST` | `/interactions/comments/{story_id}` | Reader token | Comment on a published newsletter |
| `PUT` | `/interactions/comments/{id}` | Comment owner reader token | Update own comment |
| `DELETE` | `/interactions/comments/{id}` | Comment owner reader token | Delete own comment |
| `GET` | `/interactions/bookmarks` | Reader token | List own bookmarks |
| `PUT` | `/interactions/bookmarks/{story_id}` | Reader token | Bookmark a published newsletter |
| `DELETE` | `/interactions/bookmarks/{story_id}` | Reader token | Remove own bookmark |
| `GET` | `/interactions/highlights/{story_id}` | Reader token | List own highlights for a published newsletter |
| `POST` | `/interactions/highlights/{story_id}` | Reader token | Add a highlight to a published newsletter |
| `DELETE` | `/interactions/highlights/{id}` | Highlight owner reader token | Delete own highlight |
