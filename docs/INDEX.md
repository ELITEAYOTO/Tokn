# Tokn Documentation Index

Derniere mise a jour : **2026-09-30**

Ce fichier est le point d'entree documentaire unique.

## Reprendre le developpement en 5 documents

Lire seulement :
1. `../STATUS.md` - ou en est le projet ;
2. `../ROADMAP.md` - ordre des prochaines phases ;
3. `plans/V0.1-IMPLEMENTATION-PLAN.md` - comment finir V0.1 ;
4. `plans/V0.1-TEST-MATRIX.md` - gates obligatoires ;
5. `plans/NEXT-SESSION-CHECKLIST.md` - prochaine action concrete.

Etat actuel :
**P0-P7 DONE ; P8 NEXT.**

## Organisation documentaire

- `decisions/` - decisions durables (ADR) ;
- `design/` - architecture, contrats et invariants techniques ;
- `plans/` - execution concrete des phases/releases ;
- `strategy/` - direction produit et principes d'optimisation ;
- `research/` - recherches, sources, hypotheses et inconnues ;
- `experiments/` - protocoles, runbooks et resultats ;
- `baselines/` - snapshots de reference ;
- `reference/` - compatibilite, confidentialite et reference stable ;
- `operations/` - workflows operatoires reproductibles ;
- `audits/` - audits dates et historiques ;
- `examples/` - exemples non canoniques.

Seuls `INDEX.md` et `MAINTENANCE.md` restent directement dans `docs/`.

## Sources de verite par sujet

Etat courant -> `../STATUS.md`
Roadmap -> `../ROADMAP.md`
Plan V0.1 -> `plans/V0.1-IMPLEMENTATION-PLAN.md`
Golden Experiment 001 -> `experiments/001-POSTMORTEM.md`
Strategie optimisation -> `strategy/CONTEXT-EFFICIENCY-STRATEGY.md`
Regles de preuve -> `design/EVIDENCE-AND-COVERAGE.md`
Validite experimentale -> `design/EXPERIMENT-VALIDITY.md`
Maintenance documentaire -> `MAINTENANCE.md`

Un document historique ne doit pas remplacer une source de verite courante.

## Decisions

- `decisions/ADR-001-V0.1-EVIDENCE-FALLBACK.md`
  Session rollouts = fallback obligatoire quand diagnostic insuffisant.
- `decisions/ADR-002-NO-EVIDENCE-IS-NOT-PASS.md`
  Zero observation ne peut pas devenir PASS.
- `decisions/ADR-003-QUALITY-PRESERVING-EFFICIENCY.md`
  Optimiser le contexte inutile, pas la capacite du modele.
- `decisions/ADR-004-EXPERIMENT-VALIDITY.md`
  Seule une preuve experimentale suffisante autorise une conclusion causale.
- `decisions/ADR-005-HYBRID-TOKN-ARCHITECTURE.md`
  Le moteur Rust reste la source de verite analytique ; les integrations restent des couches minces.

## Design - mesure

- `design/ARCHITECTURE-OVERVIEW.md`
- `design/V0.1-MEASUREMENT-ARCHITECTURE.md`
- `design/SESSION-ROLLOUT-ADAPTER.md`
- `design/AGENT-COST-ATTRIBUTION.md`
- `design/WORKSPACE-TRACKING.md`
- `design/EVIDENCE-AND-COVERAGE.md`
- `design/EXPERIMENT-VALIDITY.md`
- `design/TOKEN-ACCOUNTING.md`
- `design/DIAGNOSTIC-TRACE.md`
- `design/ATTRIBUTION-ENGINE.md`

## Design - policy/runtime

- `design/POLICY-MODEL.md`
- `design/CODEX-HOOK-ENFORCEMENT.md`

Le document hook est une conclusion de recherche P6.
Il n'est plus la direction principale d'optimisation.

## Design - produit futur

- `design/FIRST-TOOL-ANALYZER-VISION.md`
- `design/TARGET-ARCHITECTURE.md`
  Architecture cible Engine / Store / adapters / integrations.
- `strategy/CONTEXT-EFFICIENCY-STRATEGY.md`

## Research

- `research/2026-09-29-CONTEXT-CACHE-RESEARCH.md`
  Synthese revue : cache, contexte, Astra, Experiment 001, backlog de mesure.
- `research/OPENAI-CODEX-CAPABILITY-MATRIX.md`
  Capacites OBSERVED / RESEARCH / UNKNOWN par runtime.
- `research/OPEN-QUESTIONS.md`
  Registre compact des inconnues ayant un impact architectural/analytique.
- `plans/RESEARCH-ROADMAP.md`
  Ordre des recherches externes sans bloquer P8/P9.

Les research docs peuvent contenir des hypotheses.
Elles ne deviennent des decisions qu'apres ADR/ROADMAP.

## Experiences

Historique :
- `baselines/2026-09-26-jem-ultimate.md`
- `experiments/001-runtime-output-caps.md`
- `experiments/001-RUNBOOK.md`
- `experiments/001-POSTMORTEM.md`

Prochaine :
- `experiments/002-INSTRUMENTATION-VALIDATION.md`

Le document `001-runtime-output-caps.md` est historique.
Il ne represente plus la strategie produit.

## Maintenance / reference / operations

- `MAINTENANCE.md`
- `reference/COMPATIBILITY.md`
- `reference/PRIVACY.md`
- `reference/CONTEXT-PACK-MANIFEST.template.md`
  Template de generation des snapshots documentaires portables.
- `reference/WORKSPACE-LAYOUT.md`
  Separation entre repo canonique, preuves runtime, artefacts, launchers et archives.
- `operations/JEM-TRACE-WORKFLOW.md` (historique)
- `audits/2026-09-29-MAINTAINABILITY.md` (snapshot historique)

## Regle anti-dispersion

Avant de creer un nouveau document :
1. verifier qu'un document canonique n'existe pas deja ;
2. choisir sa classe : status / roadmap / plan / design / decision / research / experiment ;
3. ne pas dupliquer l'etat courant dans un design historique ;
4. l'ajouter ici uniquement s'il devient une reference durable.

Les chats ne sont jamais la source de verite du projet.
