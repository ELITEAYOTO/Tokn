# Source Identity + Observed Content Evolution V0

Status: M4 ACCEPTED FOUNDATION / M4.5 SEED
Date: 2026-10-03

## Purpose

Extend Context/Result Identity V0 with a stable logical identity for directly provable file sources and a conservative historical view of exact observed content evolution.

This slice answers two bounded questions:
1. can two observations be tied to the same logical file source across workspace clones?
2. when content identity is fully observed, did that source return one exact content fingerprint or multiple exact fingerprints?

It does not claim that a source is fresh, stale, still remembered by the model, safe to reuse, redundant or wasteful.

## SourceStableId V0

The Codex adapter currently emits a source locator only for conservative single-file `Get-Content` reads.

Accepted V0 evidence requires:
- activity category `file_read`;
- one simple `Get-Content` target;
- a literal target with no wildcard, variable, pipeline or compound command;
- an explicit tool workdir or directly observed agent cwd;
- a Runner `selected_workspace`;
- a resolved target located inside that selected workspace.

The transient logical locator is workspace-relative, for example `file:src/example.rs`.

The raw locator is not persisted. Tokn derives a project-scoped `src-v1-*` identifier with a source-specific BLAKE3 keyed-derivation domain distinct from the content-fingerprint domain.

Consequences:
- the same logical relative file in two clones of the same project can receive the same SourceStableId;
- the same locator under another project scope receives another SourceStableId;
- absolute workspace paths are not embedded in the durable identifier;
- SourceStableId is not an operation fingerprint and does not represent content equality.

Unsupported or ambiguous command shapes remain `NOT_CAPTURED` rather than being guessed.

## Content evolution report

`tokn-observe source-identity-history` groups observed SourceStableIds from ToolActivityHistory V2 and reports occurrence, run and thread counts plus exact content-identity coverage.

Its content evolution values are:
- `UNCHANGED_OBSERVED`: every captured occurrence has observed content identity and all exact fingerprints are equal;
- `CHANGED_OBSERVED`: every captured occurrence has observed content identity and more than one exact fingerprint exists;
- `UNKNOWN`: complete exact content identity is not available.

`PARTIAL`, `NOT_CAPTURED` and `UNKNOWN` coverage remain explicit. Missing content is never treated as equality.

## Non-claims

`UNCHANGED_OBSERVED` does not prove:
- that the source is currently fresh;
- that no edit happened between observations and was later reverted;
- that Astra still has the evidence in retained context;
- that rereading it was unnecessary;
- that a Project Memory entry would be safe to reuse.

`CHANGED_OBSERVED` proves only that at least two exact observed result fingerprints differ under complete coverage. It does not identify the edit, invalidation time or causal reason.

## Privacy and fail-closed rules

- no raw file path, command output or logical locator is added to the Store by this slice;
- persisted source IDs and content fingerprints use separate project-scoped derivation domains;
- Store validation rejects malformed scoped fingerprints;
- `OBSERVED` identity coverage without the corresponding identity is invalid;
- paths outside the selected workspace, UNC-style roots and ambiguous command forms are not captured.

## Next boundary

Still required before a real freshness/rediscovery finding:
- source version / Git commit or directly observed invalidation events;
- broader runtime-neutral source kinds such as symbol/range identities only when provable;
- compaction/context-management evidence where observable;
- chronological rediscovery events;
- explicit compatible cross-run comparison.

These remain observation-only. No optimizer, memory injection or savings claim follows from this V0 contract.
