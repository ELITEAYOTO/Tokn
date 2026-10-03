# Security and sensitive data

Tokn processes local coding-agent evidence that may contain sensitive project data.
The current V0.1 design is **local-first** and observation-first.

## Current guarantees

- no Tokn network telemetry or cloud upload by default;
- original Codex evidence is never modified;
- raw rollout/trace files are not persisted into the Tokn Store;
- raw tool-result content is not stored durably merely to create identity;
- durable identities/fingerprints are project-scoped and privacy-safe where the current contract requires it;
- repository publication is checked for common secret/private-path patterns;
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

RustSec `cargo audit` passed on 2026-10-03 for the current `Cargo.lock` (95 dependencies). This is a point-in-time check only; public releases still require release-time dependency/SBOM review.

## Not complete yet

Before public binary distribution, Tokn still requires a documented threat model plus retention/purge/export policy, dependency/SBOM review, signing/update integrity and a hostile parser/privacy corpus.
These are roadmap gates, not current guarantees.

If reporting a security issue, do not attach real secrets, rollouts or private project files to a public issue. Prefer a private GitHub security-reporting channel when available.
