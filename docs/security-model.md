# Security model and risk register

## Trust boundaries

```text
Public browser ── POST category/message ── same-origin edge ── Axum ── PostgreSQL
Admin browser ── credential exchange ── session cookie ── authorized review ── PostgreSQL
Migration/maintenance owner ── schema, backup and retention ── PostgreSQL
```

Anonymous means the form does not ask for identity. It does not guarantee untraceability: message content, network infrastructure, hosting or endpoint compromise can reveal identity. No analytics or external fonts are used. Cookies are used only for administrative access. The public submission contract rejects extra identity fields; users can still type personal information into the message, so privacy guidance is shown before submission.

## Implemented controls

- PostgreSQL-only runtime storage; missing config/connectivity/schema stops startup. Only successful inserts return a receipt. No demo seeds, memory fallback, fabricated totals or silent eviction.
- Category allowlist, trimmed 1–2,000 Unicode-scalar message length, strict JSON, 16 KiB buffered body limit, parameterized SQL and DB constraints. Pool connections enforce a 5-second statement timeout and 2-second lock timeout.
- Admin credential remains server-configured. Browser sign-in exchanges it for an opaque random HttpOnly, SameSite=Strict cookie; Secure and `__Host-` in HTTPS deployments. Origin checked for login/logout. No credential or feedback persisted in browser storage.
- Sessions expire after 30 minutes, are revoked on sign-out, and are lost on server restart. Session storage is bounded (128 active sessions) and login attempts are bounded (20/minute per API process). Edge abuse protection remains necessary.
- Public users cannot list feedback. Review requires a valid session; legacy header tokens do not authorize review. Pagination accepts 1–100 records and an offset up to 1,000,000, newest first. Concurrent submissions can shift offset pages; refresh resets to the latest page.
- Feedback is rendered as text, never raw HTML. API responses use no-store and defensive headers. Full frontend CSP and TLS headers belong at the hosting edge and are specified in the runbook.
- No body, credential or connection-string logging. Request IDs contain random UUIDs only. Tests use synthetic data in a separate database, never runtime seed records.

## Risks and release decisions

| Threat | Current mitigation | Residual risk / required release work |
| --- | --- | --- |
| Unauthorized review / stolen credential | Cookie sessions, strict origin checks, expiration, bounded attempts, no browser credential persistence | Shared credential lacks MFA, individual accountability and revocation; integrate institutional identity/MFA and per-reviewer audit before sensitive institutional use |
| Stored XSS / injection | Text rendering, parameterized SQL, allowlisted category, bound input | Enforce frontend CSP, security review and browser regression checks; no claim of zero XSS |
| Data loss / disclosure | Persistent PostgreSQL, runtime least-privilege role in local setup, no silent eviction | Configure verified TLS for remote DB, encryption, approved retention/deletion, backup/restore and restricted operator access |
| Spam / resource exhaustion | Body/query/session/login bounds, DB pool of 10, 5-second pool acquisition/statement timeouts and 2-second lock timeout | Submission rate limiting, connection/body-read timeouts, WAF policy and storage alerts required at edge/DB; global login budget can be exhausted by another client |
| Cross-site admin actions | Exact origin on mutations, strict cookie, no permissive CORS | XSS or compromised same-origin code defeats these controls; protect hosting and dependencies |
| Sensitive information in a message | No identity fields, prominent privacy guidance, confidential inbox | Approve privacy notices, lawful use, escalation, deletion and retention with the responsible institution |
| Supply chain / insecure release | Locks, source policy, CI scanners and tests | Actually run CI, review findings, pin scanner digests, protect branches, inventory/sign release artifacts and independently assess security |
| Session lifecycle / multiple instances | Process-local expiry and revocation; restart invalidates sessions | Use shared reviewed session storage or a documented single-instance deployment; rotation requires restarting every instance |

No production readiness, compliance certification, guaranteed anonymity or 500K-request capacity claim is made. Release approval requires evidence in [deployment.md](deployment.md), not a checklist merely existing.
