# Tokn V0.1 - Next Session Checklist

Derniere mise a jour : **2026-09-29**

## Etat de depart

P0-P6 : DONE
P7 : NEXT
P8-P9 : TODO
Experiment 002 : BLOCKED

Workspace :
`E:\Tokn\V0-CodexTkn-Consume\tool`

## Lecture minimale

1. `../../STATUS.md`
2. `../../ROADMAP.md`
3. `V0.1-IMPLEMENTATION-PLAN.md`
4. `V0.1-TEST-MATRIX.md`
5. `../design/EXPERIMENT-VALIDITY.md`

Ne pas refaire l'enquete P6.

## Premiere action

Implementer **P7 - Experiment validity reducer**.

Le reducer doit consommer :
- source health ;
- RunGroup coverage ;
- terminal status ;
- workspace resolution ;
- policy evidence ;
- runtime/model/task comparability ;
- quality gate.

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

## Apres P7

P8 :
runner self-contained, aucune reparation forensique manuelle.

P9 :
fmt + clippy + tests + release + privacy + golden replay.

Puis seulement :
Experiment 002 instrumentation validation.

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
