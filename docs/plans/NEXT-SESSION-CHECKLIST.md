# Tokn V0.1 - Next Session Checklist

Derniere mise a jour : **2026-10-03**

## Etat de depart

P0-P9 : DONE
Experiment 002 : DONE / ACCEPTED
Measurement Contract V1 : DONE / FROZEN
Store foundation + ModelRuntimeProfile persistence : DONE
Local MCP transport prototype : DONE / ACCEPTED
Historical Analyzer + Context Ledger : IN PROGRESS

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

Continuer **M4 Historical Analyzer + Context Ledger** en observation-only.

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

Prochain scope :
- SourceStableId seulement quand une identite logique stable est directement observable ;
- source version + freshness/invalidation sans stockage brut par defaut ;
- compaction/rediscovery uniquement quand directement observable ;
- comparaison explicite multi-run avec coverage/comparability explicites.

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
Historical Analyzer + Context Ledger (M4 IN PROGRESS).

Puis, seulement apres M4 :
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

## M4 continuation after activity timeline acceptance

Do not rebuild the accepted activity foundation.

Automatic ModelRuntimeProfile compatibility reducer: ACCEPTED FOUNDATION.
Rate-limit snapshot ingestion + RateLimitHistory V1: ACCEPTED FOUNDATION.
Cross-Agent Evidence V2 exact-operation + result-identity comparison: ACCEPTED FOUNDATION.
Exact content/result identity is now `OBSERVED` only for unambiguous one-operation/one-result evidence; ambiguous or unavailable cases remain explicit.
SourceStableId file V0 + SourceIdentityHistory V1: ACCEPTED FOUNDATION.
`source-identity-history` reports exact observed content evolution without claiming freshness, staleness or safe reuse.

Next concrete work:
1. source version / directly observed invalidation timing and broader source kinds only where provable;
2. provenance/freshness semantics with `UNKNOWN`/`NOT_CAPTURED` preserved;
3. compaction/rediscovery only when directly observed;
4. explicit cross-run comparison with compatible evidence scope.

Optional parallel hardening: measure current raw tool-result retention first; only if the cost is non-trivial, replace `BTreeMap<String, Vec<String>>` retention with a private transient fingerprint accumulator/visitor that preserves exact fingerprints and coverage. No Store/Measurement migration and no general buffer subsystem in M4.

Do not create `tokn-runtime-api`, Claude/OpenCode adapters or a public Adapter SDK in this slice. Before runtime #2: Runtime Adapter Contract V1, Token Semantics V2, Runtime Capability Manifest V1 and sanitized conformance fixtures.

The identity slice is the seed of Context Twin V0, not the full Intelligence Layer.
After M4, M4.5 builds Context Identity + shadow retrieval/edit foundations before M5 Findings + Opportunity Analyzer.
Do not start Experience Bank, Feature Store, bandits, Bayesian optimization or a Tokn LLM in the current M4 scope.
