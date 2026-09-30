# Privacy

Tokn Observer is local-only by default.

V0.0 does not:
- upload traces;
- call an LLM;
- store prompts in SQLite;
- store raw terminal output in SQLite;
- read Codex auth files.

The normal automatic source is `%USERPROFILE%\.codex\sessions`.
Diagnostic bundles can contain sensitive prompts, outputs, terminal data and paths; they are imported only when explicitly provided.

## Plugin / MCP privacy boundary - 2026-09-30

Tokn remains local-first by default.

Future Codex integration must follow least privilege:
- expose only the minimum tools/data needed for the requested Tokn operation;
- keep secrets, API keys, auth tokens and auth.json out of plugin packages,
  MCP results, evidence folders and logs;
- do not persist creator account/user IDs merely because session metadata exposes them;
- prefer normalized metrics and provenance over raw prompts or raw tool output;
- redact or hash personal filesystem paths when durable history does not need the raw path;
- treat hook/MCP inputs as untrusted and validate them at the Engine boundary;
- require explicit approval for future state-changing/destructive operations.

The initial local Tokn MCP adapter should not require network access.
A future public/remote plugin is a separate deployment mode and would require
its own privacy policy, authentication model and data-retention review.

Official guidance reviewed:
https://developers.openai.com/plugins/guides/security-privacy
https://developers.openai.com/api/docs/guides/agents-api/tools/plugins
