# Shadow Repository Index Benchmark Protocol V0

Status: M4.5 ACCEPTED PRE-IMPLEMENTATION MEASUREMENT CONTRACT
Date: 2026-10-04

## Purpose

Measure whether a persistent lexical repository index is worth implementing before Tokn commits to a backend or active retrieval path.

This protocol compares shadow retrieval mechanics only. It does not measure model-token savings and does not authorize context injection.

## Conditions

The minimum comparison is:
- `DIRECT_SCAN_V0`: bounded sequential reference scan over the eligible corpus;
- one or more indexed candidates using exactly the same corpus and gold queries.

Initial indexed candidates may include:
- `SQLITE_FTS5_UNICODE61_V0`;
- `SQLITE_FTS5_TRIGRAM_V0`.

A candidate name identifies a measurement condition, not a product commitment.

DIRECT_SCAN_V0 accepted reference semantics are executable in `tokn-platform::direct_scan`: bounded Git-backed corpus enumeration, explicit skip reasons, no retained source-text corpus, deterministic all-terms substring matching/ranking and exact current `ixc-v1-*` verification before a candidate is returned. The benchmark policy may choose stricter limits than the implementation hard ceilings (100k files / 16 MiB Git list / 16 MiB per file), but never looser limits.

## Freeze before execution

Every benchmark run must freeze before looking at candidate results:
- repository/corpus identifier;
- exact Git commit when available;
- benchmark run order / interleaving position;
- working-tree state policy;
- versioned corpus eligibility/exclusion policy ID;
- maximum-file-size policy;
- encoding/binary policy;
- versioned query-set ID + gold-set ID and gold relevant document IDs;
- top-k values;
- backend/tokenizer configuration;
- quality floors;
- latency/memory/disk budgets;
- exclusion policy;
- Tokn commit/version.

Do not choose thresholds after seeing which candidate wins.

## Corpus classes

The first pilot should include at least:
1. a small deterministic/sanitized fixture corpus for correctness;
2. the Tokn repository itself at a frozen commit for realistic code structure;
3. a larger frozen public repository only if the first two cannot expose useful latency/size differences.

Do not copy private source repositories into tracked Tokn fixtures.

For public/sanitized results, repository paths should be represented by opaque document IDs or known public fixture IDs.

## Corpus accounting

Record:
- discovered file count;
- eligible/processed file count;
- skipped file count by reason;
- eligible source bytes;
- source bytes actually read;
- Git HEAD/dirty status coverage;
- binary/encoding/size-policy exclusions.

A backend comparison is invalid if candidates did not operate on the same eligible corpus unless the difference is explicitly the subject of the test.

## Query set

Queries should cover file-level repository orientation, including:
- exact symbol/identifier terms;
- snake_case / camelCase identifiers;
- filename/path-like terms;
- multi-term concept phrases;
- substring-oriented identifier fragments;
- terms with multiple plausible files;
- negative/no-match queries.

Each positive query must declare one or more gold relevant document IDs before execution.

The machine-readable record keeps one quality row per unique query. Repeated timing observations for that query belong in `latency_samples_ms`; they are not represented as duplicate query rows. `query_count` therefore counts unique queries, while aggregate `sample_count` counts all timing samples across those queries.

Do not use model-generated relevance labels after seeing results as the primary gold set.

## Quality metrics

Report per query and aggregate:
- hit at K;
- Recall@K;
- first relevant rank;
- reciprocal rank;
- no-match correctness where applicable.

Recommended initial K values: 1, 5 and 10. These are reporting points, not hard product limits.

Do not report BM25/trigram score as relevance confidence.

## Performance metrics

### Build

Record when observable:
- elapsed build/preparation time;
- source bytes read;
- index bytes on disk;
- peak RSS / working set;
- CPU time;
- files processed/skipped.

### Query

For each unique query record:
- repeated elapsed-latency samples;
- result count / quality outcome;
- requested K;
- backend condition.

The configured timing repetitions per query must be frozen before execution and must match the number of recorded latency samples for every query row.

Aggregate at minimum:
- median / p50;
- p95;
- p25/p75 or IQR;
- max only as secondary diagnostic.

Warm/cold cache state must be declared or marked `UNKNOWN`.

### Refresh

Measure separately from full build:
- changed/added/deleted/renamed files;
- files re-read;
- bytes re-read;
- elapsed refresh time;
- peak RSS where observable;
- whether a fallback full verification/rebuild occurred.

## Correctness mutation suite

A candidate cannot advance on latency alone. It must pass deterministic correctness cases:
- tracked file content edit;
- dirty worktree with unchanged HEAD;
- added tracked file;
- non-ignored untracked file;
- file deletion;
- file rename;
- query immediately after refresh;
- hash mismatch discovered during query verification;
- index database deleted and rebuilt;
- Git provenance unavailable/ambiguous fallback behavior.

Expected principle: never serve cached source text whose current file hash has not been verified against the indexed manifest.

The machine-readable correctness suite records every required case separately plus an aggregate `gate_status`. An indexed candidate cannot receive `ELIGIBLE_FOR_IMPLEMENTATION` unless the correctness gate is `PASS`; `NOT_APPLICABLE` is allowed only for a case genuinely outside that condition (for example index deletion/rebuild under `DIRECT_SCAN_V0`).

