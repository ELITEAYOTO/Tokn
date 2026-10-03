# Source Mutation Observation V0

Status: M4 ACCEPTED FOUNDATION / OBSERVATION-ONLY
Date: 2026-10-03

## Purpose

Record when Tokn directly observes a tool operation that targets a stable file source, without turning command intent or tool completion into a claim that file content actually changed.

This slice answers three bounded questions:
1. can a write-like operation be tied to a directly provable SourceStableId?
2. was an operation timestamp directly present in the runtime evidence?
3. can that evidence be queried historically without persisting raw paths or outputs?

It does not answer whether the mutation took effect, whether the source is stale, whether Astra still retains prior evidence, or whether any reread can be skipped.

## ToolActivityHistory V3

V3 is an additive evolution of the accepted ToolActivityHistory identity surface.

It keeps V2 operation/source/content identity semantics and adds optional `observed_at` on each historical tool activity.

For the Codex rollout adapter, `observed_at` is copied only from the response-item `timestamp` when that field is directly present and non-empty.

Missing timing is not reconstructed from run creation time, sequence numbers or Store insertion time.
## Stable mutation target V0

The command may be classified broadly as `write_mutation`, but stable source identity is intentionally narrower.

V0 source identity is emitted only for simple single-target `Set-Content` or `Add-Content` operations when:
- the command target is literal and unambiguous;
- no wildcard, variable or unsupported compound shape is required to resolve the target;
- an explicit tool workdir or directly observed agent cwd is available;
- a Runner `selected_workspace` is available;
- the resolved target is inside that selected workspace.

The transient locator uses the same workspace-relative `file:*` form as file-read identity and is converted into the same project-scoped `src-v1-*` domain.

A generic `write_mutation` classification, a `Copy-Item`, an ambiguous target, or an outside-workspace target is not enough. Those cases remain `NOT_CAPTURED`.

## SourceMutationHistory V1

`tokn-observe source-mutation-history` filters ToolActivityHistory V3 to mutation operations with observed stable source identity.

Each event preserves:
- activity, run and thread identity;
- SourceStableId;
- optional `observed_at`;
- timing coverage `OBSERVED` or `NOT_CAPTURED`;
- the original tool status;
- `effect_status = NOT_VERIFIED`.
`tool_status = completed` means the runtime reported the tool call completed. It is not promoted to a verified mutation effect.

Events are ordered deterministically by run creation time, then observed timing when available, then activity identity.

## Content identity boundary

SourceIdentityHistory remains restricted to `file_read` observations.

A mutation command's tool output is not the new file content and must never be interpreted as a ContentFingerprint for source evolution. A later exact file read may separately establish new observed content identity.

This separation prevents command success text, shell output or other mutation-result payloads from masquerading as source state.

## Privacy and fail-closed rules

- no raw source locator is stored;
- no raw command, workdir or tool output is added to durable mutation history;
- SourceStableId stays project-scoped and source-domain separated from ContentFingerprint;
- missing rollout timing stays `NOT_CAPTURED`;
- unsupported ToolActivityHistory schema versions fail closed;
- `OBSERVED` source identity still requires a valid scoped SourceStableId.

## Non-claims

An observed mutation operation does not prove:
- that bytes on disk changed;
- what the resulting file content is;
- that a prior read became stale;
- that the edit survived a later revert or overwrite;
- that Astra retained or forgot any previous context;
- that a reread is redundant or avoidable.
## Next boundary

Before Tokn can emit a real freshness or invalidation finding, it still needs evidence such as:
- directly observable source version/hash/Git commit when available;
- verified mutation effect rather than operation intent alone;
- a chronological relation between reads, verified changes and later rediscovery;
- broader runtime-neutral source kinds only when stable identity is provable;
- explicit cross-run compatibility and provenance rules.

Until then, mutation timing is an observation surface only. No optimizer, memory injection, cache/reuse decision or savings claim is authorized by this V0 contract.
