# Long-term AutoLab / Intelligence Layer review - 2026-10-03

Status: ACCEPTED STRATEGIC REVIEW / NOT AN IMPLEMENTATION CLAIM

## Scope

This review reconciles the four owner-supplied long-term research documents with the current Tokn repository and roadmap.

Documents reviewed:
- architecture/data/ML/compute guide;
- token optimizers comparison and strategy;
- AutoLab adaptive optimization RFC;
- Intelligence Layer innovation design.

Repository state remains authoritative for what exists today.
The research documents are architectural direction, not evidence that a proposed optimizer already produces savings.

## Executive decision

The research is useful and directionally strong.
It does not replace Tokn's existing North Star; it sharpens it.

Tokn remains:
- evidence-first;
- local-first;
- quality-constrained;
- Astra-semantics-preserving;
- causal before claiming savings.

The material change is sequencing: Tokn needs explicit context/evidence identity and opportunity analysis before active optimization or ML.
## Accepted roadmap changes

### 1. Context Identity / Context Twin seed moves earlier

The current M4 boundary already says result identity is NOT_CAPTURED.
The next implementation should solve that as a reusable identity foundation, not as a one-off duplicate counter.

V0 should support only directly observable evidence:
- privacy-safe evidence/context ID;
- project-scoped keyed content/result fingerprint when raw content is transiently available;
- source identity and range/symbol when known;
- source version/hash/commit when known;
- provenance;
- freshness/invalidation state;
- agent/run distribution when directly observed.

It must not claim that Astra still remembers an item merely because it was previously supplied.
Coverage remains UNKNOWN/NOT_CAPTURED where the host cannot prove delivery or freshness.

Do not persist raw tool output by default just to obtain identity.
Prefer transient hashing and durable privacy-safe fingerprints.

### 2. Opportunity Analyzer becomes part of M5

Findings alone are insufficient to choose engineering priorities.
Each candidate should separate:
- observed addressable surface;
- theoretical upper bound;
- realistic bounded potential;
- frequency;
- evidence confidence;
- quality/preservation risk;
- implementation complexity;
- experiment cost/priority.

A valid outcome is DEPRIORITIZE.
This prevents Tokn from spending months optimizing a category that is only a tiny share of the real run cost.

### 3. Counterfactual replay and policy schema precede expensive A/B

Before Experiment 003 and later experiments, Tokn should have a small deterministic offline layer:
- native baseline retained explicitly;
- replayable candidate intervention;
- PolicyCandidate / PolicyGenome V0 with schema, scope, runtime constraints, provenance and evidence references;
- policy lifecycle states;
- shadow result separate from causal result.

Offline estimates remain estimates, never PROVEN SAVINGS.

### 4. AutoLab is a later research/product track

AutoLab should not be pulled into M4-M6 implementation scope.
Its first version should be offline-first and data-disciplined:
- Experience Bank reconstructed from normalized Store data where possible;
- Feature Store separating observed features from derived labels;
- DatasetManifest;
- time/project/runtime holdouts;
- deterministic replay;
- drift detection;
- Policy Registry;
- native baseline.

Only after enough evidence should Tokn study:
1. contextual bandits;
2. Bayesian optimization;
3. learning-to-rank;
4. surrogate models;
5. evolutionary search offline;
6. optional specialized LLM analyst.

No neural model or Tokn LLM is a prerequisite.

## Accepted but deferred ideas

Useful directions, not current critical-path work:
- full Evidence Graph;
- Semantic Impact Engine;
- Shared Evidence Blackboard;
- Tokn IR / Tool Compiler;
- predictive prefetch;
- embeddings;
- full API proxy backend;
- evolutionary policy search;
- specialized local LLM.
These should enter implementation only after an Opportunity analysis or a measured prerequisite justifies them.

## Event sourcing decision

The event-sourcing idea is strategically useful but does not justify rewriting the current Store now.

Decision:
- keep new persisted contracts versioned and replay/rebuild friendly;
- prefer derived views that can be recomputed from normalized evidence;
- record provenance and schema versions;
- fail closed on unknown schemas;
- defer a wholesale event-store migration until a concrete limitation of the existing Store is measured.

This preserves most benefits without destabilizing M4.

## Data lifecycle decision

The research strengthens an existing concern: retention must become a first-class design topic before AutoLab or broad distribution.

Rules:
- raw evidence: bounded/configurable retention;
- normalized privacy-safe Store: durable where justified;
- derived experiences/features: versioned and rebuildable when possible;
- datasets: manifest + filters/exclusions + schema/runtime distribution;
- purge/export/migration must exist before treating the Store as a long-lived learning corpus.

Do not build an infinite raw Astra archive.

## What does not change

Still rejected as default optimizations:- lower reasoning effort just to save tokens;
- arbitrary output/context caps;
- fewer subagents merely because they cost tokens;
- skipped final tests;
- blocked Python/direct reads;
- compression without expand/fallback;
- active learned policy without causal validation;
- one opaque scalar reward such as `-tokens`.

Quality remains a constraint; efficiency is optimized only inside the acceptable-quality region.

## Real gain assessment

Near term, these documents do not themselves save tokens.
They add architecture and therefore some implementation cost.

The likely strategic gain is high because they improve where Tokn spends effort:
- prevent low-addressable optimizers from being built;
- make duplicate evidence and rediscovery measurable instead of guessed;
- allow cheap offline filtering before expensive real A/B;
- preserve runtime drift and baseline semantics;
- create reusable structured data for later learning.

The highest-potential mechanisms remain avoidance of unnecessary context creation/re-discovery and better targeted code access, not generic terminal compression.

No percentage saving should be promised before Tokn measures addressable surface and completes causal A/B.

## Immediate next implementation after this review

Resume M4 with Context/Result Identity Foundation V0 as the next slice.
It should be deliberately small and observation-only, and seed the future Context Twin without prematurely building the full Intelligence Layer.