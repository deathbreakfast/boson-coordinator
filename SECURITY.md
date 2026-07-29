# Security Policy

## Supported versions

Security fixes are accepted against the latest `main` branch and tagged releases (`0.1.x`) of this repository's crates (`boson-coordinator`).

## Reporting a vulnerability

Please **do not** open a public GitHub issue for security-sensitive reports.

Prefer one of the following:

1. **GitHub Security Advisories** — use [Report a vulnerability](https://github.com/unified-field-dev/boson-coordinator/security/advisories/new) on this repository when available.
2. Contact the maintainers privately via the repository owner listed at https://github.com/unified-field-dev/boson-coordinator.

Include:

- a description of the issue and its impact
- steps to reproduce or a proof of concept when possible
- affected crate names and versions

We will acknowledge receipt as soon as practical and coordinate a fix and disclosure timeline with you.

## Scope

In scope: vulnerabilities in this repository's published crates and documentation that could cause unsafe production defaults, plus CI/supply-chain issues in this repository.

Out of scope: vulnerabilities solely in third-party dependencies unless this project mishandles them in a security-relevant way.

## `/api/boson` and remote HTTP

- **Fail closed by default:** [`BosonState::new`] / [`BosonState::builder`] require admin
  auth unless you explicitly opt into open lab mode (`BOSON_OPEN_LAB_MODE=1`) or call
  `require_admin_auth(false)` on the builder (tests / local dev only). Production hosts should
  install [`StaticTokenAdminAuth`] or a custom [`AdminAuth`] via the builder.
- **Operator checklist:** before exposing `/api/boson` in production, confirm
  `BOSON_OPEN_LAB_MODE` is unset and a verifier is configured on the Axum state.
- **Remote client:** When `SUBSYSTEM_AUTH_HMAC_KEY` is set, `remote-http` attaches
  Soliton-compatible `x-subsystem-auth` over method + path + body. Unset key ⇒ unsigned
  (lab / host-injected auth only).
