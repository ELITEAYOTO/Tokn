# Tokn V0.1 - Next Session Checklist

Derniere mise a jour : **2026-10-01**

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

Prochain scope :
- definir/persister uniquement l'activite tool/file privacy-safe necessaire ;
- activity/phase timeline ;
- repeated reads/searches/retries ;
- shared/duplicate evidence parent/subagent ;
- compaction/rediscovery quand directement observable ;
- comparaison explicite multi-run.

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
- GUI.

Ces sujets restent apres V0.1 et doivent venir d'un finding mesure.

## Regle quota

P7-P9 sont offline/replay.
Aucun quota Astra n'est requis.

## M4 continuation after activity timeline acceptance

Do not rebuild the accepted activity foundation.

Automatic ModelRuntimeProfile compatibility reducer: ACCEPTED FOUNDATION.
Rate-limit snapshot ingestion + RateLimitHistory V1: ACCEPTED FOUNDATION.
Cross-Agent Evidence V1 exact-operation overlap: ACCEPTED FOUNDATION.
Result identity remains `NOT_CAPTURED`; operation overlap is not duplicate evidence.

Next concrete work:
1. result-identity evidence when directly observable;
2. compaction/rediscovery only when directly observed;
3. explicit cross-run comparison.

After M4, prefer shadow retrieval/edit analysis before any active Context Compiler behavior.
