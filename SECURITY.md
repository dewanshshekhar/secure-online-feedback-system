# Security reporting

Do not publish feedback content, credentials, connection strings or working exploit details in public issues. Contact a repository maintainer through an existing private trusted channel to agree on a secure reporting method. A dedicated monitored security address and formal response SLA have not yet been configured; this is a production release blocker.

Provide the affected commit/version, a minimal reproduction using synthetic data, expected behavior, observed impact and any proposed mitigation. Maintainers should acknowledge, triage, contain, repair, add a regression test, retest and coordinate disclosure. Never request real student feedback to reproduce an issue.

Review [the security model](docs/security-model.md), [SSDLC](docs/ssdlc.md) and [deployment response procedure](docs/deployment.md). A working local portal is not evidence that all production controls are in place.
