# Cross-Agent Evidence V1

Status: M4 ACCEPTED FOUNDATION

Date: 2026-10-03

## Purpose

Cross-Agent Evidence V1 identifies exact privacy-safe operation overlap between distinct agents inside the same run.

It answers a narrow question:

> Did multiple agent threads execute the exact same fingerprinted operation?

It does **not** answer whether the returned evidence was identical, redundant, unnecessary or wasteful.

## Inputs

The reducer joins two already accepted historical contracts:
- `HistoricalSnapshot V1` for agent lineage (`thread_id`, `parent_thread_id`, `depth`);
- `ToolActivityHistory V1` for privacy-safe exact operation fingerprints.

Both inputs must use their supported V1 schema and the same project/workspace filters. A mismatch fails closed instead of silently joining unrelated evidence.

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

Every V1 overlap explicitly carries:

`result_identity_coverage = NOT_CAPTURED`

Current ToolActivityHistory stores result sizes and counters but not a stable privacy-safe identity of the returned tool content.

Therefore V1 must not relabel an operation overlap as:
- duplicate evidence;
- repeated context;
- wasted work;
- avoidable tokens;
- estimated savings.

Two agents can run the same command and legitimately receive different results because repository state, time, environment or previous edits changed.

Actual duplicate-evidence classification requires a future evidence contract proving result identity or equivalent directly observed semantics.

## CLI

`tokn-observe cross-agent-evidence` reads the accepted historical Store surfaces and emits JSON.

Filters reuse the privacy-preserving `prj-*` and `wsp-*` identifiers; invalid filters and unsupported source schemas fail closed.

## Privacy boundary

The report contains privacy-safe operation fingerprints and historical thread/run identifiers already present in accepted M4 contracts. It does not persist raw commands, raw workdirs, raw tool outputs, prompts, credentials or account metadata.

## Acceptance

Validation branch `m4-cross-agent-evidence` passed publication privacy, rustfmt, strict Clippy, workspace tests, release/provenance, MCP smoke, Experiment 001 golden replay, Experiment 002 regression and documentation consistency before merge.
