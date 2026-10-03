# Source Freshness Evidence V0

Status: M4 ACCEPTED CORROBORATION FOUNDATION / OBSERVATION-ONLY
Date: 2026-10-03

## Purpose

Join already accepted source chronology, run-boundary version evidence and workspace Git provenance without inventing a `FRESH` / `STALE` verdict.

This slice is deliberately a corroboration layer. It answers what Tokn directly observed around one stable source and one mutation window; it does not decide whether Astra retained stale context or whether an invalidation action was required.

## Inputs

The reducer consumes three existing read-only history contracts with identical project/workspace filters and run limit:
- ToolActivityHistory V3, reduced through Source Mutation Window V0;
- SourceVersionHistory V1, reduced through run-boundary source comparison;
- WorkspaceGitProvenanceHistory V1.

Mismatched input scopes fail closed.
Unknown future source schemas continue to fail closed through the existing reducers, and unknown Workspace Git provenance schema versions are rejected directly.

No Store V2 migration and no Measurement Contract V1 change are required.

## Identity separation

`ContentFingerprint` and `SourceVersionFingerprint` remain separate project-scoped derivation domains.
Tokn never compares them directly for equality.

The join is made through the common `SourceStableId`, run, project and workspace identities. This preserves the distinction between:
- exact content observed transiently by a file read;
- exact source version observed by a Runner snapshot boundary;
- workspace-level Git HEAD/dirty provenance.

## Evidence observations

For every Source Mutation Window V0 observation, Tokn emits one Source Freshness Evidence record.

`CHANGE_AND_REREAD_WITH_BOUNDARY_CHANGE_OBSERVED` requires:
- a complete unambiguous same-thread mutation window;
- exact read content before and after the window differs;
- the same `SourceStableId` also has exact Runner BEFORE/AFTER source versions that differ.

This is multi-surface corroboration that a source change and a later exact reread were observed in the run. It is not proof that the targeted mutation caused the durable boundary change.

`CHANGE_AND_REREAD_OBSERVED` requires exact read content difference in the accepted mutation window, while run-boundary source evidence is unchanged or unavailable. This can happen when content changes and later returns, or when boundary coverage cannot corroborate the in-run observation.

`SAME_CONTENT_REREAD_OBSERVED` means the accepted read-before/read-after window observed the same exact content fingerprint. It does not prove that no transient change occurred and is not promoted to `FRESH`.

`UNKNOWN` preserves an incomplete or ambiguous Source Mutation Window V0 observation.

## Workspace Git corroboration

Workspace Git evidence is reported independently as:
- `HEAD_CHANGED_OBSERVED`;
- `HEAD_UNCHANGED_OBSERVED`;
- `UNKNOWN`.

The reducer exposes BEFORE/AFTER dirty booleans only when the corresponding Git coverage is `OBSERVED`.

A matching HEAD does not imply matching working-tree content. A changed HEAD does not identify the source responsible for the commit change. Git evidence therefore never upgrades the freshness or invalidation status.

## Explicit non-claims

Every record keeps:
- `freshness_status = NOT_PROVEN`;
- `invalidation_status = NOT_PROVEN`.

This V0 does not prove:
- that an earlier model context item was stale;
- that Astra retained, forgot or used the earlier read;
- that a reread was required, redundant or avoidable;
- that a mutation tool call caused the observed content change;
- that a source change was caused by the observed Git transition;
- that invalidation, cache reuse, context reuse or memory reuse is safe;
- any token, latency or quality savings.

The word `freshness` in this contract names the evidence domain being prepared, not a positive freshness verdict.

## CLI

`tokn-observe source-freshness-evidence`

Optional project/workspace filters and bounded run limit follow the same privacy-safe rules as the underlying history queries.
The command is read-only and persists no derived findings.

## Privacy

The reducer uses only durable pseudonymized identities/fingerprints and coverage metadata already accepted by M4.
It does not add raw paths, raw file bytes, raw tool outputs, raw Git commit hashes, branch names or remotes.

## Next boundary

A real stale/fresh or invalidation verdict still requires evidence that is not currently captured authoritatively, especially the relation between earlier supplied/used context and later source state.

Next M4 work should therefore focus on directly observable rediscovery/compaction events and explicit cross-run comparison primitives with compatible provenance scope. Until those contracts exist, `NOT_PROVEN` is the correct freshness/invalidation state.
