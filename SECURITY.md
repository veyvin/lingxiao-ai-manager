# Security Policy

## Supported Versions

The open-source `main` branch is the only supported version for security fixes.

## Reporting a Vulnerability

Please do not publish secrets, tokens, cookies, logs, screenshots, database files, or proof-of-concept data in a public issue.

Report security issues privately to the repository owner through GitHub's private vulnerability reporting when available. If private reporting is unavailable, open a minimal issue that says a private security report is needed and avoid including sensitive details.

## Scope

Security reports are welcome for:

- Token, cookie, or machine identifier leakage.
- Logs or UI surfaces exposing sensitive data.
- Unsafe filesystem access.
- Dependency vulnerabilities reachable from the app.
- Build or release steps that could publish private artifacts.

Reports requesting token export, machine identifier reset, account resale,
unauthorized access, or usage-limit bypass functionality are out of scope and
will be rejected. Account switching issues are in scope when they involve
user-owned local accounts and do not expose raw sessions to the frontend or
logs.

## Maintainer Expectations

- Rotate any accidentally exposed credentials immediately.
- Remove sensitive files from the working tree before publishing.
- Prefer a fresh open-source snapshot when historical commits may contain secrets.
- Keep `.env`, local config, databases, logs, packages, signing certificates, and build outputs out of commits.
