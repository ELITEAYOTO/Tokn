# Tokn Documentation Index

Derniere mise a jour : 2026-09-29 19:01 +02:00

## A lire en premier

1. ../STATUS.md
2. ../ROADMAP.md
3. experiments/001-POSTMORTEM.md
4. plans/V0.1-IMPLEMENTATION-PLAN.md
5. plans/V0.1-TEST-MATRIX.md
6. design/V0.1-MEASUREMENT-ARCHITECTURE.md
7. experiments/002-INSTRUMENTATION-VALIDATION.md

Ces documents suffisent pour reprendre le developpement V0.1 sans relire l'historique de chat.

Etat de reprise actuel :
**P0-P5 sont implementes et valides. P6 Policy placement/evidence est en cours.**

## Architecture et invariants

- architecture.md : architecture generale V0.
- token-accounting.md : regles de comptabilite tokens.
- diagnostic-trace.md : statut et limites du Rollout Trace diagnostic.
- attribution-engine.md : attribution outils/contexte.
- design/V0.1-MEASUREMENT-ARCHITECTURE.md : architecture cible V0.1.
- design/FIRST-TOOL-ANALYZER-VISION.md : vision long terme du premier outil Tokn.
- MAINTAINABILITY-AUDIT-2026-09-29.md : audit structurel, dettes acceptees et seuils de revue.
- design/EVIDENCE-AND-COVERAGE.md : PASS/FAIL/NO_EVIDENCE et couverture.
- design/SESSION-ROLLOUT-ADAPTER.md : adapter session standard.
- design/AGENT-COST-ATTRIBUTION.md : cout parent/sous-agents.
- design/WORKSPACE-TRACKING.md : suivi des workspaces crees/copied.
- design/POLICY-MODEL.md : hint vs observed vs enforced.
- design/EXPERIMENT-VALIDITY.md : validite machine des experiences.
## Decisions

- decisions/ADR-001-V0.1-EVIDENCE-FALLBACK.md
  Les session rollouts sont une source de preuve de premier rang et fallback obligatoire.

- decisions/ADR-002-NO-EVIDENCE-IS-NOT-PASS.md
  Zero observation ne peut jamais produire PASS.

## Experiences

- baselines/2026-09-26-jem-ultimate.md
  Premiere baseline diagnostic validee.

- experiments/001-runtime-output-caps.md
  Conception initiale des caps offline.

- experiments/001-RUNBOOK.md
  Runner Experiment 001 historique.

- experiments/001-POSTMORTEM.md
  Source de verite sur les resultats reels d'Experiment 001.

- experiments/002-INSTRUMENTATION-VALIDATION.md
  Prochain run, uniquement apres V0.1 P0-P9.

## Plans

- plans/V0.1-IMPLEMENTATION-PLAN.md
  Ordre de developpement P0-P10 avec criteres de sortie.

- plans/V0.1-TEST-MATRIX.md
  Tests unitaires, integration et golden replay obligatoires.

## Maintenance

- MAINTENANCE.md
  Discipline de mise a jour et reprise.
- privacy.md
  Confidentialite locale.
- compatibility.md
  Compatibilite Codex.
