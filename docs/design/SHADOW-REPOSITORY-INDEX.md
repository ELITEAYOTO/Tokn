# Shadow Repository Index V0

Status: M4.5 ACCEPTED DESIGN / PREREQUISITES IMPLEMENTED / DIRECT_SCAN ACCEPTED REFERENCE FOUNDATION
Date: 2026-10-04

## Purpose

Define a local, rebuildable repository-orientation index that can later help Tokn find likely relevant files without changing Astra/runtime behavior.

V0 is a **shadow** subsystem. It does not inject context, replace runtime search, suppress reads, claim token savings or emit Findings.

The first implementation decision must be earned by measurement. This document therefore freezes boundaries and correctness rules, not a backend winner.

## Why this exists

M4 established privacy-safe source identity, exact observed content identity, source-version history, Git provenance and strict cross-run reread chronology. Those observations show where repository orientation can be measured, but they do not prove that any reread was wasteful.

M4.5 may now build deterministic mechanics beside the runtime:
- enumerate a repository safely;
- maintain a rebuildable lexical index;
- retrieve file-level candidates;
- measure recall, latency, memory, disk and refresh cost;
- preserve provenance and freshness of the **index itself**.

Only later evidence may justify a Context Package or active intervention.

## Architectural boundary

The Shadow Repository Index is **DERIVED / rebuildable local state**.

It is not:
- Measurement Store evidence;
- a new Measurement Contract V1 field;
- a historical truth source;
- an Event Store;
- a runtime adapter;
- a memory bank;
- a public MCP algorithm surface.

The normalized Store remains Tokn's historical analytical record. The shadow index may be deleted and rebuilt without losing measurement history.

Recommended data flow:

```text
Authorized repository root
  -> safe corpus enumeration
  -> file identity + exact index-content hash
  -> backend candidate (direct scan or lexical index)
  -> shadow query result
  -> current-file hash verification
  -> file-level candidate set
  -> measurement only
```

No V0 arrow continues into Astra/runtime context injection.

## V0 document granularity

V0 indexes **files**, not arbitrary chunks or inferred symbols.

Reasons:
- file identity already has an accepted SourceStableId foundation;
- arbitrary line windows would introduce an unmeasured chunking policy;
- symbol/range identity remains evidence-gated;
- file-level retrieval is enough to measure repository orientation value first.

Chunk, symbol, AST/LSP and dependency-graph retrieval are future candidates only after file-level measurements identify a real limitation.

## Corpus enumeration

V0 is Git-backed first.

For a Git workspace, the intended corpus is:
- tracked files;
- untracked but non-ignored files;
- regular files under the authorized repository root only.

Ignored/generated material should stay excluded by Git semantics by default.

The implementation must fail closed on:
- paths escaping the authorized root;
- symlink/reparse traversal that resolves outside the root;
- unreadable files;
- unsupported encodings/binary files;
- file-size policy violations.

Non-Git repositories are `UNSUPPORTED_V0`, not silently full-recursive-scanned. A future provider-neutral enumerator may extend this deliberately.

File-size and encoding policies are operational safety controls, not context-optimization caps. Their values must be declared in the benchmark/corpus configuration before measurement.

## Identity model

Do not collapse retrieval identity into existing evidence fingerprints.

### SourceStableId

When the same project scope key and workspace-relative file locator are available, the index should reuse the exact accepted SourceStableId derivation so retrieval results can join historical evidence without exposing an absolute path.

Implementation prerequisite accepted: `scoped_source_id_bytes()` now lives canonically in `tokn-domain::identity`; `tokn-storage` only re-exports it for compatibility and `store-evidence` consumes the domain primitive directly. The accepted golden vector for `project-a` + `file:src/lib.rs` is locked byte-for-byte, so existing `src-v1-*` identities cannot drift silently.

Do not duplicate the derivation algorithm in a second crate.

### IndexContentHash

The index needs an exact whole-file version identity for rebuild/refresh correctness. This is **not** `ContentFingerprint`:
- existing ContentFingerprint may represent observed tool-result bytes;
- the index hashes whole repository-file bytes;
- cross-domain equality must never be inferred merely because both are hashes.

Implementation prerequisite accepted: `scoped_index_content_hash_bytes()` now derives project-scoped whole-file `ixc-v1-*` identities in `tokn-domain::identity` using a dedicated `tokn.project-scoped-index-content-hash.v1` derivation domain plus `repository-file-bytes-v1`. Cross-domain equality with `src-v1-*`, `cnt-v1-*` or `ver-v1-*` remains forbidden.

