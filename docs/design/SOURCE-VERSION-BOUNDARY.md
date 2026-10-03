# Source Version Boundary Evidence

Status: ACCEPTED FOUNDATION / OBSERVATION-ONLY (2026-10-03).

## Purpose

This slice records an exact privacy-safe version observation for a logical file at Runner run boundaries.
It is intentionally narrower than freshness, invalidation or mutation-effect verification. Workspace-level Git provenance is a separate accepted evidence surface.

## Direct evidence source

The Runner already emits `workspace-before-snapshot.json` and `workspace-after-snapshot.json` using ProjectSnapshot V1.
For each captured file the snapshot directly contains a workspace-relative path, SHA-256, byte count and optional snapshot timestamp.
Tokn accepts only the existing snapshot schema and fails closed on unsupported versions.

The relative path is canonicalized with the same workspace-relative locator rules used by SourceStableId.
Therefore the same logical file observed by a tool read and by a Runner snapshot can share one project-scoped source identity.

## Durable identities

For every captured boundary file Tokn stores:
- the project-scoped `SourceStableId` (`src-v1-*`);
- boundary `BEFORE` or `AFTER`;
- a project-scoped, domain-separated source-version fingerprint (`ver-v1-*`);
- optional snapshot observation time;
- byte count.

The version fingerprint is derived from the observed snapshot SHA-256 under a dedicated project-scoped derivation domain.
The raw relative path and raw snapshot SHA-256 are not persisted in `source_versions_v1`.
This keeps source identity, result/content identity and source-version identity separate.

## Store contract

`source_versions_v1` is schema version 1.
There may be at most one record for `(run_id, source_stable_id, boundary)`.
The Rust validator rejects duplicates and SQLite enforces the same invariant with a UNIQUE index.
Unknown future source-version schema versions fail closed.

`tokn-observe source-version-history` exposes the durable observed records with optional project/workspace filters and a bounded run limit.
It is a read-only history surface; it does not rewrite or enrich evidence.

## Boundary comparison

The analysis reducer groups one logical source inside one run and emits:
- `UNCHANGED_OBSERVED` when both BEFORE and AFTER exact version fingerprints exist and match;
- `CHANGED_OBSERVED` when both exist and differ;
- `UNKNOWN` when either boundary is absent.

Duplicate boundary input fails closed even outside the Store.
A source present at only one boundary is not called ADDED or REMOVED because snapshot absence alone is not enough evidence for that claim.

## Non-claims

This slice does not prove:
- which mutation, tool call or process caused a version difference;
- that a completed mutation operation changed durable bytes;
- source-specific Git causality, branch/remote identity or file-level Git attribution;
- that an earlier context/read is stale;
- that Astra retained or forgot any context;
- invalidation correctness, safe reuse, cacheability or token savings.

`CHANGED_OBSERVED` means only that two directly observed run-boundary versions differ.
`UNCHANGED_OBSERVED` means only that the two directly observed boundary versions match.
Neither result is a freshness verdict. Workspace-level HEAD/dirty boundary provenance is captured separately by `WORKSPACE-GIT-PROVENANCE.md`.

## Next evidence needed

Source Freshness Evidence V0 now joins exact source boundary comparison with Source Mutation Window chronology and Workspace Git provenance without comparing `ContentFingerprint` and `SourceVersionFingerprint` directly.
This is corroboration only: even a read-content difference plus `CHANGED_OBSERVED` boundary keeps freshness/invalidation `NOT_PROVEN`.

Before a stronger freshness contract Tokn still needs, where directly observable:
- compaction/rediscovery evidence;
- explicit coverage/comparability/provenance rules across runs;
- evidence tying supplied/used context to later source state;
- broader source kinds only when stable identity is provable.

These additions must preserve `UNKNOWN` / `NOT_CAPTURED` rather than turning absence of evidence into a positive claim.
