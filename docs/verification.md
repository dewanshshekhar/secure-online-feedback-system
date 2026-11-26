# Verification record — 24 November 2026

Implementation baseline: `b598cb1`. Environment: Windows, Rust/Cargo 1.85.1, Trunk 0.21.14, PostgreSQL 17.5, Chromium through the connected Kimi WebBridge. This record describes local evidence; it does not claim GitHub CI or a production deployment ran.

## Automated checks

| Check | Actual result |
| --- | --- |
| Backend `cargo fmt --check` | Passed |
| Backend `cargo clippy --all-targets --locked -- -D warnings` | Passed (latest dependency update resolved before the locked test run) |
| Backend `cargo test --locked` | 18 passed; the opt-in PostgreSQL test is ignored by this command |
| `cargo test --locked --test postgres -- --ignored` with isolated `feedback_test` DB | 1 passed; inserted record survived application recreation; test removed its record |
| Frontend `cargo fmt --check` | Passed |
| Frontend `cargo clippy --target wasm32-unknown-unknown --locked -- -D warnings` | Passed |
| Standalone `trunk build --locked` | Passed; browser loaded the generated WASM/JS/CSS bundle |
| `python scripts/security-check.py` | Passed; bounded source policy checks only |
| Gitleaks 8.30.1, local Git history, redacted output | No leaks found across 48 commits at implementation baseline |
| Trivy 0.76.2, filesystem vulnerability/misconfiguration scan | No HIGH/CRITICAL findings in either Cargo.lock; no infrastructure configuration files detected |
| `git diff --check` | Passed before final documentation commit |

Scanner binaries were obtained from their official release assets and matched the SHA-256 digests published in the release metadata. Scanner reports, downloaded tools and generated local credentials are ignored rather than committed. Trivy scope was application lockfiles; skipped `.local`, `.tools`, build targets and generated frontend bundle. No container image exists to scan. Absence of findings is not proof of absence of vulnerabilities.

## Browser and runtime checks

1. Started the public frontend and PostgreSQL-backed API. Initial review API returned an empty items array; the inbox displayed the genuine empty state with disabled pagination.
2. An empty public message was blocked by native required-field validation. Category choices and message labels appeared in the accessibility tree.
3. Submitted one synthetic browser-verification message. The API returned a UUID receipt; the success panel appeared and the draft was cleared only after the receipt was read.
4. Signed in with the locally generated credential. The submitted record appeared in the admin inbox. Expanding it displayed the full message and receipt.
5. Used script-like text in the synthetic message. It appeared literally in the list/detail; no injected script executed. This is a targeted regression check, not a penetration test.
6. Signed out. Review returned 401 and the inbox content was removed from the page. Tested again with the final fixed-digest credential comparison.
7. Stopped the API and attempted browser sign-in. The UI showed a service-unavailable message and cleared the credential input. Restarted the API; the submitted database record still existed.
8. Checked both public and admin layouts in a real 390px-wide same-origin iframe viewport. Both used one-column layouts and had no horizontal document overflow. Desktop viewport was 1536 CSS pixels wide, also without horizontal overflow. This is responsive browser testing, not physical-device testing.
9. Inspected label association, native radio/button semantics, live status/alert regions and skip-link/focus CSS. Full manual keyboard/screen-reader testing and a formal accessibility audit remain release checks; programmatic focus in an inactive tab did not establish visible keyboard focus.
10. Deleted only the known temporary browser-test UUID. Final review returned HTTP 200 with zero items and `has_more=false`. The disposable persistence test also cleaned its row. No runtime demo records remain. Signed out of the verification browser sessions.

## Screenshots

Normal browser viewport captures; scroll to inspect the complete live pages. Images contain no submitted feedback or credentials.

- [Public page](screenshots/public-desktop.png)
- [Public form fields and send action](screenshots/public-form.png)
- [Administrator sign-in](screenshots/admin-signin.png)
- [Genuine empty administrator inbox](screenshots/admin-empty.png)

## Explicit limits

GitHub-hosted CI, production TLS/CSP/edge limits, institutional identity/MFA, per-reviewer audit, backup/restore, retention/deletion approval, monitoring, penetration testing, physical-device/full accessibility testing and capacity/load testing were not performed. These remain release-owner tasks in [deployment.md](deployment.md) and [security-model.md](security-model.md). The UI is usable locally; this evidence does not approve collection of sensitive institutional feedback.
