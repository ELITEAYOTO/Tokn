# Tokn Historical Context Ledger

Status: M4 IN PROGRESS / CONTEXT LEDGER V1 CORE ACCEPTED
Date: 2026-10-01

## Purpose

Provide observation-only historical context accounting from persisted Tokn evidence
without changing Measurement Contract V1 and without inventing unavailable detail.

The current path is:

Runner V1 evidence
-> Store V2
-> HistoricalSnapshot V1
-> shared Rust Context Ledger V1
-> CLI and thin read-only MCP surfaces.

## Accepted V1 core

HistoricalSnapshot V1 loads:
- filtered project/workspace runs ;
- run-level usage and validity/quality/terminal state ;
- agent history and parent relationships ;
- runtime profiles ;
- provenance ;
- workspace ancestry.
Context Ledger V1 adds:
- input / cached input / cache-write / output / reasoning ;
- per-metric observed-token totals plus known-record coverage ;
- ordinary uncached input only when input and cached coverage are complete ;
- logical total only when authoritative ;
- workspace lineage ;
- integrity issues for mismatches instead of silent repair.

Coverage states:
- COMPLETE ;
- PARTIAL ;
- NO_EVIDENCE ;
- NOT_CAPTURED ;
- UNKNOWN.

## Hard semantic boundaries

Measurement Contract V1 persistence does not provide authoritative per-turn usage
history to the Store. Therefore:
- turn granularity is NOT_CAPTURED ;
- run/agent aggregates must not be redistributed into invented turns.

Tokn also does not have an authoritative current retained-context occupancy metric.
Therefore:
- current retained-context tokens remain absent ;
- retained-context status remains UNKNOWN ;
- model context window, total usage and last usage are not substitutes.
UNKNOWN is not zero.
NOT_CAPTURED is not NO_EVIDENCE.
NO_EVIDENCE is not PASS.

## CLI

Command:
`tokn-observe context-ledger`

Optional filters:
- `--project-id prj-<24 hex>` ;
- `--workspace-id wsp-<24 hex>` ;
- `--limit <1..1000>` (default 50) ;
- `--db <path>` for explicit local Store selection ;
- `--output-json <path>`.

Project/workspace filters accept only privacy-preserving Store IDs, never raw paths.

## MCP

Tool:
`tokn_context_ledger`

Properties:
- read-only ;
- optional project/workspace privacy-preserving filters ;
- limit 1..100, default 20 ;
- delegates Store query + ledger construction to shared Rust crates ;
- does not duplicate token accounting or SQL schema logic.
Validated with:
- standalone release stdio smoke ;
- target `codex-cli 0.161.0-alpha.2` ;
- isolated disposable `CODEX_HOME` ;
- app-server tool discovery ;
- direct `mcpServer/tool/call` ;
- ephemeral idle thread with zero model turns ;
- no copied or inspected user authentication material.

## Integrity rules

The builder reports, rather than repairs:
- persisted logical-total mismatch ;
- run agent-count mismatch ;
- missing root agent ;
- missing parent agent ;
- run totals differing from loaded agent totals ;
- workspace lineage cycle ;
- workspace/project mismatch ;
- missing runtime profile ;
- missing provenance.

These issues remain evidence for later diagnosis.

## Privacy
The historical ledger uses pseudonymized Store identities.
The MCP response must not expose:
- local database path ;
- raw prompts ;
- raw session JSONL ;
- raw cwd/personal path ;
- secrets or authentication material.

Future activity history must preserve the same rule.
If file/tool identity is needed for repeated-read analysis, use an explicitly designed
privacy-preserving identity instead of persisting raw personal paths by default.

## M4 evidence status and remaining work

Before M4 can be marked DONE:
- rate-limit snapshots over time where supported : ACCEPTED FOUNDATION ;
- tool/file activity timeline : ACCEPTED FOUNDATION ;
- phase timeline from direct evidence ;
- parent/subagent exact-operation overlap : ACCEPTED FOUNDATION ;
- repeated reads/searches/retries : exact-repeat foundation ACCEPTED ;
- compaction observations where directly available ;
- duplicate-evidence finding remains deferred even when exact result identity is observed, because identity alone does not prove redundancy or waste ;
- rediscovery across runs ;
- explicit cross-run comparison primitives.

Per-turn history requires a future directly evidenced contract if the product truly
needs it. It must not be backfilled heuristically into V1.

M5 Findings starts only after these historical observations are sufficiently reliable.

## Accepted Tool Activity V1 slice - 2026-10-02

ToolActivityHistory V1 is now an additive historical contract for M4.

It persists only analysis-safe metadata and privacy-preserving fingerprints. It does not persist raw command text, raw workdir/path text, raw tool output or parse-error text.

ActivityTimeline V1:
- orders observations deterministically inside each agent;
- reports source-sequence coverage when available;
- does not invent a global parent/subagent order;
- groups exact repeated operation fingerprints with occurrence/run/thread counts;
- emits no efficiency score, savings claim or optimization recommendation.

CLI: `tokn-observe activity-timeline`.

## Accepted RateLimitHistory V1 slice - 2026-10-03

Rate-limit snapshots are ingested only when directly present in Codex token-count evidence.

