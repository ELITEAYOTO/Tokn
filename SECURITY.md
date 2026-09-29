# Security and sensitive data

Tokn Observer processes local Codex evidence that may contain sensitive data.

V0.0 principles:
- no network telemetry;
- no cloud upload;
- no automatic reading of auth.json, config.toml, attachments or sandbox secrets;
- aggregate metrics only in SQLite;
- original Codex evidence is never modified;
- diagnostic trace bundles should be treated as sensitive local artifacts.

Do not publish real rollout or trace files. Use synthetic fixtures for tests and bug reports.
