# Cross-Run Comparability V0

Status: M4 ACCEPTED OBSERVATION FOUNDATION / OBSERVATION-ONLY
Date: 2026-10-03

## Purpose

Compare two explicitly selected historical runs on captured starting-state and runtime evidence without ranking their outcomes and without authorizing causal claims.

This primitive answers a narrow question: **are the conditions Tokn actually captured compatible across the two runs?**
With the Task Input Identity V0 extension it can also compare exact frozen task-artifact identity when both sides directly captured it. It still does not prove that those bytes were delivered unchanged to the runtime/model, whether only one intervention changed, or which run was better.

## Inputs

`tokn-observe cross-run-comparison` reads existing Store V2 history only:
- HistoricalSnapshot V1 for run identity, contract versions and runtime profiles;
- SourceVersionHistory V1 for exact `BEFORE` source state;
- WorkspaceGitProvenanceHistory V1 for `BEFORE` Git HEAD/dirty evidence;
- TaskInputHistory V1 for exact privacy-safe frozen task-artifact identity when captured.

The accepted Cross-Run base required no Store migration. The Task Input Identity V0 extension uses additive Store migration `0008_task_input_identity.sql`; Measurement Contract V1 and Runner V1 remain unchanged.

The baseline and candidate run ids must be distinct and present inside the selected bounded history window.
Optional project filtering uses the existing private `prj-*` identity boundary.

## Required observed-scope checks

The top-level `observed_scope_status` is reduced fail-closed from these axes:

### Project scope

`PROJECT_SCOPE` passes only when both runs belong to the same project id.
SourceStableId, SourceVersionFingerprint and Git HEAD fingerprints are project-scoped derivations, so Tokn does **not** compare those fingerprints across different projects.

### Frozen contract versions

`MEASUREMENT_CONTRACT` and `EVIDENCE_LAYOUT` pass only when both runs use the same currently supported frozen V1 versions.
Different versions fail; matching but unsupported future versions remain `UNKNOWN`.

### Runtime profile

`RUNTIME_PROFILE` delegates to the already accepted RuntimeProfileCompatibility V1 reducer.
Runtime kind/version, model/provider, reasoning, configuration and feature evidence keep their existing PASS/FAIL/UNKNOWN semantics.

### Source BEFORE state

Tokn compares only exact `BEFORE` SourceVersion records within the same project scope.

Per source it reports:
- `SAME_VERSION_OBSERVED`;
- `DIFFERENT_VERSION_OBSERVED`;
- `BASELINE_ONLY_OBSERVED`;
- `CANDIDATE_ONLY_OBSERVED`.

`SOURCE_BEFORE_STATE`:
- `PASS` only when both observed source sets are non-empty, identical, and every exact version matches;
- `FAIL` when the same observed SourceStableId has different exact versions;
- `UNKNOWN` when either side has no observed BEFORE versions or the observed source sets are asymmetric.

An asymmetric snapshot is not relabeled as an added/removed source because absence from captured evidence is not proof of filesystem absence.
Duplicate BEFORE source records fail closed.

### Task input identity

`TASK_INPUT_IDENTITY` compares only project-scoped `tsk-v1-*` identities within one project scope:
- `PASS` when both runs directly captured exact task-artifact identity and the fingerprints match;
- `FAIL` when both runs directly captured exact task-artifact identity and the fingerprints differ;
- `UNKNOWN` for missing, partial, not-captured or otherwise incomplete identity.

Across different project ids Tokn does not compare task fingerprints. `SAME_IDENTITY_OBSERVED` proves equality of the supplied task artifact bytes only; it does not prove runtime/model delivery. TaskInputHistory therefore keeps `delivery_status=NOT_PROVEN`.

### Workspace Git BEFORE state

`WORKSPACE_GIT_BEFORE`:
- `PASS` only when both BEFORE records are `OBSERVED`, HEAD fingerprints match, and both working trees are explicitly clean;
- `FAIL` when observed HEAD fingerprints differ, or matching HEAD has an observed dirty-state disagreement;
- `UNKNOWN` for missing/non-observed evidence, missing HEAD/dirty values, or when both worktrees are dirty.

Same HEAD with dirty worktrees is insufficient because uncommitted content may differ.
Duplicate BEFORE Git records fail closed.

## Result semantics

`observed_scope_compatible=true` means only that **all captured comparability axes above pass**.
It is not equivalent to causal experiment validity.

Every report keeps:

`causal_claims_status = NOT_ESTABLISHED`

because this historical surface does not by itself prove:
- that the captured task artifact was delivered unchanged to the runtime/model;
- same external environment/dependencies;
- same host-side hidden state;
- a single primary intervention difference;
- equivalent quality requirements or completion semantics.

Experiment Validity remains the authority for causal A/B claims.

## Non-claims

Cross-Run Comparability V0 does not:
- rank baseline vs candidate;
- call either run better/worse;
- estimate token savings;
- prove policy effect;
- turn task-artifact identity into proof of runtime delivery;
- infer source or task equality across project scopes;
- turn missing evidence into compatibility.

## Compaction boundary

Current Codex Diagnostic parsing can count payload kinds containing `compaction`, but the retained real local traces and sanitized fixtures inspected on 2026-10-03 contain no detailed compaction event example.
Tokn therefore does not invent a detailed compaction-event schema or join diagnostic `seq` values to session-rollout `seq` values.
Detailed compaction chronology remains `NOT_CAPTURED` until a real event can be observed, sanitized and covered by adapter fixtures.

## Next boundary

After this comparability primitive, remaining M4 evidence work includes:
- real compaction/rediscovery evidence only when directly observable;
- runtime task/context-delivery evidence needed beyond the now-captured task-artifact identity for stronger causal controls;
- broader source kinds only when stable identity is provable.
