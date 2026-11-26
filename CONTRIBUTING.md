# Contributing

## Before starting

Read [README.md](README.md), [the SSDLC](docs/ssdlc.md) and [the risk register](docs/security-model.md). PostgreSQL persistence and administrator sessions are implemented; institutional identity/MFA, approved retention, edge controls and deployment review remain release gates. Discuss changes to anonymity, permissions, retention or deployment assumptions before implementing them.

## Development setup

1. Install Rust 1.85.1 or later and PostgreSQL 17.
2. Install `wasm32-unknown-unknown` with `rustup target add wasm32-unknown-unknown`.
3. Install Trunk with `cargo install trunk --version 0.21.14 --locked --root ./.tools`.
4. On Windows/PowerShell 7, run `scripts/setup-local.ps1`, then `scripts/start-backend.ps1`.
5. Run `scripts/start-frontend.ps1` in another terminal and open <http://127.0.0.1:8080>. Other environments must provision the schema and variables in [the runbook](docs/deployment.md).

Both applications are independent Cargo packages. Run commands from the appropriate directory. The manifests declare Rust 1.85 as the baseline, with dependency resolution configured to prefer compatible releases. Leptos 0.7 is the initial frontend baseline; evaluate upgrades deliberately.

`backend/.env.example` documents required variables; it is not loaded automatically. For example, after providing `DATABASE_URL`, `ADMIN_REVIEW_TOKEN`, `APP_ORIGIN` and `COOKIE_SECURE` securely, in PowerShell:

```powershell
$env:BIND_ADDR = '127.0.0.1:3000'
$env:RUST_LOG = 'info'
cargo run
```

In a POSIX shell:

```sh
BIND_ADDR=127.0.0.1:3000 RUST_LOG=info cargo run
```

Use synthetic feedback only. Keep credentials and local `.env` files out of Git.

## Making changes

- Use a focused branch, for example `codex/feedback-validation`.
- Keep changes small and describe the user-visible behavior they add.
- Put frontend pages in `frontend/src/pages/` and API features in `backend/src/features/`.
- Add shared modules only when multiple features need them. Keep transport, domain rules, and persistence responsibilities clear.
- Validate input on the server, render user text safely, and never log feedback bodies, tokens, or personal information.
- Do not introduce unsafe Rust without an explicit design review; the current applications forbid it.
- Add meaningful tests for behavior, especially authorization, validation, and failure paths.
- Treat feedback text, credentials, tokens, and request metadata as sensitive. Do not add logging or analytics without a documented privacy review.

## Required checks

Run in `backend/`:

```sh
cargo fmt --check
cargo clippy --all-targets --locked -- -D warnings
cargo test --locked
```

In `frontend/`, run `cargo fmt --check`, `cargo clippy --target wasm32-unknown-unknown --locked -- -D warnings` and `trunk build --locked`. Run `python scripts/security-check.py` at the root. Check mobile/desktop, keyboard navigation, visible focus, real submission receipt, protected review, sign-out, refresh and empty/error states. Verify health returns HTTP 200 and unknown routes return 404. Run the ignored PostgreSQL test only against the disposable `feedback_test` database described in the runbook.

For security-sensitive changes, also test malformed JSON, unexpected fields, non-JSON content types, oversized bodies, missing and invalid admin credentials, pagination bounds, response headers, and that error responses never echo feedback text. Review [the security model](docs/security-model.md) before calling the system production-ready.

Keep both `Cargo.lock` files committed. Record actual results in [verification.md](docs/verification.md); never report a blocked check as passing. CI includes history secret scanning and dependency/misconfiguration scanning. Findings require triage, a fix, and retesting; do not add blanket exclusions. GitHub required checks and release approval must be configured separately by the repository owner.

The API body limit protects supported buffered extractors such as `String` and `Json`. Any future streaming/raw-body endpoint needs its own explicit limit and tests. The health endpoint reports liveness only, not database readiness.

## Commits and review

Use descriptive messages such as `feat: validate feedback submissions`, `fix: reject oversized requests`, or `docs: explain local setup`. Keep unrelated work in separate commits.

Pull request descriptions should explain the problem, resulting behavior, validation, and remaining limitations. Include screenshots for visible changes and migration/rollback details for schema changes. Never claim production security or throughput without evidence.

Pushing commits and opening a pull request are separate actions from creating local commits; follow the repository owner's instructions about when to publish work.

## Security reports

Follow [SECURITY.md](SECURITY.md). During onboarding, review privacy guidance, session and SQL boundaries, secrets/logging restrictions, threat modeling and the SSDLC repair/retest loop. Security-sensitive changes need reviewer evidence; training and sign-off must be recorded by the operating organization rather than assumed from this document.
