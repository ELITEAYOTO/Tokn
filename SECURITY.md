# Security and sensitive data

Tokn processes local coding-agent evidence that may contain sensitive project data.
The current V0.1 design is **local-first** and observation-first.

## Current guarantees

- no Tokn network telemetry or cloud upload by default;
- original Codex evidence is never modified;
- raw rollout/trace files are not persisted into the Tokn Store;
- raw tool-result content is not stored durably merely to create identity;
- durable identities/fingerprints are project-scoped and pseudonymized to avoid raw project/path/content strings; this is not a cryptographic-secrecy guarantee against dictionary confirmation by an adversary who obtains the Store;
- repository publication is checked for common secret/private-path patterns and CI scans full Git history with Gitleaks;
- Runner quality-gate child processes use an explicit environment allowlist, retain at most 64 KiB per stdout/stderr stream while draining excess output, redact common secret shapes best-effort before persistence, and are terminated after a fixed 120 s timeout; this is process hardening, not a sandbox or cryptographic secrecy boundary;
- release packages are checked again before they are kept as publishable artifacts.

The Store contains structured historical evidence, not only aggregate counters. Depending on the accepted schema this includes token accounting, runtime/profile evidence, activity metadata, privacy-safe identities/fingerprints, source versions and workspace Git provenance.

## Sensitive local inputs

Treat all of the following as sensitive unless explicitly sanitized:
- Codex rollouts and diagnostic traces;
- prompts and tool outputs;
- project paths and source files;
- local database files;
- auth/config/credential files;
- raw experiment run directories.

Do not publish real rollout, trace or experiment-run evidence. Use synthetic/sanitized fixtures for tests and bug reports.

## Packaging

Release packaging must:
1. start from a clean Git worktree;
2. copy tracked repository files only;
3. add external experiment-kit files only from an explicit whitelist;
4. include release provenance;
5. run `scripts/validation/check-package-privacy.ps1` before the ZIP is kept.

A failed privacy check must delete the generated ZIP.

## Dependency audit status

CI runs pinned `cargo-audit 0.22.2` and `cargo-deny 0.20.2` on every PR/main validation, alongside immutable GitHub Action pins and Dependabot update proposals. Public releases still require release-time dependency/SBOM review; CI dependency checks are necessary controls, not a software-supply-chain attestation.

## Not complete yet

The MCP production entrypoint now opens the existing SQLite Store through `Database::open_read_only`, so that path does not create or migrate the Store. Remote MCP, broader Store lifecycle/retention controls and persistent Shadow indexing remain separate future threat-model gates.

Runner quality-gate hardening bounds the direct child execution and, on Windows, terminates the exact spawned PID tree on timeout. It does not provide OS sandboxing, syscall/filesystem/network isolation, or a guarantee that every possible secret format is redacted.

Before public binary distribution, Tokn still requires a documented threat model plus retention/purge/export policy, dependency/SBOM review, signing/update integrity and a hostile parser/privacy corpus.
These are roadmap gates, not current guarantees.

If reporting a security issue, do not attach real secrets, rollouts or private project files to a public issue. Prefer a private GitHub security-reporting channel when available.
