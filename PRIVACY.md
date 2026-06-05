# Privacy

Lingxiao AI Manager is designed as a local-first desktop tool.

## Data the App Reads

The app may read local Cursor account state from the user's machine to detect
whether Cursor is logged in and to request usage statistics. This can include an
in-memory access token needed to call Cursor endpoints.

## Data the App Stores Locally

If the user adds accounts in the Accounts tab, the app stores those user-owned
session values in the user's local config directory. The account list shown in
the frontend only receives labels, status, email or redacted hints, and an
account index. Raw sessions are not returned to the frontend list.

## Data the App Does Not Export

The open-source version does not intentionally export:

- Refresh tokens.
- Cookies.
- Machine identifiers.
- Email passwords or Cursor passwords.
- Local databases.
- Account inventory data to a project-owned backend.

## Network Access

Usage statistics are requested from Cursor over HTTPS when the user opens or refreshes the usage view. The app does not send account data to a project-owned server by default.

The open-source build does not include a project-owned automatic update check.

## Logs

Logs should only contain operational status. They must not include token values, cookie values, password values, database connection strings, or local machine identifiers.

## User Responsibility

Before sharing logs, screenshots, crash reports, or issues, remove personal information such as emails, paths, account details, and usage records that you do not want public.
