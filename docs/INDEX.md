# Tokn Documentation Index

Derniere mise a jour : **2026-10-01**

Ce fichier est le point d'entree documentaire unique.

## Reprendre le developpement en 5 documents

Lire seulement :
1. `../STATUS.md` - ou en est le projet ;
2. `../ROADMAP.md` - ordre des prochaines phases ;
3. `plans/V0.1-IMPLEMENTATION-PLAN.md` - comment finir V0.1 ;
4. `plans/V0.1-TEST-MATRIX.md` - gates obligatoires ;
5. `plans/NEXT-SESSION-CHECKLIST.md` - prochaine action concrete.

Etat actuel :
**P0-P9 DONE ; Experiment 002 ACCEPTED ; Measurement Contract V1 FROZEN ; Store foundation DONE ; Local MCP transport prototype ACCEPTED ; Historical Analyzer + Context Ledger IN PROGRESS.**

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
- `design/MEASUREMENT-CONTRACT-V0.1.md`
  Contrat V1 fige apres Experiment 002 : schemas, evidence layout et semantiques de mesure.
- `design/TOKN-STORE-V2.md`
  Fondation SQLite V2 : identities pseudonymisees, runs/agents/profiles/provenance et ingestion `store-evidence`.
- `design/HISTORICAL-CONTEXT-LEDGER.md`
  M4 en cours : HistoricalSnapshot/Context Ledger V1, coverage, lineage, CLI/MCP et limites explicites.
- `design/CROSS-AGENT-EVIDENCE.md`
  Cross-Agent Evidence V1 : chevauchement exact d'operations privacy-safe entre threads, lineage et garde-fou `NOT_CAPTURED` sur l'identite du resultat.
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
- `design/PLUGIN-ENGINE-INTEGRATION.md`
  Transport local valide : adapter MCP stdio process-bound, sans daemon impose.
- `design/LOCAL-MCP-PROTOTYPE.md`
  Implementation `tokn-mcp`, validations release + Codex runtime et exposition read-only du Context Ledger.
- `design/MODEL-RUNTIME-PROFILE.md`
  Identite runtime/modele, capacites observees et telemetry de rate limits versionnee.
- `strategy/CONTEXT-EFFICIENCY-STRATEGY.md`

## Research

- `research/2026-09-29-CONTEXT-CACHE-RESEARCH.md`
  Synthese revue : cache, contexte, Astra, Experiment 001, backlog de mesure.
- `research/2026-09-30-CODEX-PLUGIN-TELEMETRY-RESEARCH.md`
  Verification ciblee des surfaces telemetry, plugins, skills, hooks, MCP et quota.
- `research/OPENAI-CODEX-CAPABILITY-MATRIX.md`
  Capacites OBSERVED / RESEARCH / UNKNOWN par runtime.
- `research/OPEN-QUESTIONS.md`
  Registre compact des inconnues ayant un impact architectural/analytique.
- `plans/RESEARCH-ROADMAP.md`
  Ordre et statut des recherches externes.
- `plans/IMPLEMENTATION-PATH.md`
  Chemin d'execution de P8 jusqu'aux couches Store, MCP, analyzers et experiments causaux.

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

## Audit / design review 2026-10-02

- `audits/2026-10-02-CLAUDE-HANDOFF-REVIEW.md` - current-code review of the Claude handoff and accepted/deferred recommendations.
- `audits/2026-10-03-LONG-TERM-AUTOLAB-REVIEW.md` - review of owner-supplied AutoLab, Intelligence Layer, data/ML and optimizer strategy research; accepted sequencing changes and deferred ideas.
- Long-term Context Compiler / shadow guardrails are folded into ROADMAP, TARGET-ARCHITECTURE and CONTEXT-EFFICIENCY-STRATEGY instead of duplicating a second canonical design document.