### ShadowDocumentId

If a backend needs a row/document key, derive a dedicated deterministic ID from the project scope + SourceStableId + index content version. It is a backend identity, not evidence that the runtime saw the document.

## Project scope input

Existing `src-v1-*` derivation uses the project scope key, not only the already-hashed `project_id`.

Therefore V0 build/refresh operations must receive the same project scope key explicitly or through a future local project configuration. The raw project scope key must not be written into the shadow database or Measurement Store.

## Sensitive derived-cache policy

A repository index is sensitive even when it does not store raw source text.

An FTS term index can reveal vocabulary/identifiers. Workspace-relative locators can reveal repository structure. Therefore:
- keep shadow index files local and outside the tracked repository;
- never include them in release/package artifacts;
- never upload them through Store publication flows;
- never expose backend tables directly through MCP/UI;
- never store absolute paths or the raw project scope key in the index;
- make the entire index deletable/rebuildable;
- document local retention separately from normalized Store privacy guarantees.

Local cache-root prerequisite accepted: `tokn-platform::observer_shadow_index_root()` resolves under the existing Observer data root (`%LOCALAPPDATA%/Tokn/Observer/shadow-index` on Windows), and `observer_shadow_project_root(project_id)` accepts only a safe single cache segment. No backend database filename or SQLite layout is selected yet.

A contentless FTS backend may reduce raw-text duplication, but **must not** be described as cryptographically private or secret-free.

## Backend candidates

No backend is selected by this design.

### DIRECT_SCAN_V0

Reference baseline:
- sequential/bounded scan over the same eligible corpus;
- no persistent lexical index;
- deterministic file-level matching semantics;
- used to establish quality and latency baselines.

Accepted reference implementation (`tokn-platform::direct_scan`):
- requires the authorized root to be the Git top-level directory;
- corpus = `git ls-files --cached --others --exclude-standard --full-name -z`;
- path material must stay relative, contain no traversal/control/backslash/drive-prefix material and canonicalize inside the authorized root;
- per-file candidate is accepted only when it is a regular file, <= configured max size, NUL-free and valid UTF-8;
- hard operational ceilings: 100,000 discovered files, 16 MiB Git file-list output, 16 MiB configured max per file;
- manifest retains only workspace-relative path + `src-v1-*` + `ixc-v1-*` + byte count, never source text;
- query limit is >=1 result, <=4096 query bytes and <=64 unique whitespace terms;
- matching lowercases query/path/content, then requires every unique term to occur in path or current content;
- deterministic ranking: path-term hit count descending, content-term hit count descending, then relative path ascending;
- every query reopens each eligible current file and recomputes `ixc-v1-*`; any mismatch/unreadable/now-ineligible entry fails closed instead of returning a stale candidate (`REFRESH_REQUIRED` for hash divergence);
- returned hits contain no source text.

It should not shell out to an external search binary in the final contract unless explicitly treated as a separate benchmark candidate. `git` is used only for bounded corpus enumeration under the Git-backed V0 contract.

### SQLITE_FTS5_UNICODE61_V0

Candidate lexical index:
- file-level documents;
- BM25/rank candidate ordering;
- contentless/contentless-delete mode preferred for evaluation so the FTS database does not keep a second raw-text copy;
- current file is reopened for any returned source text/snippet;
- exact file hash must be checked before content is returned.

### SQLITE_FTS5_TRIGRAM_V0

Candidate for code identifiers and substring-heavy queries.

It must be measured separately because substring support can improve identifier retrieval while increasing index size and changing short-query behavior.

### Deferred candidates

Not V0:
- custom tokenizer;
- Tree-sitter/LSP symbol index;
- dependency graph;
- embeddings/vector search;
- reranker model;
- fuzzy semantic equivalence.

Add them only after the measurement protocol identifies a specific recall/value gap.

## FTS capability probe

Implementation must probe actual SQLite FTS5 support at runtime/build validation before selecting an FTS candidate.

Accepted capability implementation lives in the dedicated `tokn-shadow` crate so SQLite-specific derived-index mechanics do not contaminate `tokn-platform` or the normalized Measurement Store. `tokn-observe shadow-index-capabilities` exposes the report as JSON.

The probe uses only an in-memory SQLite connection and real ephemeral operations. It observes separately:
- SQLite version and `ENABLE_FTS5` compile option;
- basic FTS5 create/insert/MATCH behavior;
- `unicode61` tokenizer;
- `trigram` tokenizer;
- contentless tables;
- contentless-delete behavior;
- FTS5 integrity-check command.

