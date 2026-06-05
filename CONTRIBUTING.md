# Contributing

Thanks for contributing to Lingxiao AI Manager.

## Ground Rules

- Keep the project local-first and privacy-first.
- Account switching must stay local, user-owned, and backend-only for raw
  sessions.
- Do not add token export, token import, account resale, machine identifier
  reset, unauthorized access, or usage-limit bypass features.
- Do not commit `.env`, local config, databases, logs, screenshots with personal data, packaged binaries, signing certificates, or dependency directories.
- Do not log tokens, cookies, passwords, machine identifiers, or full local database values.
- Keep changes focused and avoid unrelated refactors.

## Development

```bash
cargo check --locked
cargo test --locked
```

For UI-only changes, also open the Tauri app locally and verify the usage and logs tabs still render correctly.

## Pull Request Checklist

- [ ] No secrets or personal data are included.
- [ ] No high-risk token/device bypass behavior was added.
- [ ] Account management changes do not expose raw sessions to the frontend.
- [ ] The app still works without a project-owned backend.
- [ ] `cargo check --locked` passes.
- [ ] Documentation was updated when behavior changed.

## Security Changes

If your change touches authentication state, local account data, logging, update checks, filesystem access, or network requests, include a short security note in the PR description.
