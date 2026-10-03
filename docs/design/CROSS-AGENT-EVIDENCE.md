# Cross-Agent Evidence V2

Status: M4 ACCEPTED FOUNDATION

Date: 2026-10-03

## Purpose

Cross-Agent Evidence V2 identifies exact privacy-safe operation overlap between distinct agents inside the same run and, when evidence is complete, compares privacy-safe result identity.

It answers a narrow question:

> Did multiple agent threads execute the exact same fingerprinted operation?

It can answer whether exact observed result fingerprints are the same or different. It still does **not** conclude redundancy, necessity, waste or savings from identity alone.

## Inputs

The reducer joins two already accepted historical contracts:
- `HistoricalSnapshot V1` for agent lineage (`thread_id`, `parent_thread_id`, `depth`);
- `ToolActivityHistory V2` for privacy-safe exact operation fingerprints plus source/result identity fields and explicit coverage.

Both inputs must use their supported schema versions and the same project/workspace filters. A mismatch fails closed instead of silently joining unrelated evidence.

## Grouping rule

An overlap group is keyed by:
- `run_id`;
- activity `category`;
- exact privacy-safe `operation_fingerprint`.

A group is emitted only when the same operation appears in at least two distinct `thread_id` values in the same run.

The reducer does not merge identical operations across separate runs. Cross-run recurrence is a different observation with different causal requirements.

Repeated operations inside one thread remain ActivityTimeline observations and do not become cross-agent overlap by themselves.

## Lineage relation

For each participating pair Tokn reports one of:
- `DIRECT_PARENT_CHILD`;
- `ANCESTOR_DESCENDANT`;
- `SIBLING`;
- `OTHER_KNOWN_LINEAGE`;
- `UNKNOWN_LINEAGE`.

Missing or broken lineage remains `UNKNOWN_LINEAGE`. Tokn does not infer a parent relationship from depth alone.

## Result identity boundary

ToolActivityHistory V2 separates:
- `source_stable_id`: stable logical source identity when directly observable;
- `content_fingerprint`: project-scoped privacy-safe fingerprint of exact observed result content;
- independent coverage for both identities.

The current Codex adapter computes `content_fingerprint` while raw output is transiently available, then discards the raw output. Durable storage contains only the fingerprint and coverage.

Coverage values are `OBSERVED`, `PARTIAL`, `NOT_CAPTURED` and `UNKNOWN`. An exact result is `OBSERVED` only when one tool result maps unambiguously to one normalized operation. Multi-operation ambiguity stays `NOT_CAPTURED`; unavailable source evidence stays `UNKNOWN`.

Cross-Agent Evidence V2 emits `result_identity_match = SAME` or `DIFFERENT` only when every occurrence has complete observed identity. Otherwise the match is `UNKNOWN`.

Even `SAME` is evidence identity, not an automatic claim of duplicate waste, avoidable tokens or savings. Such interpretation requires later contextual and causal evidence.

## CLI

`tokn-observe cross-agent-evidence` reads the accepted historical Store surfaces and emits JSON.

Filters reuse the privacy-preserving `prj-*` and `wsp-*` identifiers; invalid filters and unsupported source schemas fail closed.

## Privacy boundary

The report contains privacy-safe operation fingerprints and historical thread/run identifiers already present in accepted M4 contracts. It does not persist raw commands, raw workdirs, raw tool outputs, prompts, credentials or account metadata.

## Acceptance

The V1 operation-overlap slice and the V2 result-identity extension both passed publication privacy, rustfmt, strict Clippy, workspace tests, release/provenance, MCP smoke, Experiment 001 golden replay, Experiment 002 regression and documentation consistency on their validation branches.
