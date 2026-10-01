# Tokn Measurement Contract V0.1

Status: FROZEN V1 CONTRACT
Date: 2026-10-01
Contract ID: `tokn.measurement.v0.1`

## Purpose

This contract freezes the measurement boundary consumed by the next Tokn layers:
Store, ModelRuntimeProfile persistence, Local MCP integration, Historical Analyzer,
Context Ledger and Findings Engine.

It freezes measurement semantics, not product conclusions.
A frozen measurement contract does not make an experiment causal.

The machine-readable source of truth is:
- `crates/tokn-domain/src/contract.rs`;
- `measurement-contract.json` emitted by every Runner evidence folder.

## Compatibility rule

V1 is the first post-Experiment-002 frozen contract.

From this freeze onward:
- changing the meaning of a metric requires a semantics-version bump;
- adding, removing or renaming a serialized field outside an explicitly extensible map
  requires the corresponding schema-version bump;
- changing required/optional status requires a schema-version bump;
- changing evidence-folder filenames or required/conditional artifact rules requires
  an evidence-layout-version bump;
- unknown future versions must be rejected or routed through an explicit adapter;
- missing evidence never becomes zero, PASS or unsupported by default.

Additive values inside explicitly extensible maps such as
`ModelRuntimeProfile.capabilities` or `feature_flags` do not by themselves change
the enclosing schema when their existing semantics remain unchanged.

## Frozen versions

All current V0.1 versions are `1`:
- measurement contract manifest;
- evidence layout;
- Runner request;
- Runner result;
- normalized session evidence;
- RunGroup;
- token-accounting semantics;
- source-health report;
- terminal-status semantics;
- workspace inventory;
- workspace resolution;
- project snapshot;
- workspace diff;
- policy evidence;
- quality gate;
- recovery report;
- experiment validity;
- ModelRuntimeProfile;
- analyzer semantics.

The exact numeric list is serialized in `measurement-contract.json`.

## Evidence folder layout V1

Always emitted by a successful Runner invocation far enough to initialize evidence:
- `runner-request.json`;
- `measurement-contract.json`;
- `recovery-report.json`;
- `source-health.json`;
- `session-evidence.json`;
- `run-group.json`;
- `workspace-resolution.json`;
- `runner-result.json`.

Conditionally emitted:
- `workspace-before-snapshot.json`;
- `workspace-after-snapshot.json`;
- `workspace-diff.json`;
- `policy-evidence.json`;
- `quality-gate.json`;
- `validity-input.json`;
- `validity-report.json`.

Snapshot artifacts are a pair: before and after must either both be supplied or both
be absent.

Runner-owned evidence is immutable after `runner-result.json` exists.
A partial run may remove only known Runner-owned partial artifacts and records that
action in `recovery-report.json`.

Raw private runtime evidence is not copied into this evidence folder by default.

## Runner request/result V1

`RunnerRequest.schema_version=1` is validated fail-closed.

The request identifies:
- logical run ID;
- analysis source;
- optional explicit session candidates;
- source workspace;
- before/after workspace inventory;
- optional before/after project snapshots;
- expected output workspaces;
- evidence destination;
- optional experiment validity hints;
- optional policy observation configuration;
- optional quality gate.

`RunnerResult.schema_version=1` summarizes measured state and references generated
artifacts. It is not a replacement for the detailed artifacts.

A future field/semantic change after this freeze follows the compatibility rule above.

## RunGroup identity V1

A RunGroup represents one root agent plus all discovered descendants that belong to
the same logical run.

Frozen rules:
- `root_thread_id` identifies the root thread;
- `session_id` is retained when the runtime exposes it;
- `parent_thread_id` represents ancestry;
- agent depth is derived from the parent graph;
- each unique member is counted exactly once;
- the parent is never used as a proxy for the complete run when descendants exist;
- `source_path` is provenance, not project identity;
- workspace/project identity is handled by the workspace lineage layer.

If ancestry cannot be established safely, Tokn must keep the state incomplete or
unknown rather than merge unrelated sessions.

## Token accounting semantics V1

Frozen invariants:
- cached input is included in input;
- reasoning output is included in output;
- `logical_total = input + output`;
- logical total is authoritative only when both input and output are known for every
  aggregated usage record;
- `ordinary_uncached_input = input - cached_input`;
- cache-write input is preserved separately and is not subtracted from ordinary
  uncached input;
- cache-write input is not added again to logical total;
- known counters preserve coverage: a numeric aggregate is not proof that every
  record exposed that metric;
- invalid relationships are reported, never silently repaired.

These metrics are not equivalent to:
- current retained-context occupancy;
- API price/cost;
- Codex/ChatGPT product quota;
- model quality.

In particular:
`model_context_window != current occupancy`;
`total token usage != current occupancy`.

## Source health and provenance V1

Source kinds currently include Codex session rollout and Codex diagnostic trace.

