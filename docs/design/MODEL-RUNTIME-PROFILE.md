# Tokn ModelRuntimeProfile

Status: FROZEN V1 DOMAIN CONTRACT / PERSISTENCE PENDING
Date: 2026-10-01

## Purpose

Tokn must compare runs without assuming that every model, Codex version
or configuration exposes the same telemetry or behaves identically.

ModelRuntimeProfile records the environment in which evidence was produced
and the capabilities Tokn actually observed.

It is descriptive infrastructure, not a model quality score.

The V1 serializable domain contract is implemented in
`crates/tokn-domain/src/model_runtime.rs` and versioned by
`docs/design/MEASUREMENT-CONTRACT-V0.1.md`.
Store persistence is intentionally the next phase, not part of this freeze.

## Profile identity

Minimum identity fields:

- schema_version;
- observed_at;
- runtime_kind;
- runtime_version;
- app_version when independently observable;
- model;
- model_provider;
- reasoning/effort mode when observable;
- model_context_window when observable;
- multi-agent/runtime protocol version when observable.

## Configuration context

Record only configuration that can materially affect comparison:

- permission profile;
- approval policy;
- sandbox policy;
- collaboration/multi-agent mode;
- context-management mode when observable;
- relevant feature flags with provenance.

Workspace paths and user/account identifiers are not profile identity.

If project identity is needed, Tokn should use its own project/workspace
identity layer rather than persisting personal path/account data here.

## Capability observations

Each capability is evidence-scoped and versioned.

Candidate capabilities:

- token_usage_record;
- total_token_usage;
- last_token_usage;
- cached_input_tokens;
- cache_write_input_tokens;
- per-turn usage;
- per-thread usage;
- parent/fork/subagent links;
- compaction events;
- rate-limit telemetry;
- hook event availability;
- hook field availability;
- local plugin MCP process launch.

Capability state must preserve:
- evidence status;
- source kind;
- source/runtime version;
- first/last observation where useful;
- known gaps.

Missing data never becomes unsupported automatically.

## RateLimitSnapshot

Current Codex evidence supports a separate rate-limit snapshot.

Observed candidate fields:

- limit_id;
- limit_name when available;
- primary.used_percent;
- primary.window_minutes;
- primary.resets_at;
- secondary.used_percent;
- secondary.window_minutes;
- secondary.resets_at;
- credits.has_credits when exposed;
- credits.unlimited;
- credits.balance only if product/privacy requirements justify storing it;
- individual_limit when exposed;
- spend_control_reached when exposed;
- plan_type when exposed;
- rate_limit_reached_type.

Tokn should store only fields required for diagnostics.

The initial analyzer needs percentage/window/reset information;
it does not need monetary credit balances or account identifiers.

A rate-limit snapshot is not the same metric as:
- logical token usage;
- current context occupancy;
- API cost;
- a single universal "Astra budget remaining".

Multiple independent limit windows can coexist.

## Context accounting boundary

Current local session evidence exposes:
- model_context_window;
- total_token_usage;
- last_token_usage.

No field has yet been accepted as direct effective retained-context occupancy.

Therefore:
model_context_window != current occupancy;
total_token_usage != current occupancy;
last_token_usage != current occupancy.

OQ-005 remains open until a stable retained-context signal or a validated
derivation is established.

## Comparison rule

Two runs may be compared descriptively across different profiles,
but differences in model/runtime/configuration must remain visible.

Causal A/B comparability requires the constraints from ADR-004,
including compatible runtime/model/configuration.

Tokn must not label a recurring pattern as a universal model defect.
Report frequency within compatible profiles, for example:
"observed in N of M compatible runs".

## Privacy exclusions

Do not persist in ModelRuntimeProfile by default:
- creator account/user IDs;
- authentication secrets;
- raw prompts;
- raw terminal output;
- full personal filesystem paths;
- credit balances unless explicitly required by a future feature.

## Current evidence

Local runtime observed 2026-09-30:
- codex-cli 0.161.0-alpha.2;
- OpenAI.Codex 26.928.1915.0;
- session JSONL exposes the telemetry listed above.

Official OpenAI/codex source defines rate-limit windows and snapshots,
but capability presence remains checked against the actual target runtime.
