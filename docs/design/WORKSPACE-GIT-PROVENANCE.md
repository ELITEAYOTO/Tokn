# Workspace Git Provenance Boundary Evidence

Status: ACCEPTED FOUNDATION / OBSERVATION-ONLY (2026-10-03).

## Purpose

This slice records workspace-level Git provenance at the same Runner BEFORE/AFTER boundaries used by source-version evidence.
It adds directly observed repository state without turning workspace provenance into source freshness, mutation causality or safe-reuse claims.

## Direct evidence source

`ProjectSnapshot` V1 now carries an optional `git` object.
A newly produced snapshot records:
- `status=OBSERVED` only when Git is available, the selected project is inside a Git work tree, `HEAD` resolves and porcelain status succeeds;
- the exact Git `HEAD` commit hash inside the raw Runner snapshot artifact;
- a boolean `dirty` working-tree state;
- `status=UNKNOWN` when Git is unavailable, the path is not a Git work tree, or Git probing cannot be completed reliably.

Legacy ProjectSnapshot V1 artifacts without the optional `git` object remain valid and are projected as `NOT_CAPTURED`.
The additive field does not change the ProjectSnapshot schema version.

## Durable Store projection

`store-evidence` projects each available BEFORE/AFTER snapshot into `workspace_git_provenance_v1`.
For durable storage Tokn keeps only:
- run boundary `BEFORE` or `AFTER`;
- coverage `OBSERVED`, `NOT_CAPTURED` or `UNKNOWN`;
- a project-scoped, domain-separated `git-v1-*` fingerprint derived from the observed HEAD when coverage is `OBSERVED`;
- the observed dirty boolean when coverage is `OBSERVED`;
- optional snapshot observation time.

The raw Git commit hash is not persisted in the SQLite Store.
Branch names, remote URLs, commit messages, authors and per-file `git status` paths are not captured by this contract.

The Store and SQLite both enforce at most one workspace Git provenance row per `(run_id, boundary)`.
Unknown future `workspace_git_provenance_schema_version` values fail closed.

CLI: `tokn-observe workspace-git-provenance-history` with optional project/workspace filters and a bounded run limit.

## Coverage semantics

`OBSERVED` requires both a valid HEAD fingerprint and a dirty state.
`NOT_CAPTURED` is reserved for legacy/current evidence where the snapshot itself did not carry a Git block.
`UNKNOWN` means the producer attempted the Git observation but could not prove repository state.
Non-observed coverage cannot carry a HEAD fingerprint or dirty value.

## Non-claims

This evidence does not prove:
- which file or source caused a dirty working tree;
- that a mutation tool call changed durable bytes;
- that a changed HEAD is the cause of a particular SourceVersion change;
- branch identity, remote identity or commit ancestry semantics beyond the observed HEAD value;
- that an earlier read/context is stale;
- that Astra retained or forgot context;
- invalidation correctness, safe reuse, cacheability or token savings.

A matching HEAD does not imply matching working-tree content because either boundary may be dirty.
A changed HEAD does not by itself identify which source changed.

## Validation

The capture path was exercised against a clean Git repository, a dirty Git repository and a non-Git directory:
- clean Git -> `OBSERVED`, `dirty=false`;
- dirty Git -> `OBSERVED`, `dirty=true`;
- non-Git/unavailable provenance -> `UNKNOWN` without failing the project snapshot.

Rust validation covers idempotent persistence/history, project-scoped `git-v1-*` fingerprints, absence of the raw HEAD in SQLite and duplicate-boundary rejection.

## Next boundary

Source Freshness Evidence V0 now consumes Workspace Git provenance as independent run-boundary corroboration beside Source Mutation Window and exact SourceVersion evidence.
It reports only `HEAD_CHANGED_OBSERVED` / `HEAD_UNCHANGED_OBSERVED` / `UNKNOWN` plus observed dirty booleans; Git state never upgrades `freshness_status` or `invalidation_status` from `NOT_PROVEN`.

Directly observable compaction/rediscovery and explicit cross-run provenance compatibility remain required before any stronger stale/fresh contract.
