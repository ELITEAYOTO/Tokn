# Task Input Identity V0

Status: M4 ACCEPTED OBSERVATION FOUNDATION / ARTIFACT-IDENTITY ONLY
Date: 2026-10-03

## Purpose

Give Tokn a privacy-safe, directly evidenced identity for the frozen task artifact associated with a run, so historical run comparison no longer has to treat task identity as wholly absent when an exact task file is available.

This contract deliberately separates **task artifact identity** from **runtime delivery**.

An observed Task Input fingerprint means Tokn read the exact bytes of the task artifact explicitly supplied to Store ingestion. It does **not** prove that Codex/Astra, another runtime, or a model actually received those bytes.

## Existing evidence basis

The Experiment 001 and Experiment 002 harnesses already follow the same evidence pattern:
- freeze/copy `TASK.md` into the run folder;
- compute and record a SHA-256 for the frozen task artifact in harness `run.json`;
- place the extracted task text in the clipboard for manual runtime submission.

That harness evidence is sufficient to justify an exact artifact-identity primitive. It is not sufficient to claim runtime delivery, because clipboard placement and later user/runtime interaction are separate events.

## Store ingestion

`tokn-observe store-evidence` accepts the optional argument:

`--task-input <task-file>`

When supplied:
- Tokn reads the exact file bytes transiently;
- derives a project-scoped, domain-separated `tsk-v1-*` BLAKE3 fingerprint;
- records byte length;
- persists `coverage=OBSERVED`;
- does not persist the task text or task path.

When omitted, ingestion writes an explicit `NOT_CAPTURED` task-input observation for the run.

An empty or missing explicitly requested task file is rejected instead of being collapsed into `NOT_CAPTURED`.

## Durable schema

Migration:
`crates/tokn-storage/migrations/0008_task_input_identity.sql`

Table:
`task_input_identity_v1`

Persisted fields are limited to:
- private observation id;
- run id;
- coverage (`OBSERVED`, `PARTIAL`, `NOT_CAPTURED`, `UNKNOWN`);
- optional project-scoped `tsk-v1-*` fingerprint;
- optional byte length;
- Store creation time.

Exactly one task-input observation is owned by each newly ingested run. Re-ingestion replaces that run-owned observation deterministically, preserving Store idempotence.

The migration is additive to Store V2. It does **not** change Measurement Contract V1, RunnerRequest V1, RunnerResult V1 or Evidence Layout V1.

Unknown future task-input schema versions fail closed when the Store opens.

## Query contract

CLI:
`tokn-observe task-input-history`

Optional filters follow the other historical surfaces:
- `--project-id prj-*`;
- `--workspace-id wsp-*`;
- `--limit <1..1000>`;
- `--db <path>`;
- `--output-json <path>`.

`TaskInputHistory V1` returns only privacy-safe Store identities and task fingerprints.

Every historical task record also exposes:

`delivery_status = NOT_PROVEN`

This value is derived at query time. It is a semantic guardrail, not a SQLite column: Task Input Identity V0 does not observe the final prompt payload delivered into the runtime/model context.

## Cross-run semantics

Cross-Run Comparability V0 consumes TaskInputHistory as an additional captured-scope axis named `TASK_INPUT_IDENTITY`.

Within one project scope:
- `PASS` only when both runs have `OBSERVED` exact task fingerprints and they match;
- `FAIL` when both runs have `OBSERVED` exact task fingerprints and they differ;
- `UNKNOWN` when either side is missing, not captured, partial, unknown, or lacks an observed fingerprint.

Across different project ids Tokn does not compare `tsk-v1-*` values because they are intentionally project-scoped. The task axis stays `UNKNOWN` there.

The structured comparison reports:
- `SAME_IDENTITY_OBSERVED`;
- `DIFFERENT_IDENTITY_OBSERVED`;
- `BASELINE_ONLY_OBSERVED`;
- `CANDIDATE_ONLY_OBSERVED`;
- `UNKNOWN`.

Even when `TASK_INPUT_IDENTITY=PASS`, Cross-Run keeps:

`causal_claims_status = NOT_ESTABLISHED`

because artifact equality does not prove runtime delivery, external-environment equality, single-primary-variable control, or causal attribution.

## Privacy properties

The raw task file is never written into SQLite by this feature.
The task path is never persisted.
The exact fingerprint is keyed/project-scoped and domain-separated, so the same task bytes in a different project scope produce a different durable identity.

The Experiment 001 golden replay exercises observed task-input ingestion twice, queries TaskInputHistory, and checks that a unique raw task marker is absent from the SQLite bytes.

## Non-claims

Task Input Identity V0 does not prove:
- that the runtime/model received the frozen task;
- that the runtime received it unchanged;
- that only one experimental variable changed;
- that two runs had identical hidden context or external dependencies;
- causal validity;
- quality equivalence;
- freshness/staleness or invalidation behavior.

## Next boundary

A stronger causal/task-delivery gate requires directly observable runtime delivery evidence with a versioned privacy-safe adapter contract. Until such evidence exists, delivery remains `NOT_PROVEN` and Experiment Validity remains the authority for causal A/B claims.