Retained evidence is deliberately minimal: observation timestamp, limit_id, primary/secondary used percentage, window duration, reset timestamp and reached type. Tokn does not retain limit_name, credit balance, plan/account identity or other monetary/account metadata in this contract.

Re-ingestion replaces the run-owned snapshot set deterministically, duplicate agent copies are collapsed, and missing telemetry remains absent rather than being inferred.

CLI: `tokn-observe rate-limit-history`.

Automatic runtime compatibility, rate-limit history, Cross-Agent Evidence V2, exact result identity, conservative SourceStableId file V0, mutation-operation timing, run-boundary SourceVersionHistory V1 and Workspace Git Provenance V0 are accepted foundations. Remaining M4 work is fail-closed freshness/invalidation and verified mutation effect where provable, broader source kinds, rediscovery/compaction and explicit cross-run analysis, not per-turn token invention.

## Accepted Cross-Agent Evidence V2 slice - 2026-10-03

Cross-Agent Evidence V2 joins HistoricalSnapshot V1 lineage with ToolActivityHistory V3 exact privacy-safe operation fingerprints and result-identity coverage.

It emits same-run overlap only when the same operation fingerprint appears in at least two distinct threads. Same-thread repeats remain ActivityTimeline evidence, and identical operations across separate runs are not merged into one cross-agent observation.

Lineage is reported as direct parent/child, ancestor/descendant, sibling, other known lineage or unknown lineage. Broken or missing lineage remains UNKNOWN instead of being inferred from depth.

When every overlap occurrence has directly observed exact result identity, Cross-Agent Evidence V2 can report `SAME` or `DIFFERENT`. Partial, ambiguous or unavailable identity stays `PARTIAL`, `NOT_CAPTURED` or `UNKNOWN`; identity alone never becomes a savings/waste claim.

CLI: `tokn-observe cross-agent-evidence`.

## Accepted Source Identity + Content Evolution V0 slice - 2026-10-03

For conservative single-file `Get-Content` observations, the Codex adapter can resolve a workspace-relative logical locator under the Runner selected workspace and project-scope it into `src-v1-*` without persisting the raw locator. The same relative source across project clones therefore keeps one logical identity while another project scope receives another identity.

`SourceIdentityHistory V1` groups those stable sources and reports occurrence/run/thread counts plus exact content-identity coverage. `UNCHANGED_OBSERVED` means every captured occurrence had complete exact identity and one fingerprint; `CHANGED_OBSERVED` means complete exact identity contained multiple fingerprints; incomplete evidence stays `UNKNOWN`. Neither value claims freshness, retained context, unnecessary rereads or safe memory reuse.

CLI: `tokn-observe source-identity-history`.

## Accepted Source Mutation Observation V0 slice - 2026-10-03

ToolActivityHistory V3 adds optional `observed_at` evidence copied from the directly observed Codex rollout response-item timestamp. Missing timing remains absent and is reported as `NOT_CAPTURED`.

For conservative single-target `Set-Content` / `Add-Content` operations, Tokn can resolve the same project-scoped SourceStableId used by file reads when the target is literal, unambiguous and inside the selected workspace.

SourceMutationHistory V1 emits the run/thread/source identity, observed timing coverage and tool status. It deliberately reports `effect_status=NOT_VERIFIED`: a completed tool call is not proof that durable file content changed.

CLI: `tokn-observe source-mutation-history`. No freshness, staleness, invalidation effect, safe reuse or savings claim follows from this operation evidence alone.

## Accepted Run-Boundary Source Version V0 slice - 2026-10-03

Runner workspace BEFORE/AFTER snapshots directly expose per-file relative path, SHA-256 and byte count. Tokn reuses the workspace-relative logical locator to derive the same project-scoped SourceStableId, then derives a dedicated project-scoped `ver-v1-*` fingerprint from the observed snapshot SHA-256. Raw paths and raw SHA-256 values are not persisted.

`SourceVersionHistory V1` stores at most one BEFORE and one AFTER record per `(run, source)`; the Store and SQLite both reject duplicates. Missing a boundary remains explicit rather than being interpreted as addition or deletion.

The analysis reducer reports `UNCHANGED_OBSERVED` only when both exact boundary fingerprints exist and match, `CHANGED_OBSERVED` when both exist and differ, otherwise `UNKNOWN`. CLI: `tokn-observe source-version-history`. This proves source-state difference across the run boundary only; it does not prove which operation caused it, source-specific mutation causality, freshness/staleness, invalidation effect or safe reuse.

## Accepted Workspace Git Provenance V0 slice - 2026-10-03

ProjectSnapshot V1 now carries an optional Git evidence block at Runner BEFORE/AFTER boundaries. `OBSERVED` requires a directly resolved HEAD plus working-tree dirty state; failed/non-Git observation is `UNKNOWN`, while legacy snapshots with no Git block remain `NOT_CAPTURED`.

`workspace_git_provenance_v1` stores at most one record per `(run, boundary)`. Durable evidence contains coverage, dirty state and a project-scoped `git-v1-*` HEAD fingerprint; the raw Git SHA is not persisted in SQLite.

CLI: `tokn-observe workspace-git-provenance-history`. This is workspace-level provenance only: it does not identify which source changed, prove mutation effect, emit freshness/staleness, or authorize invalidation/reuse/savings claims.