Capability states reuse the frozen benchmark vocabulary: `OBSERVED_AVAILABLE`, `OBSERVED_UNAVAILABLE`, `NOT_APPLICABLE`, `UNKNOWN`. Probe/runtime failures are represented as `UNKNOWN` plus static reason codes; raw SQLite error text is not part of the report. The report always declares `extension_loading_attempted=false` and `fallback_backend_id=DIRECT_SCAN_V0`.

Current bundled validation observes SQLite 3.53.2 with every probed FTS5 capability above available. This is build/runtime evidence only, not a backend-selection decision; future builds must probe again.

If FTS5 is unavailable or cannot be proven:
- do not fail the whole Tokn product;
- keep the FTS candidate unavailable/unknown;
- keep DIRECT_SCAN_V0 available for measurement/fallback;
- do not dynamically load untrusted extensions to make the benchmark pass.

## Index sync semantics

Index freshness is separate from model-context freshness.

Use explicit states such as:
- `SYNCED_OBSERVED`: current file bytes match the indexed manifest hash;
- `REFRESH_REQUIRED`: a directly observed file/hash/Git change invalidates one or more entries;
- `UNKNOWN`: current content cannot be verified;
- `UNSUPPORTED_V0`: repository/capability is outside the V0 contract.

Do **not** call these states `FRESH` / `STALE` without the `INDEX_`/sync qualifier.

They say nothing about what Astra remembers or needs.

## Incremental invalidation

Prefer Git-assisted narrowing over full rescans, but exact content identity remains authoritative for index correctness.

Expected refresh flow:
1. compare indexed Git provenance with current repository provenance when available;
2. use Git diff/status to identify tracked changes, additions, removals, renames and non-ignored untracked files;
3. recompute exact index-content hashes for affected files before updating entries;
4. remove deleted/renamed-old documents;
5. insert/update changed documents;
6. if Git provenance is unavailable/ambiguous, fall back to a bounded manifest verification/full rebuild rather than serving unverified content.

Git HEAD equality alone is never enough when either working tree is dirty.

## Query correctness and fallback

A shadow query result is only a candidate.

Before Tokn returns source text derived from an indexed hit:
- resolve the workspace-relative locator under the authorized current root;
- verify it remains inside that root;
- recompute/verify its IndexContentHash;
- if the hash differs, refresh that entry or fall back to direct current-file search;
- never return a stale cached source-text copy from the index.

If correctness cannot be verified, prefer `UNKNOWN`/fallback over a fast wrong answer.

## Retrieval score semantics

BM25/rank/trigram scores are ordering signals only.

They are not:
- confidence that a file is semantically relevant;
- evidence the model needs that file;
- evidence a prior read was wasteful;
- evidence of token savings;
- evidence of quality preservation.

Expose backend score only inside shadow measurement/debug surfaces until a stable goal-level query contract exists.

## Query / product boundary

Do not expose `bm25_search` or `fts_query` as public MCP tools.

A later product surface should express a goal such as `tokn_context_query`, with the backend private and replaceable.

V0 itself has no MCP tool and no active runtime hook.

## Measurement gate

No backend graduates from candidate status until `docs/benchmarks/SHADOW-INDEX-PROTOCOL-V0.md` has been run on frozen corpora with predeclared quality and resource budgets.

Required dimensions:
- retrieval quality;
- build cost;
- query latency;
- peak memory where observable;
- index size;
- incremental refresh cost;
- mutation/delete/rename correctness;
- privacy/publication checks.

The direct-scan baseline and indexed candidates must use the same corpus and query gold set.

## Non-claims

Shadow Repository Index V0 does not prove:
- fewer model tokens;
- fewer necessary reads;
- better task quality;
- semantic relevance beyond the benchmark gold set;
- safe context injection;
- source freshness in model context;
- runtime memory/retention.

Experiment 003 remains the first causal optimization A/B after a reproducible finding and an actual intervention.

## External technical evidence

SQLite FTS5 official documentation is the source for the backend capabilities considered here:
- BM25/rank ordering;
- contentless and contentless-delete indexes;
- unicode61 and trigram tokenizers;
- FTS integrity-check behavior.

Reference: https://www.sqlite.org/fts5.html

This reference supports candidate capability design only. It does not establish that FTS5 is the best backend for Tokn; that decision belongs to the measurement gate.
