# Context / Result Identity Foundation V0

Status: M4 ACCEPTED FOUNDATION / M4.5 SEED
Date: 2026-10-03

## Purpose

Provide runtime-neutral identity primitives for evidence without persisting raw tool outputs by default. This foundation lets later analyzers distinguish exact same-result evidence from merely repeated operations while preserving uncertainty explicitly.

## Identity model

`SourceStableId` and `ContentFingerprint` are intentionally different concepts.

- `SourceStableId`: stable logical source identity across content versions when the runtime exposes enough evidence.
- `ContentFingerprint`: project-scoped privacy-safe fingerprint of exact observed content/result bytes.

V0 does not derive SourceStableId from an operation fingerprint, path size or other proxy. If a stable logical source cannot be proved, coverage stays `NOT_CAPTURED` or `UNKNOWN`.

## Coverage

Identity coverage is one of:
- `OBSERVED`: directly captured and usable;
- `PARTIAL`: only part of the relevant occurrences is observed;
- `NOT_CAPTURED`: the current adapter/contract did not capture it;
- `UNKNOWN`: evidence should have been inspectable but was unavailable/unreadable.

Absence of a fingerprint is never silently interpreted as equality or difference.

## Codex V0 capture

For persisted session evidence, the Codex adapter may transiently read `custom_tool_call_output`. Raw result content is not added to `AgentEvidence` or the Store contract.

A `content_fingerprint` is emitted only when:
1. the output is directly observed;
2. exactly one normalized operation belongs to the base call;
3. exactly one result output belongs to that call.

If one Code Mode call expands to multiple operations, the common result is not copied onto every operation. Coverage remains `NOT_CAPTURED` for those activities. If the source file cannot be read, coverage is `UNKNOWN`.

## Persistence

Tool Activity Store schema V2 adds:
- `source_stable_id`;
- `source_identity_coverage`;
- `content_fingerprint`;
- `content_identity_coverage`.

Migration from Tool Activity V1 defaults old rows to `NOT_CAPTURED` and remains fail-closed for unknown future schema versions. No raw command, workdir or tool-result content is persisted by this slice.

## Cross-agent semantics

Cross-Agent Evidence schema V2 groups exact operation overlap as before, then summarizes result identity:
- complete observed identity + one distinct fingerprint -> `SAME`;
- complete observed identity + multiple fingerprints -> `DIFFERENT`;
- incomplete/unavailable identity -> `UNKNOWN`.

`SAME` does not by itself mean redundant, unnecessary, wasteful or avoidable. Those are later findings requiring contextual and eventually experimental evidence.

## Transient result identity capture hardening

The current Codex path still materializes raw `custom_tool_call_output` values in a `BTreeMap<String, Vec<String>>` before `store-evidence` reduces them to exact project-scoped fingerprints. This is acceptable today, but it is an implementation hardening opportunity rather than a new product subsystem.

Accepted direction: first benchmark representative large and many-small outputs. If peak RSS, allocation pressure or repeated rollout IO is non-trivial, prefer a private adapter-local visitor or bounded fingerprint+count accumulator that hashes observed results promptly and releases raw content as early as practical.

Any such refactor must preserve identical exact fingerprints, one-operation/one-result ambiguity rules, `OBSERVED` / `NOT_CAPTURED` / `UNKNOWN` semantics, and durable Store output. It must not require a Store migration, Measurement Contract change, generic buffer crate, daemon, fuzzy similarity or raw spill-to-disk. Dropping a Rust `String` is not claimed to provide secure zeroization.

## Remaining boundary

SourceStableId file V0 is now implemented for conservative simple `Get-Content` reads that resolve inside the Runner selected workspace. The durable ID is project-scoped and the raw workspace-relative locator is not persisted. See `SOURCE-IDENTITY-CONTENT-EVOLUTION.md`.

Still not solved by V0:
- broader SourceStableId extraction beyond the conservative file-read subset;
- range/symbol identity;
- broader source-version/Git provenance semantics beyond the accepted file SourceVersion V0 + workspace Git provenance V0, especially across runtimes;
- freshness/invalidation;
- compaction/rediscovery;
- semantic/fuzzy equivalence;
- cross-run duplicate interpretation.

These remain observation-only M4/M4.5 work and must preserve `UNKNOWN` / `NOT_CAPTURED`.