## Backend capability observations

Record separately:
- FTS5 available/unavailable;
- tokenizer/configuration accepted by the runtime SQLite build;
- contentless/contentless-delete capability where tested;
- integrity-check result where applicable.

Do not dynamically load untrusted SQLite extensions to convert an unavailable capability into a passing result.

`tokn-observe shadow-index-capabilities` is the candidate capability surface. It must keep `extension_loading_attempted=false`, report the fallback as `DIRECT_SCAN_V0`, and reuse the benchmark capability labels rather than inventing a second status vocabulary.

## Resource guardrails

Benchmark tooling must respect the same local safety discipline as development:
- one heavy operation at a time;
- bounded worker count;
- no uncontrolled recursive parallel scan;
- record peak memory where practical;
- abort/report rather than saturating the machine if an operational guardrail is exceeded.

For Rust/Cargo validation in this project, keep `TOKN_CARGO_JOBS=1` during heavy local checks unless deliberately overridden.

## Privacy

Shadow index benchmark artifacts may contain sensitive repository vocabulary even when FTS uses a contentless table.

Rules:
- raw indexes remain local and untracked;
- raw source content is not copied into tracked benchmark results;
- absolute paths are not published;
- project scope keys are never published;
- public result records use sanitized/opaque document IDs;
- only schemas, sanitized manifests and aggregate/per-query metrics that pass the privacy gate may be committed.

## Decision gate

A backend is not selected merely because it is fastest.

Before a run, declare:
- minimum acceptable retrieval-quality floor relative to the direct-scan/gold baseline;
- maximum acceptable index-size budget;
- maximum acceptable build/refresh overhead;
- memory budget where measurable;
- which latency improvement would be material enough to justify persistence/complexity.

Record quality, resource and correctness gate outcomes separately. `ELIGIBLE_FOR_IMPLEMENTATION` is valid only when the predeclared quality/resource flags are true and all three gate outcomes are `PASS`.

After measurement, the outcome is one of:
- `BASELINE_ONLY`: reference measurement, no backend decision;
- `REJECTED`: fails a predeclared correctness/quality/resource gate;
- `ELIGIBLE_FOR_IMPLEMENTATION`: meets every predeclared gate and justifies a bounded implementation experiment;
- `INCONCLUSIVE`: insufficient/unstable evidence.

Do not emit `BEST`, `WINNER` or a token-savings estimate from this protocol.

## Repetition

For deterministic correctness cases, one passing result is sufficient only for deterministic assertions.

For latency/resource comparisons:
- run enough repetitions to report stable median and p95;
- freeze `timing_repetitions_per_query` before execution;
- record benchmark `run_order` / interleaving position;
- record aggregate sample count as the total number of latency observations, not the number of unique queries;
- do not rely on one timing;
- interleave candidate order when cache/OS effects could bias results.

The pilot may start small, but sample size must increase if variance makes the decision gate unstable.

## Machine-readable record

`benchmarks/shadow-index-measurement.schema.json` defines `Tokn ShadowIndexMeasurement V1`.

The record separates:
- corpus identity/accounting plus versioned corpus-policy reference;
- benchmark run order / interleaving position;
- condition/backend identity;
- versioned query-set + gold-set references;
- build/refresh metrics;
- one quality row per unique query plus repeated latency samples;
- aggregate quality/latency with total timing-sample count;
- deterministic correctness-case results and correctness gate;
- capability coverage;
- predeclared quality/resource gates and their outcomes;
- final measurement verdict.

This schema is a benchmark contract only. It is not part of Measurement Contract V1 or Store V2.

## Transition from design to implementation

Current prerequisite state:
1. protocol and schema: **ACCEPTED**;
2. SourceStableId derivation reuse: **RESOLVED** in `tokn-domain::identity`, byte-compatible with existing `src-v1-*`;
3. local shadow cache root: **RESOLVED** under `observer_shadow_index_root()` / safe per-project cache segment;
4. direct-scan reference semantics + sanitized Git fixture: **ACCEPTED REFERENCE FOUNDATION / FULL GATES PASS**;
5. FTS capability probing: **ACCEPTED CAPABILITY FOUNDATION / FULL GATES PASS** with runtime-observed in-memory probes and DIRECT_SCAN fallback;
6. `SQLITE_FTS5_UNICODE61_V0`: **FUNCTIONAL BENCHMARK CANDIDATE / FULL GATES PASS** using an in-memory contentless table over the exact verified DIRECT_SCAN manifest;
7. sanitized pilot harness: **IMPLEMENTED / TARGETED TESTS PASS**. `benchmarks/shadow-index-sanitized-pilot-v1.json` freezes a generated Git corpus, eight gold queries, K={1,5,10}, five timing repetitions and 0.95 quality ratios before execution. `tokn-observe shadow-index-pilot` emits separate `Tokn ShadowIndexMeasurement V1` records. DIRECT_SCAN is `BASELINE_ONLY`; unicode61 fails the frozen quality floor and is `REJECTED` for this condition. Full mutation/refresh correctness, RSS/resource evidence and any other backend candidate remain **NEXT**.

Even after an index backend becomes eligible, active runtime context injection remains a separate future gate.
