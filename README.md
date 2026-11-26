# Secure Feedback

A Rust student-feedback portal with a public submission form and a separate administrator inbox. Both use the real API and PostgreSQL. No sample feedback, seeded accounts, invented statistics or runtime demo data are included.

## See it locally

On Windows with PowerShell 7, PostgreSQL 17 binaries and Rust installed:

```powershell
rustup target add wasm32-unknown-unknown
cargo install trunk --version 0.21.14 --locked --root ./.tools
./scripts/setup-local.ps1
./scripts/start-backend.ps1
# In another terminal:
./scripts/start-frontend.ps1
```

- Public form: <http://127.0.0.1:8080>
- Admin sign-in/inbox: <http://127.0.0.1:8080/admin>
- Generated local access credential: `.local/admin-credential.txt` (private, ignored by Git).

Start with an empty inbox, submit feedback through the form, then sign in and refresh the inbox to read it. Accepted messages survive application restarts. The local scripts use an isolated PostgreSQL cluster on port 55432 and create no feedback records. For other environments, configuration, least privilege, shutdown and deployment guidance, see [the runbook](docs/deployment.md).

## What works

- Responsive Leptos WebAssembly frontend with keyboard-accessible category selection, message validation, pending/error states and server-issued receipts.
- Admin sign-in, short-lived HttpOnly sessions, sign-out, newest-first paginated review, expandable messages and genuine empty/loading/error states.
- Axum/Tokio API, strict request handling, parameterized PostgreSQL persistence, body/query bounds, defensive API headers and no content/credential logging.
- Security regression tests, database persistence verification and SSDLC CI gates.

## Structure

```text
frontend/src/pages/       Public form, admin session and inbox components
backend/src/features/     Feedback validation, persistence, API and admin sessions
backend/migrations/       PostgreSQL schema (no seed data)
scripts/                 Isolated local setup and source security checks
.github/workflows/       Tests and security gates (no automatic deployment)
docs/                    Requirements, API, threat model, SSDLC and operations
```

## Verification

In `backend/`: `cargo fmt --check`, `cargo clippy --all-targets --locked -- -D warnings`, and `cargo test --locked`. The separate persistence test needs a disposable `feedback_test` database; see the runbook.

In `frontend/`: `cargo fmt --check`, `cargo clippy --target wasm32-unknown-unknown --locked -- -D warnings`, and `trunk build --locked`. Run `python scripts/security-check.py` from the root. Actual local results and browser observations are recorded in [verification.md](docs/verification.md).

## Security and production scope

The design follows the five principles and six phases in the supplied SSDLC images, mapped to NIST SSDF, Microsoft SDL, and automated security gates in [ssdlc.md](docs/ssdlc.md). Documentation does not constitute certification or evidence that CI/deployment controls have run.

This is a production-oriented foundation, not an approved institutional deployment. A shared admin credential is not institutional identity/MFA or per-reviewer audit. Configure those controls, TLS, edge abuse limits, backups, retention/deletion, monitoring and an independent security review before collecting sensitive real-world feedback. See [security-model.md](docs/security-model.md) and [deployment.md](docs/deployment.md) for residual risks and release gates. Anonymous forms do not guarantee untraceability, and Rust does not guarantee zero XSS. No 500K-request capacity claim has been tested.

Contributors: [CONTRIBUTING.md](CONTRIBUTING.md). Security reporting: [SECURITY.md](SECURITY.md). No license has been selected.
