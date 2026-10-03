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

## Source Mutation Window V0

The next observation-only reducer joins already-persisted ToolActivityHistory V3 evidence without changing Store V2 or Measurement Contract V1.

`tokn-observe source-mutation-window-history` evaluates a mutation only when Tokn can conservatively anchor one logical source inside one run/thread chronology:
- the mutation has an observed `SourceStableId`;
- the mutation runtime status is `completed`;
- mutation `started_seq` / `ended_seq` are directly captured;
- an exact same-source `file_read` exists before the mutation and another after it in the same thread;
- both selected reads have runtime status `completed` and complete observed `ContentFingerprint` identity;
- no other same-thread mutation of that source can intervene between the selected reads;
- no same-run mutation of that source exists on another thread, because cross-agent ordering is not currently provable.

The reducer emits only:
- `EXACT_CONTENT_EQUALITY_OBSERVED` when the exact read fingerprints before and after match;
- `EXACT_CONTENT_DIFFERENCE_OBSERVED` when they differ;
- `UNKNOWN` when any required chronology/identity evidence is incomplete or ambiguous.

Every window keeps `causality_status = NOT_PROVEN`.
A content difference around one completed mutation is stronger evidence than mutation intent alone, but it still does not prove that this tool call caused the bytes to change: an unobserved external writer, process, hook, filesystem side effect or other source may exist.
Likewise, equality before/after does not prove the mutation had no transient effect because content could have changed and later returned to the same bytes.

This window is therefore a chronology primitive for the later freshness/invalidation reducer, not a mutation-causality verdict.

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

Source Freshness Evidence V0 now joins this chronology with exact SourceVersion BEFORE/AFTER and workspace Git HEAD/dirty provenance as an observation-only corroboration layer.
It can report observed change+reread sequences and boundary corroboration, but keeps both `freshness_status` and `invalidation_status` at `NOT_PROVEN`.

Before Tokn can emit a real stale/fresh or invalidation finding it still needs:
- directly observable compaction/rediscovery evidence when available;
- explicit cross-run compatibility and provenance rules;
- broader runtime-neutral source kinds only when stable identity is provable;
- evidence tying supplied/used context to a later source state, rather than assuming model retention from prior delivery.

No optimizer, memory injection, cache/reuse decision or savings claim is authorized by this V0 contract.