Source-health statuses preserve:
`HEALTHY`, `PARTIAL`, `EMPTY`, `MALFORMED`, `UNSUPPORTED`, `UNKNOWN`.

A source with zero observed usage but incomplete coverage does not prove zero usage.

Fallback rules:
- fallback must be explicit;
- the selected source and source kind remain visible;
- a diagnostic/session fallback never permits blind addition of overlapping evidence;
- no fallback may invent or semantically reconstruct missing events;
- persisted immutable session evidence may be used deterministically when the original
  diagnostic representation is unsupported, as demonstrated by Experiment 002.

## Terminal status semantics V1

Frozen values:
- `COMPLETED`;
- `INCOMPLETE_USAGE_LIMIT`;
- `INCOMPLETE_CANCELLED`;
- `INCOMPLETE_ERROR`;
- `INCOMPLETE_USER_ABORT`;
- `UNKNOWN_TERMINAL_STATE`.

Token usage may still be described for an incomplete run.
Completion-sensitive quality/causal comparisons must respect terminal state.

## Workspace lineage V1

Workspace inventory and project snapshots are schema-versioned inputs.

The Runner:
- resolves output workspace from declared expectations plus observed activity;
- never assumes input workspace equals output workspace;
- fails conservatively on ambiguous/no candidate;
- requires after-snapshot project root to target the selected workspace before a
  workspace diff can support downstream claims.

Workspace diff preserves:
- added files;
- modified files with before/after SHA-256 and byte sizes;
- removed files.

Full personal filesystem paths are runtime evidence and must not become
ModelRuntimeProfile identity.

## Quality gate V1

A quality gate records:
- required/not required;
- selected workspace;
- executable and arguments;
- exit code;
- stdout/stderr;
- status `PASS`, `FAIL`, `UNAVAILABLE`, `NOT_REQUIRED` or `UNKNOWN`.

A required unavailable quality gate blocks the pipeline.
A failed gate may still be evidence, but it is not successful task acceptance.

## Experiment validity V1

ADR-004 remains authoritative.

Verdicts:
- `VALID_FOR_CAUSAL_AB`;
- `VALID_FOR_DESCRIPTIVE_COMPARISON`;
- `INSTRUMENTATION_ONLY`;
- `INVALID_CAPTURE`;
- `INCOMPLETE_TASK`.

Only `VALID_FOR_CAUSAL_AB` permits causal optimization claims.

Experiment 002 is intentionally `INSTRUMENTATION_ONLY`; its acceptance validates
the measurement pipeline, not token savings.

## ModelRuntimeProfile V1

The domain type is frozen in:
`crates/tokn-domain/src/model_runtime.rs`.

Minimum identity/context:
- schema version;
- observed timestamp;
- runtime kind;
- runtime version when observed;
- app version when independently observed;
- model/provider when observed;
- reasoning/effort mode when observed;
- model context window when observed;
- multi-agent protocol version when observed.

Configuration fields are optional and evidence-scoped:
- permission profile;
- approval policy;
- sandbox policy;
- collaboration mode;
- context-management mode;
- relevant feature flags.

`configuration_complete` uses the existing validity status contract.
Experiment 002 correctly leaves it `UNKNOWN`.

Capability observations are extensible by key and preserve:
- evidence status;
- source kind;
- runtime version;
- optional first/last observation;
- known gap/note.

`UNKNOWN` does not mean unsupported.

Do not persist in ModelRuntimeProfile by default:
- user/account IDs;
- auth secrets;
- raw prompts;
- raw terminal output;
- full personal paths;
- monetary credit balances.

Store persistence is intentionally outside this freeze and comes next.

## Analyzer semantics V1

Current analyzers must preserve the frozen source/accounting/identity semantics above.

Future findings are higher-level derived evidence.
They must not mutate the underlying measured facts.

A finding schema may evolve separately, but any analyzer change that changes the
meaning of an existing V1 derived metric requires an analyzer-semantics-version bump.

## Evidence from Experiment 002

The first real Experiment 002 run validated the boundary with:
- 1 parent + 1 subagent;
- 49 usage records;
- terminal `COMPLETED`;
- Runner `COMPLETE`;
- quality `PASS`, 863/863 tests;
- workspace diff 1 added / 2 modified / 0 removed;
- model `gpt-6.1-sol` observed;
- configuration completeness `UNKNOWN`;
- verdict `INSTRUMENTATION_ONLY`.

Its FINISH harness recovery reused immutable persisted rollouts without a model rerun.
That event motivated explicit source/layout versioning in this contract.

## Exit condition

Measurement Contract Freeze is complete when:
- the machine-readable manifest is emitted by Runner;
- the golden replay asserts every frozen version;
- token accounting semantics have regression tests;
- ModelRuntimeProfile V1 has a serializable tested domain contract;
- workspace input versions are rejected when unsupported;
- full workspace tests, Clippy, release replay and documentation consistency pass.

Only then may Tokn start Store + ModelRuntimeProfile persistence.
