# Tokn V0.1 - Next Session Checklist

Derniere mise a jour : **2026-10-01**

## Etat de depart

P0-P9 : DONE
Experiment 002 : DONE / ACCEPTED
Measurement Contract V1 : DONE / FROZEN
Store foundation + ModelRuntimeProfile persistence : DONE
Local MCP integration prototype : NEXT

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

Implementer le **Local MCP integration prototype**.

Premier scope :
- process MCP local lance a la demande ;
- handshake/version/capabilities structures ;
- doctor/status ;
- lecture evidence/history via le Store commun ;
- appel des fonctions Runner/Store existantes sans dupliquer leur logique ;
- erreurs structurees ;
- aucun daemon permanent par defaut.

Aucun nouveau quota Astra/Codex n'est necessaire pour ce travail.

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

NEXT :
Local MCP integration prototype.

Puis :
- Historical Analyzer + Context Ledger ;
- findings observation-only ;
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
