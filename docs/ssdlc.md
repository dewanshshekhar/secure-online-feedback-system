# Secure Software Development Lifecycle (SSDLC)

The four supplied images are the source for this project's lifecycle structure: principles, phases, framework families, and the automated security feedback loop. This document maps that structure to actual repository evidence and explicitly identifies deployment-owner work. A mapping is not NIST or Microsoft certification.

## Five principles

| Image principle | Project practice and evidence | Responsibility |
| --- | --- | --- |
| Security by Design | Minimal category/message contract; parameterized SQL; authenticated admin sessions; no browser persistence of credentials; `forbid(unsafe_code)` | Implementer and security reviewer |
| Risk Assessment | Threats and release blockers in [security-model.md](security-model.md); re-evaluate when adding identity, exports, notifications or attachments | Security/privacy reviewer |
| Continuous Security | Pull-request and weekly [security workflow](../.github/workflows/security.yml), locked dependencies, regression tests, scan findings fixed and retested before release | Maintainer |
| Security Awareness | Contributor onboarding covers sensitive feedback, credential handling, safe rendering, reporting, and limitations of anonymity | Maintainer and all contributors |
| Collaboration | Focused commits, review descriptions with evidence, separate security/privacy and deployment sign-off | Author, reviewer, deployment owner |

## Six phases and exit gates

| Phase from images | Activities | Evidence / exit condition |
| --- | --- | --- |
| Planning | Define public submission and admin review, non-functional privacy requirements, no seeded data, actor boundaries | [requirements.md](requirements.md); owner accepts scope and release exclusions |
| Design | Trace browser → same-origin API → PostgreSQL; separate public writes and authenticated reads; assess access, retention, abuse and recovery risks | [security-model.md](security-model.md); reviewers approve residual risks |
| Implementation | Rust modules, strict validation, short-lived cookies, origin checks, bound requests/queries, server-issued receipts, text-only feedback rendering | Source diff, lockfiles, no raw HTML or embedded secrets; code review |
| Testing | Unit/API negative tests, session lifecycle, isolated PostgreSQL persistence, WASM build, manual public-to-admin/browser/keyboard checks | [verification.md](verification.md) records actual results; automated checks pass |
| Deployment | Build immutable artifacts, review scanner results, configure HTTPS/identity, least-privilege DB role, backups, CSP, edge limits and rollback | [deployment.md](deployment.md); explicit release approval, never inferred from local tests |
| Maintenance | Monitor redacted operational signals, review advisories, patch dependencies, run restore tests, execute retention policy, handle incidents | Operational evidence and issue resolution records maintained by deployment owner |

## Framework mapping from image 3

NIST [SSDF v1.1, SP 800-218](https://doi.org/10.6028/NIST.SP.800-218) groups practices into four families:

| NIST SSDF family | Application here |
| --- | --- |
| Prepare the Organization (PO) | Requirements, responsibilities, contributor awareness, required security checks and explicit release criteria |
| Protect the Software (PS) | Review source changes, protect secrets and branch access, lock dependencies; production artifact provenance and signing remain release-owner tasks |
| Produce Well-Secured Software (PW) | Threat assessment, minimal data collection, parameterized storage, controlled sessions, boundary/security tests and safe UI rendering |
| Respond to Vulnerabilities (RV) | Private reporting, triage, containment, rotation, regression tests, root-cause notes and follow-up maintenance |

Microsoft [SDL practices](https://www.microsoft.com/en-us/securityengineering/sdl/practices) include the five topics in the image:

| Microsoft SDL topic | Application here |
| --- | --- |
| Security Training | Onboard contributors on access, privacy, logs, secrets, supply-chain review and incident escalation; record actual training before a real deployment |
| Threat Modeling | Review trust boundaries and attack scenarios in the security model before features alter the data flow |
| Secure Coding | Validation, SQL bind parameters, no unsafe code or raw user HTML, safe error messages, least privilege |
| Security Testing | API rejection, unauthorized access, login origin/expiry/logout, database durability, rendering/keyboard checks |
| Vulnerability Management | Weekly scans, severity assessment, fix/retest, review and patch deadlines set by the operating organization |

## Automation and feedback loop from image 4

```text
Developer → Git repository → CI security gates
  ├─ SAST: compiler/Clippy + bounded source policy checks
  ├─ Dependency scanning: Trivy against manifests and lockfiles
  ├─ Secrets scanning: Gitleaks against Git history
  ├─ Security tests: validation, authentication, sessions, database persistence
  └─ Container/IaC: filesystem misconfiguration scan; actual deployment artifacts pending
       ↓
Checks pass? ── No → report findings → fix → retest → repeat gates
       │ Yes
       ↓
Release review → secure deployment → continuous monitoring → security feedback
       └───────────────────── new requirements / patches / regression tests
```

The workflow has read-only repository permissions and no deployment credentials or publishing step. Scanner failures block their jobs; local checks do not imply remote CI has run. Set required checks and branch protection in repository settings. Manual review and penetration testing complement the narrow source policy rules; these rules are not a comprehensive vulnerability detector.

The container/IaC stage must scan the actual built image and infrastructure once those are introduced. No container, infrastructure, monitoring integration or secure deployment is fabricated to make the diagram appear complete. Release evidence must include artifact hashes, scanner versions/results, reviewer decisions, and any approved risk acceptance with an expiry date.

## Reporting and response

Use [SECURITY.md](../SECURITY.md) for private reporting guidance. Triage public impact without copying feedback content into issues. Record the affected version, severity, owner, remediation plan, regression test, reviewer and retest evidence. Treat credential exposure or unauthorized feedback access as urgent: restrict access, revoke sessions/rotate secrets, preserve redacted evidence, assess notification requirements, then repair and retest. Feed lessons into planning and contributor awareness.
