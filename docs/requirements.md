# Feedback portal requirements and acceptance

This change prioritizes a usable student form and an administrator inbox containing only real submissions. No sample records, invented totals, charts, testimonials, or seeded accounts belong in runtime code. Synthetic fixtures are allowed only in isolated tests.

## Acceptance criteria

- Public page: category, message, privacy guidance, validation, pending/error/success states, and a server-issued receipt.
- Administrator page: explicit sign-in, protected paginated inbox, refresh, sign-out, and honest empty/error/loading states.
- Persistence: successful submissions survive application restarts in PostgreSQL; startup fails if the database is unavailable. No silent fallback or record eviction.
- Privacy: collect no identity fields; render feedback as text; never write bodies or credentials to logs or browser storage.
- Access: exchange a server-configured administrative credential for a short-lived HttpOnly session. No credential in URLs, source bundles, or localStorage.
- Verification: backend boundary and access tests, frontend WASM build, keyboard/mobile inspection, live submit-to-inbox check, and documented limitations.

## SSDLC commitment

The supplied images define five principles (security by design, risk assessment, continuous security, security awareness, collaboration), six phases (planning, design, implementation, testing, deployment, maintenance), two framework mappings (NIST SSDF and Microsoft SDL), and automated security gates. These guide the repository documentation and checks; they are not evidence of certification.

Deployment is a separate release decision. Institutional identity/MFA, edge abuse controls, privacy/retention approval, operational monitoring, and independent review require deployment-owner configuration and evidence.
