# Experiment 002 - Preflight

Status: PREPARED / NO MODEL RUN YET
Generated: __DATE__

## Tokn baseline

- Tokn V0.1 Measurement Hardening: DONE.
- Required commit: `__TOKN_COMMIT__` (`__TOKN_COMMIT_SHORT__`).
- Workspace version: `0.1.0`.
- P9 release gates: PASS.
- Harness definition is versioned under `scripts/experiment/002/`.
- Release binary must have a provenance record for the same clean commit.

## Target workspace

`__PROJECT_ROOT__`

Workspace watch root:
`__WORKSPACE_WATCH_ROOT__`

## Known starting failure

`npm run verify:local` is expected to have exactly one frozen failure:
- 833 tests total;
- 832 PASS;
- 1 FAIL;
- missing `src/plugin/quick-fix-workflow.ts`.

START re-runs the complete `verify:local` baseline and the targeted
B07-C workflow test. Any count/reason drift refuses the experiment before
Codex is launched.

## Runtime / model evidence

START records:
- Codex Desktop version;
- Codex CLI version;
- Tokn release binary SHA-256;
- source Git commit;
- harness manifest SHA-256.

FINISH writes `runtime-observation.json` from the run-scoped diagnostic trace.
Observed model/config values are preserved with provenance.

`model_recorded` may PASS when model evidence is present.
`configuration_recorded` intentionally remains UNKNOWN until the
post-Experiment-002 ModelRuntimeProfile contract is frozen.

## Experiment rule

This experiment validates instrumentation only.
No token-savings claim, winner claim, output cap, context reduction or
model restriction is part of the task.
