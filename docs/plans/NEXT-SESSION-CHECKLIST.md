# Tokn V0.1 - Next Session Checklist

Derniere mise a jour : **2026-10-03**

## Etat de depart

P0-P9 : DONE
Experiment 002 : DONE / ACCEPTED
Measurement Contract V1 : DONE / FROZEN
Store foundation + ModelRuntimeProfile persistence : DONE
Local MCP transport prototype : DONE / ACCEPTED
Historical Analyzer + Context Ledger : CORE ACCEPTED / EVIDENCE-GATED EXTENSIONS

Workspace :
`E:\Tokn\V0-CodexTkn-Consume\tool`

## Lecture minimale

1. `../../STATUS.md`
2. `../../ROADMAP.md`
3. `V0.1-IMPLEMENTATION-PLAN.md`
4. `V0.1-TEST-MATRIX.md`
5. `../design/EXPERIMENT-VALIDITY.md`
6. `IMPLEMENTATION-PATH.md`
7. `../design/MULTI-RUNTIME-CORE-AND-DATA-ARCHITECTURE.md` - contraintes architecturales a respecter, pas un scope a implementer maintenant

Ne pas refaire l'enquete P6 ni la recherche plugin/telemetry R1-R8 avant qu'un besoin concret de P8/P9 ne le justifie.

## Premiere action

Commencer **M4.5 Context Identity + Shadow Foundations** en observation-only, sans modifier le comportement d'Astra/runtime.

Le core deja valide ne doit pas etre refait :
- HistoricalSnapshot V1 + Store history queries ;
- Context Ledger V1 run/agent coverage-aware ;
- WorkspaceLineage ;
- terminal + quality + validity + ModelRuntimeProfile + provenance ;
- CLI `tokn-observe context-ledger` ;
- MCP read-only `tokn_context_ledger` ;
- Codex 0.161.0-alpha.2 direct tool-call PASS sur thread idle / 0 turns ;
- per-turn = `NOT_CAPTURED` et current retained context = `UNKNOWN`.

Context/Result Identity Foundation V0 : ACCEPTED FOUNDATION.

M4 exit boundary :
- `docs/design/M4-EXIT-GATE.md` est ACCEPTED ; le coeur M4 ne doit plus accumuler des heuristiques pour combler des signaux absents ;
- phase timeline et compaction detaillee restent `NOT_CAPTURED` sans evidence directe fixtureable ;
- runtime delivery, interpreted rediscovery/redundancy/freshness restent `NOT_PROVEN` ;
- Cross-Run Comparability, Task Input Identity et Source Re-read Evidence restent des primitives d'observation, pas des findings.

Premier scope M4.5 : **ACCEPTED PRE-IMPLEMENTATION DESIGN**.
- `SHADOW-REPOSITORY-INDEX.md` fige l'index file-level DERIVED/local/rebuildable, separe du Store ;
- `SHADOW-INDEX-PROTOCOL-V0.md` + ShadowIndexMeasurement V1 figent quality/latency/build/refresh/privacy avant choix backend ;
- DIRECT_SCAN_V0 est la reference ; SQLite FTS5 unicode61/trigram restent seulement des candidats ;
- invalidation Git/hash incrementale + verification exacte avant retour de contenu ;
- aucune injection active de contexte et aucune economie revendiquee.

Aucune heuristique de finding tant que ces observations historiques ne sont pas fiables.
Aucun nouveau quota Astra/Codex n'est requis pour ce travail offline/replay.

Build/test environment valide :
Visual Studio Build Tools 2022 via `vcvars64.bat`.

## P7 acceptance

Experiment 001 doit produire automatiquement :
`INSTRUMENTATION_ONLY`.

Une comparaison non causale doit :
- conserver les metriques descriptives ;
- afficher le verdict ;
- interdire tout wording de winner/savings causal.

Tests obligatoires :
- valid causal fixture ;
- descriptive-only fixture ;
- incomplete task ;
- invalid capture ;
- Experiment 001 golden.

## Golden facts a conserver

- 4 sessions ;
- 80 usage records ;
- logical total 5,311,758 ;
- uncached 226,025 ;
- terminal INCOMPLETE_USAGE_LIMIT ;
- diagnostic PARTIAL ;
- policy observed FAIL 58/17 ;
- enforcement SUPPORTED_INSUFFICIENT_INPUT ;
- workspace B07-C_WORKING\PROJECT ;
- diff 13/4/0 ;
- verify:local PASS ;
- 1 parent + 3 descendants.

## Apres V0.1

CURRENT :
M4.5 Context Identity + Shadow Foundations (STARTED / OBSERVATION-ONLY).

M4 core est ACCEPTED ; ses extensions sans preuve directe restent evidence-gated.

Puis, seulement apres une fondation shadow mesurable :
- findings observation-only (M5) ;
- Experiment 003 seulement apres un finding reproductible.

## Non-goals

Ne pas commencer :
- hard output caps ;
- command rewriting ;
- context truncation ;
- RAG/embeddings ;
- Project Memory active ;
- Context Compiler ;
- plugin d'optimisation ;
- GUI;
- speculative runtime #2 adapter/refactor or public Adapter SDK.

Ces sujets restent apres V0.1 et doivent venir d'un finding mesure.

## Regle quota

P7-P9 sont offline/replay.
Aucun quota Astra n'est requis.

## M4 continuation after workspace Git provenance acceptance

