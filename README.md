# Lingxiao AI Manager

Lingxiao AI Manager is a local-first desktop utility for AI IDE account
management and usage visibility. The public build focuses on user-owned Cursor
accounts: it can keep a private local account list, switch the local Cursor
session by account index, and show usage statistics such as model calls, token
consumption, quota status, and estimated cost.

## Why This Exists

- Faster developer workflow when working with several personal or work accounts.
- Local-first storage: saved sessions stay on the user's machine.
- Usage visibility: model, token, quota, and cost data are easier to review.
- Open-source transparency: local storage, switching, API requests, and log
  redaction are inspectable.
- Privacy-aware defaults: the frontend never receives raw session values.

## Current Features

- Detect the locally signed-in Cursor account.
- Add user-owned Cursor sessions to a private local account store.
- List saved accounts with labels, status, and redacted account hints.
- Switch the local Cursor account by index without exposing the token to the UI.
- Show account type, quota, recent usage summary, and per-model usage details.
- Show a redacted frontend log stream.

## Open-Source Safety Boundary

The repository intentionally does not include:

- Real account files, logs, local databases, screenshots with private data, or
  packaged binaries.
- Scripts that obtain, print, export, or sell third-party account tokens.
- Hardcoded third-party OAuth client secrets.
- Machine identifier reset, device-identity rewriting, or limit-bypass wording.
- Private download/update infrastructure.

Users are responsible for adding only accounts they own or are authorized to
use. Account sessions are sensitive credentials and should be handled like
passwords.

## Requirements

- Rust stable
- Tauri CLI v2
- Windows, macOS, or Linux
- Cursor installed locally

## Run Locally

```bash
cargo install tauri-cli --version "^2"
cargo tauri dev
```

## Build

```bash
cargo tauri build
```

The static UI is checked in under `ui/dist`; there is no separate frontend build
step for the current public build.

## Verification

Before publishing, the public snapshot should pass:

```bash
cargo fmt -- --check
cargo check --locked
cargo test --locked
```

Recommended release checks also include secret scanning, dependency audit,
binary/log/database artifact scanning, and a quick manual launch of the Tauri
app.

## Privacy Principles

- No project-owned telemetry is enabled by default.
- Saved account sessions are stored in the user's local config directory.
- Raw sessions are not returned to the frontend account list.
- Usage requests are triggered locally by the user's machine.
- Logs must not contain sessions, cookies, passwords, local database values, or
  machine identifiers.
- Issues and screenshots should be scrubbed before being posted publicly.

See [PRIVACY.md](./PRIVACY.md) and [SECURITY.md](./SECURITY.md).

## Disclaimer

This project is not an official Cursor product and is not endorsed by Cursor.
Users are responsible for following applicable terms, licenses, and laws. This
tool is intended for personal productivity, local account organization, and
usage visibility. It must not be used for unauthorized account access, account
resale, billing bypass, or other improper activity.

## Roadmap

- Better account-store encryption options.
- More robust cross-platform path detection.
- Safer account import flows that never print secrets.
- More redaction tests and UI reliability tests.
- Optional offline reporting views.

## License

MIT. See [LICENSE](./LICENSE).
