# Cross-Run Source Re-read Evidence V0

Status: M4 ACCEPTED CHRONOLOGY FOUNDATION / OBSERVATION-ONLY
Date: 2026-10-03

## Purpose

Describe a narrow directly observed sequence: the same project-scoped `SourceStableId` is read in distinct runs, and the corresponding read windows can be strictly ordered from rollout timestamps.

This primitive is intentionally narrower than a rediscovery finding. A later read of the same logical source does not prove that the model forgot earlier evidence, that the read was unnecessary, that retained context was stale, or that tokens could safely have been saved.

## Evidence source

`tokn-observe source-reread-evidence` reads ToolActivityHistory V3 only.

Eligible activity must be:
- category `file_read`;
- status `completed`;
- `source_identity_coverage = OBSERVED`;
- backed by a project-scoped `SourceStableId`.

No Store migration and no Measurement Contract V1 change are required.

## Timestamp semantics

Cross-run ordering must never use `measurement_runs_v2.created_at_unix` / `run_created_at_unix` as runtime chronology. That field records Store ingestion time.

The reducer uses only `HistoricalToolActivityRecord.observed_at`, which comes from the directly observed rollout event timestamp when the adapter captured it.

RFC3339 timestamps are normalized to UTC epoch milliseconds through the shared runtime-neutral parser in `tokn-domain`. Numeric timezone offsets are therefore compared correctly; raw timestamp strings are never ordered lexically.

Timing coverage is explicit:
- `COMPLETE_OBSERVED`: every eligible read of the source has a valid observed timestamp;
- `PARTIAL`: only part of the eligible reads have timestamps;
- `NOT_CAPTURED`: none of the eligible reads have timestamps;
- `INVALID_OBSERVED_TIMESTAMP`: a timestamp value was present but cannot be parsed;
- `UNKNOWN`: reserved fail-closed state.

Only `COMPLETE_OBSERVED` can produce an ordered cross-run re-read chain.

## Strict run ordering

For each `(project_id, SourceStableId)` pair, Tokn groups eligible reads by run and computes the directly observed read interval of each run:
- first read timestamp;
- last read timestamp.

A source gets `STRICT_RUN_ORDER_OBSERVED` only when:
1. it appears in at least two distinct runs;
2. every eligible read has a valid timestamp;
3. each run maps to one observed workspace id in the selected history;
4. after ordering run intervals by first read, every previous last-read timestamp is strictly earlier than the next first-read timestamp.

If two run read windows overlap or touch, chronology becomes `OVERLAP_OR_EQUAL_OBSERVED` and no directional re-read transition is emitted.

Missing/partial/invalid timestamp evidence keeps chronology `UNKNOWN`.

## Re-read transitions

Under a complete strict run order, Tokn emits only transitions between consecutive observed run intervals for that source.

Each transition contains:
- earlier/later run ids;
- earlier/later workspace ids;
- the earlier run's last observed read time;
- the later run's first observed read time;
- exact boundary-content relation when directly provable.

This is enough to say: **the same stable source was read again in a distinct later run under directly observed chronology**.

It is not enough to say why.

## Boundary content relation

The content relation compares only the exact boundary reads:
- all eligible reads at the earlier run's last timestamp;
- all eligible reads at the later run's first timestamp.

It reports:
- `EXACT_EQUALITY_OBSERVED` only when every boundary read has complete observed content identity and both sides reduce to one identical `cnt-v1-*` fingerprint;
- `EXACT_DIFFERENCE_OBSERVED` when both boundary identities are complete and differ;
- `UNKNOWN` otherwise.

ContentFingerprint remains distinct from SourceVersionFingerprint and is never compared across those identity domains.

## Project/workspace boundary

`SourceStableId` is project-scoped. The reducer groups by both `project_id` and `SourceStableId`, so a coincidental source-id string from another project is never joined.

Different workspace ids inside the same project are allowed. This supports the already accepted stable-source identity across workspace clones while preserving the workspace ids in the observation.

## Machine-readable non-claims

Every emitted transition keeps:

- `rediscovery_status = NOT_PROVEN`;
- `redundancy_status = NOT_PROVEN`;
- `freshness_status = NOT_PROVEN`.

These fields are deliberate semantic guardrails.

A later exact-equal read does not prove:
- that the model forgot the prior read;
- that the prior content was still present in retained context;
- that the later read was avoidable;
- that caching/reuse would have been correct;
- token waste or savings.

A later different-content read does not by itself prove:
- that the source changed between runs because of a particular mutation;
- that prior context was stale at the moment it was used;
- correct invalidation behavior.

Those stronger interpretations require additional directly observed evidence.

## Shared time primitive

The RFC3339-to-epoch parser previously lived inside the Codex adapter even though its semantics are provider-neutral. V0 moves that unchanged parser into `tokn-domain` and keeps the Codex session module as a thin re-export.

The parser remains dependency-free and covered for:
- `Z` timestamps;
- numeric timezone offsets;
- fractional seconds;
- invalid calendar/time values;
- malformed non-ASCII UTF-8 inputs returning `None` without panicking.

## Acceptance gate

This foundation was promoted to ACCEPTED only after targeted reducer/CLI tests, the unchanged Codex adapter suite, strict Clippy, a canonical `scripts/dev-check.ps1 -Full` with `TOKN_CARGO_JOBS=1`, repository privacy/documentation gates, and a real command smoke all passed.

## Next boundary

This primitive may later support a rediscovery finding only when combined with stronger evidence about what context was delivered/retained and why a later read occurred.

Runtime task delivery remains `NOT_PROVEN`; detailed compaction remains `NOT_CAPTURED`; neither is inferred from cross-run source re-read chronology.