Do not rebuild the accepted activity foundation.

Automatic ModelRuntimeProfile compatibility reducer: ACCEPTED FOUNDATION.
Rate-limit snapshot ingestion + RateLimitHistory V1: ACCEPTED FOUNDATION.
Cross-Agent Evidence V2 exact-operation + result-identity comparison: ACCEPTED FOUNDATION.
Exact content/result identity is now `OBSERVED` only for unambiguous one-operation/one-result evidence; ambiguous or unavailable cases remain explicit.
SourceStableId file V0 + SourceIdentityHistory V1: ACCEPTED FOUNDATION.
`source-identity-history` reports exact observed content evolution without claiming freshness, staleness or safe reuse.
ToolActivityHistory V3 + SourceMutationHistory V1: ACCEPTED FOUNDATION. Mutation-operation timing may be observed; mutation effect remains `NOT_VERIFIED`.
Run-Boundary Source Version V0 + SourceVersionHistory V1: ACCEPTED FOUNDATION. BEFORE/AFTER snapshot versions are directly observed; missing boundaries remain `UNKNOWN`, and no mutation causality/freshness is inferred.
Workspace Git Provenance V0 + WorkspaceGitProvenanceHistory V1: ACCEPTED FOUNDATION. BEFORE/AFTER snapshots now carry directly observed HEAD/dirty evidence when available; legacy absence is `NOT_CAPTURED`, attempted but unprovable Git state is `UNKNOWN`, and raw Git SHA values are not persisted in SQLite.
Source Mutation Window V0: ACCEPTED CHRONOLOGY FOUNDATION. Same-thread exact read-before -> completed mutation -> exact read-after can report exact content equality/difference only under complete unambiguous sequence/identity evidence; intervening or cross-thread same-source mutation makes the window `UNKNOWN`, and causality always remains `NOT_PROVEN`.
Source Freshness Evidence V0: ACCEPTED CORROBORATION FOUNDATION / OBSERVATION-ONLY. It joins Source Mutation Window + SourceVersion BEFORE/AFTER + Workspace Git provenance, reports only observed change/reread corroboration, never compares ContentFingerprint to SourceVersionFingerprint directly, and keeps freshness/invalidation `NOT_PROVEN`.
Cross-Run Comparability V0: ACCEPTED OBSERVATION FOUNDATION / OBSERVATION-ONLY. It compares explicit baseline/candidate runs only on captured project/contract/runtime/SourceVersion BEFORE/Git BEFORE state; project-scoped fingerprints are never compared across projects and causal claims remain `NOT_ESTABLISHED`.
Task Input Identity V0: ACCEPTED OBSERVATION FOUNDATION. `store-evidence --task-input` derives project-scoped `tsk-v1-*` exact artifact identity without persisting raw task/path; `task-input-history` exposes `delivery_status=NOT_PROVEN`, and Cross-Run adds a fail-closed `TASK_INPUT_IDENTITY` axis while causal claims remain `NOT_ESTABLISHED`.
Cross-Run Source Re-read Evidence V0: ACCEPTED CHRONOLOGY FOUNDATION. `source-reread-evidence` orders same project-scoped SourceStableId reads across distinct runs only under complete parseable rollout `observed_at`; Store ingestion time is never runtime chronology and rediscovery/redundancy/freshness remain `NOT_PROVEN`.
Detailed compaction chronology: `NOT_CAPTURED` until a real diagnostic event sample exists; diagnostic-trace seq and session-rollout seq are not assumed comparable.
Stability/benchmark readiness: local Rust toolchain + dev-check + package hardening + Benchmark Baseline Protocol V1 are ACCEPTED/PREPARED; this does not claim token savings.

Next concrete work:
1. resolve exact SourceStableId derivation reuse outside `tokn-storage` without changing any existing `src-v1-*` output; do not duplicate the algorithm;
2. define local shadow-index storage/retention location outside tracked/package artifacts;
3. implement/test DIRECT_SCAN_V0 reference semantics on a sanitized file corpus with bounded enumeration and path-escape protection;
4. add SQLite FTS5 capability probing only after the direct-scan reference exists, then benchmark candidates under the frozen protocol;
5. keep all active context injection disabled and add future M4 phase/compaction/delivery evidence only when directly observable and fixtureable.

Parallel descriptive work allowed now: prepare/collect 6-10 frozen native baseline tasks using `docs/benchmarks/BASELINE-PROTOCOL-V1.md`, starting with a >=3-run variance pilot per task and no active Tokn intervention.
Repository-admin housekeeping: protect `main` so PR + green CI are required before merge.

Optional parallel hardening: measure current raw tool-result retention first; only if the cost is non-trivial, replace `BTreeMap<String, Vec<String>>` retention with a private transient fingerprint accumulator/visitor that preserves exact fingerprints and coverage. No Store/Measurement migration and no general buffer subsystem in M4.

Do not create `tokn-runtime-api`, Claude/OpenCode adapters or a public Adapter SDK in this slice. Before runtime #2: Runtime Adapter Contract V1, Token Semantics V2, Runtime Capability Manifest V1 and sanitized conformance fixtures.

The identity slice is the seed of Context Twin V0, not the full Intelligence Layer.
After M4, M4.5 builds Context Identity + shadow retrieval/edit foundations before M5 Findings + Opportunity Analyzer.
Do not start Experience Bank, Feature Store, bandits, Bayesian optimization or a Tokn LLM in the current M4 scope.
