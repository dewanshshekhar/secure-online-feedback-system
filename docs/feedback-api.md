# Feedback API contract

All paths are under `/api`. JSON responses use `Cache-Control: no-store`; errors never echo submitted messages or credentials. Requests are limited to 16 KiB by buffered extractors. APIs use the same browser origin; no permissive CORS is configured.

## Public endpoints

`GET /health` returns `200 {"status":"ok"}` for process liveness only, not database readiness.

`POST /feedback` requires `Content-Type: application/json` and exactly two fields:

```json
{"category":"teaching","message":"A specific observation and constructive suggestion."}
```

This is a request example, not a seeded record. Category is `teaching`, `course-content`, `facilities` or `other`. Message is trimmed and must contain 1–2,000 Unicode scalar values. Identity fields and other unexpected fields are rejected. A successful PostgreSQL insert returns `201 {"id":"<uuid>"}`. The receipt is a reference, not a public read-access token. There is no public lookup endpoint.

| Status | Meaning |
| --- | --- |
| 400 | Malformed JSON, invalid category, missing or unknown fields |
| 413 | Buffered body exceeds 16 KiB |
| 415 | Content type is not application/json |
| 422 | Empty or overlong message |
| 503 | Storage failure; no success receipt is returned |

A lost connection after an insert can leave a successfully stored record without a receipt reaching the client. Automatic retries are not used; user guidance warns of duplicate risk. Idempotency keys are a future contract change.

## Administrator session

`POST /admin/session` accepts `{"credential":"<server-configured credential>"}` with JSON content type and an exact `Origin` matching `APP_ORIGIN`. Success returns 204 and an HttpOnly, SameSite=Strict session cookie (Secure, `__Host-` prefixed in HTTPS mode), max age 1,800 seconds. The credential is never returned. Invalid credential: 401; origin mismatch/missing: 403; login budget/session capacity exceeded: 429; disabled sign-in: 503. There is no shipped credential.

`GET /admin/session`: 204 for a valid session, otherwise 401.

`DELETE /admin/session`: requires the configured Origin, revokes the presented session and clears its cookie, returns 204. Signing out does not delete feedback. Sessions are memory-held credentials only, expire after 30 minutes, and are invalidated by API restart.

## Review

`GET /admin/feedback?limit=20&offset=0` requires the session cookie; a header token alone is insufficient. Missing/invalid/expired session returns 401. Invalid query returns 400. Limit: 1–100; offset: 0–1,000,000. Unknown query fields are rejected. Storage failure returns 503.

```json
{"items":[],"has_more":false,"next_offset":null}
```

An empty database returns this genuine empty page. Each item contains `id`, `category`, `message` and `created_at` (UTC RFC3339). Results are newest first with ID tie-breaking. One extra record is queried to compute `has_more`; no invented or expensive total count is exposed. `next_offset` is provided only when another page exists. Concurrent inserts may shift offset pages; refresh starts at offset zero. Authenticated detail views use the already loaded record and render its message as text.
