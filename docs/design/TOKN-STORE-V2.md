# Tokn Store V2

Status: FOUNDATION IMPLEMENTED
Date: 2026-10-01
Store schema version: 2

## Purpose

Tokn Store V2 is the first persistent consumer of the frozen
`tokn.measurement.v0.1` contract.

Its current purpose is deliberately narrow:
- persist project/workspace identity;
- persist runs and agents;
- persist V1 usage summaries without losing coverage counters;
- persist provenance fingerprints;
- persist ModelRuntimeProfile V1;
- preserve idempotence;
- avoid raw personal paths, prompts and secrets by default.

This is a Store foundation, not the complete historical analytics layer.

## Current implementation

Migration:
`crates/tokn-storage/migrations/0002_measurement_store.sql`

Core persistence:
`crates/tokn-storage/src/measurement.rs`

Runner evidence adapter:
`tokn-observe store-evidence`

CLI shape:

`tokn-observe store-evidence <evidence_dir> --project-key <logical-key> --workspace-key <logical-key> [--parent-workspace-key <logical-key>] [--runtime-profile <profile.json>] [--db <path>]`

The Runner contract is not modified.
Store ingestion consumes the immutable evidence folder after Runner completion.

## Tables implemented

### projects_v2

Stores:
- privacy-preserving project_id;
- identity version;
- creation time.

The raw project key is never persisted.

### workspaces_v2

Stores:
- privacy-preserving workspace_id;
- project relationship;
- optional parent workspace relationship;
- optional snapshot fingerprint;
- identity version.

The raw workspace key and workspace filesystem path are never persisted.

### runtime_profiles_v2

Stores:
- stable profile_id derived from the serialized V1 profile;
- schema version;
- runtime/model fields;
- configuration completeness;
- the complete ModelRuntimeProfile V1 JSON.

The profile contract itself rejects unsupported schema versions.

### measurement_runs_v2

Stores:
- run_id;
- project/workspace/profile references;
- measurement contract and evidence-layout versions;
- root thread and terminal state;
- source health;
- validity/quality summaries;
- agent count;
- token totals;
- per-metric known counters;
- logical total only when authoritative.

### agents_v2

Stores per-agent:
- thread_id;
- parent_thread_id;
- depth;
- terminal state/duration;
- token totals;
- per-metric known counters;
- logical total only when authoritative.

Raw agent source paths and cwd are not persisted.

### provenance_v2

Stores:
- privacy-preserving source_id;
- run relationship;
- source kind;
- fingerprint of normalized session evidence;
- normalized evidence byte count;
- adapter name/version.

It intentionally does not store the original runtime path.

### rate_limit_snapshots_v2

The schema exists so the Store does not need another structural migration for the
first rate-limit history work.

M4 ingestion is implemented from directly observed Codex token-count rate-limit snapshots.
The Store retains only limit identity, primary/secondary percentage-window-reset data and reached type. It deliberately excludes credit balances, plan/account identity and limit_name.
No rate-limit values are invented or backfilled.

## Deferred by design

Not yet implemented in the Store foundation:
- Findings persistence;
- Experiment Lab persistence;
- broader historical query/index APIs beyond the accepted Context Ledger, Tool Activity and RateLimitHistory surfaces;
- Context Ledger tables;
- cross-run finding aggregation.

Those belong to later layers after the thin local MCP prototype and/or Historical
Analyzer has a concrete consumer contract.

## Identity and privacy rules

Project and workspace identities come from explicit logical keys supplied at ingestion.

Before persistence:
- project key -> BLAKE3-derived `prj-<hex>`;
- workspace key -> BLAKE3-derived `wsp-<hex>`;
- runtime profile JSON -> BLAKE3-derived `rtp-<hex>`;
- source kind + normalized evidence fingerprint -> BLAKE3-derived `src-<hex>`.

The raw logical keys are not stored.

V2 does not persist:
- full personal filesystem paths;
- raw prompts;
- auth secrets;
- account IDs;
- raw terminal output;
- raw session JSONL.

The legacy V1 import tables previously stored source paths.
Opening an old database through the V2 binary:
1. pseudonymizes those legacy path columns;
2. records Store schema version 2;
3. runs a one-time SQLite `VACUUM` so old path strings do not remain in free pages.

Future Store versions are fail-closed.
A binary that knows schema 2 refuses an unknown future schema instead of silently
downgrading or rewriting it.

## Idempotence

The same Runner evidence can be ingested repeatedly.

Current keys:
- run: `run_id`;
- agent: `run_id + thread_id`;
- project/workspace: privacy-preserving logical IDs;
- runtime profile: serialized V1 profile fingerprint;
- provenance: source fingerprint identity.

Re-ingesting the same evidence updates the same logical rows and does not multiply
runs or agents.

## Coverage semantics

Store V2 persists both totals and known counters:
- input_known;
- cached_input_known;
- cache_write_input_known;
- output_known;
- reasoning_output_known.

Therefore a numeric aggregate does not erase UNKNOWN coverage.

Token semantics remain governed by
`docs/design/MEASUREMENT-CONTRACT-V0.1.md`:
- ordinary uncached = input - cached;
- cache-write remains separate;
- logical total = input + output only when authoritative.

## Golden validation

Experiment 001 golden replay now validates:

1. Runner emits the frozen V1 evidence;
2. all historical golden facts remain unchanged;
3. Store V2 ingests the evidence with a sanitized ModelRuntimeProfile fixture;
4. the exact same evidence can be ingested a second time;
5. project/workspace/source/profile IDs remain deterministic;
6. the SQLite file does not contain the raw golden workspace or session path.

Unit tests additionally validate:
- V2 measurement idempotence;
- raw project IDs are rejected;
- runtime profile round-trip;
- legacy path migration physically removes path strings after VACUUM;
- unknown future Store versions are refused.

## Next boundary

Store foundation is sufficient for the next prototype:

Codex integration -> command-launched local MCP adapter -> shared Rust Engine/Store.

The MCP layer must remain thin.
It must not duplicate Runner, Store or analysis logic.
