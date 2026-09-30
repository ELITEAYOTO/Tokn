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
