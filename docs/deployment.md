# Local operation and release runbook

## Windows preview

Prerequisites: Rust 1.85.1 or later, PostgreSQL 17 binaries, PowerShell 7, Trunk 0.21.14, and `rustup target add wasm32-unknown-unknown`.

```powershell
cargo install trunk --version 0.21.14 --locked --root ./.tools
./scripts/setup-local.ps1
./scripts/start-backend.ps1
# Separate terminal:
./scripts/start-frontend.ps1
```

Open <http://127.0.0.1:8080> for submissions and <http://127.0.0.1:8080/admin> for review. Read the generated credential in `.local/admin-credential.txt` to sign in. The scripts create an isolated loopback PostgreSQL cluster on port 55432, use SCRAM passwords, apply the schema, and grant the runtime role SELECT/INSERT only. They never seed records or modify another installed cluster. `.local` holds credentials/data, is ignored by Git, and is restricted to the current Windows identity. Never commit or share it. Stop application terminals with Ctrl+C, then run `scripts/stop-local.ps1`.

The separate `feedback_test` database is disposable. For the ignored persistence test, set `TEST_DATABASE_URL` to its owner connection from `.local/settings.json`, then run `cargo test --locked --test postgres -- --ignored` in `backend/`. Never point tests at a production database.

## Other environments

Provision PostgreSQL 17 and apply `backend/migrations/0001_feedback.sql` as a migration owner (`psql -v ON_ERROR_STOP=1 -f ...`). Use a separate runtime account with CONNECT, schema USAGE, and table SELECT/INSERT only. Set `DATABASE_URL`, `ADMIN_REVIEW_TOKEN` (32+ random bytes), `APP_ORIGIN` (exact browser origin without trailing slash), `COOKIE_SECURE=true`, and `BIND_ADDR`. The app fails startup if required configuration, connectivity, or schema are missing. There is no runtime in-memory fallback.

`COOKIE_SECURE=false` is restricted to loopback development. Production needs HTTPS and a secure cookie; do not weaken settings to solve proxy configuration mistakes. Secrets must come from a secret manager. The supplied credential flow is a single-reviewer deployment baseline, not institutional identity or per-user audit attribution.

## Hosting

Run `trunk build --release --locked` in `frontend/`, host `frontend/dist/` at the same origin as `/api`, and route non-API application paths (including `/admin`) to `index.html`. Forward `/api/*` to the backend without rewriting its prefix. Do not enable permissive CORS. Protect the backend from direct internet access.

At the edge enforce TLS, request/connection timeouts, 16 KiB body limits, connection budgets and rate limits for both submissions and sign-in. Trust forwarding headers only from configured proxies. Return no-store for HTML and admin routes; use hashed immutable assets for JS/WASM/CSS. Apply frontend headers separately: `X-Content-Type-Options: nosniff`, `Referrer-Policy: no-referrer`, `Permissions-Policy: camera=(), microphone=(), geolocation=()`, and a tested CSP including `default-src 'self'; script-src 'self' 'wasm-unsafe-eval'; style-src 'self'; connect-src 'self'; object-src 'none'; base-uri 'none'; frame-ancestors 'none'; form-action 'self'`. The API's `default-src 'none'` policy is not a frontend policy. Enable HSTS only after HTTPS is verified. Serve the bundle with correct MIME types; never deploy Trunk's development server.

## Release gate

Record the commit, artifact hashes, dependency/SBOM inventory, successful CI runs, security findings and resolutions, restore-test evidence, privacy/retention approval, identity/MFA design, and reviewer sign-off. The pipeline has no automatic deployment. Configure required branch checks in GitHub; workflow files alone do not enforce branch protection. Scanner images use version tags and must be reviewed and pinned to verified digests before a production release. Container/IaC scanning has no deployment artifacts to inspect yet; scan the actual image and infrastructure when those artifacts exist.

## Maintenance and incidents

- Monitor availability, DB pool health, response codes, sign-in failures, storage growth, and dependency advisories. Do not capture bodies, cookies, tokens, or full connection strings. API request IDs are correlation aids, not identities.
- Configure DB encryption, TLS certificate verification (`sslmode=verify-full` for remote production PostgreSQL), encrypted backups, and restricted backup access. Test restore into isolation before release and periodically thereafter.
- Approve a time-based retention period and deletion procedure before collecting real feedback. No automatic eviction is implemented; monitor storage and execute reviewed deletion jobs with a separate maintenance role. Never silently discard accepted feedback.
- For a suspected disclosure: disable edge access, rotate the admin credential, restart API instances to revoke in-memory sessions, preserve redacted evidence, assess impact, notify according to the approved policy, fix, retest and document the cause.
- Roll back application artifacts only after checking schema compatibility. The initial migration is additive and idempotent; do not drop the feedback table as a rollback. Back up before any migration and test both recovery and forward repair.
