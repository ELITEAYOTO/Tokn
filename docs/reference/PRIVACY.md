# Privacy

Tokn is local-first by default. Current V0.1 does not upload Tokn evidence or call an LLM on its own.

## Current durable boundary

Tokn may read explicitly supported local coding-agent evidence and persist structured historical evidence in its local Store. Durable data may include token accounting, runtime/profile evidence, activity metadata, project-scoped identities/fingerprints, source-version evidence, workspace Git provenance and project-scoped task-artifact fingerprints when an explicit task file is supplied.

Tokn does not persist raw rollout/trace files or raw terminal/tool-result content merely for convenience or identity generation. `store-evidence --task-input` reads task bytes transiently and stores only a project-scoped/domain-separated `tsk-v1-*` fingerprint plus byte length; it does not store the task text or path. Raw prompts, source files, auth credentials and private runtime evidence remain outside the Store unless a future explicit contract says otherwise.

The normal Codex session source is `%USERPROFILE%\.codex\sessions`. Diagnostic bundles can contain sensitive prompts, outputs, terminal data and paths; they are imported only when explicitly provided.

## Publication boundary

- real rollouts/traces and benchmark runs stay local;
- repository publication scans tracked files for common secret/private-path patterns;
- benchmark raw/run directories are ignored and forbidden from publication paths;
- release packaging starts from tracked Git files plus an explicit external whitelist;
- package text is rescanned before the ZIP is kept;
- failed package privacy validation deletes the generated ZIP.

## Plugin / MCP boundary

Tokn remains local-first by default. Runtime/plugin integrations must follow least privilege:
- expose only the minimum data needed for the requested Tokn operation;
- keep secrets, API keys, auth tokens and `auth.json` out of packages, MCP results, evidence folders and logs;
- do not persist account/user IDs merely because session metadata exposes them;
- prefer normalized evidence and privacy-safe provenance over raw prompts/tool output;
- treat hook/MCP inputs as untrusted and validate them at the Engine boundary;
- require explicit approval for future state-changing/destructive operations.

The current local MCP adapter does not require network access. A future public/remote mode requires its own privacy, authentication and retention review.

Before public binary distribution, the remaining gates include a threat model, retention/purge/export policy, hostile parser/privacy corpus, dependency/SBOM review and signing/update integrity.
