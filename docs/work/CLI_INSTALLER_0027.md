# CLI INSTALLER 0027 — Commit and Push CLI 1.0.18

Status: Prepared for commit and push.

## Reviewed source changes

The authoritative CLI version is 1.0.18 in codex-rs/Cargo.toml. The reviewed source changes are:

- codex-rs/Cargo.toml: workspace version 1.0.17 to 1.0.18.
- codex-rs/Cargo.lock: workspace package versions reconciled to 1.0.18; unrelated dependency versions preserved.
- codex-rs/cli/tests/version.rs: expected CLI version banner updated to 1.0.18.
- codex-rs/codex-client/src/telemetry.rs: default-compatible RequestTelemetry on_request_start hook.
- codex-rs/codex-api/src/telemetry.rs: invokes request-start telemetry before dispatch.

These changes are intended for the Linux production build source.

## Explicitly excluded

The following remain uncommitted:

- Tracked installer deletions caused by relocating installer outside the repository.
- The external installer tree at C:\ai\anzoth-cli\installer, which is not tracked by this repository.
- Existing installer/offline recovery files.
- Existing release artifacts and releases directory.
- macOS installer recovery files and scripts.
- Prior uncommitted work ledgers and reports not required for this commit.

The installer relocation remains a separate unresolved CI/source-ownership issue. The current GitHub workflow still expects installer/windows inside the repository.

## Safety checks

- No Cargo command was executed.
- No Linux build script was executed.
- No installer or packaging build was executed.
- No release artifact was modified.
- No reset, rebase, checkout, pull, fetch, clean, or branch switch was performed.
- Current branch remains anzoth-rebrand.
- Remote-tracking branch was checked read-only before push.

## Commit intent

Commit only the five source files above plus this ledger. Push to origin/anzoth-rebrand without force. If the remote rejects the push because it has diverged, stop without pulling or merging.

CLI INSTALLER 0027
