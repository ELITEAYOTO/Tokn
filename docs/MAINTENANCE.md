# Tokn Documentation and Maintenance

Derniere mise a jour : **2026-10-03**

## Objectif

Une nouvelle session doit pouvoir reprendre Tokn sans relire les chats
et sans reconstruire l'histoire du projet.

## Hierarchie documentaire

`STATUS.md`
Etat courant uniquement. Pas de plan detaille.

`ROADMAP.md`
Ordre des milestones uniquement. Pas de journal historique.

`docs/plans/*`
Execution concrete d'une release ou phase.

`docs/design/*`
Invariants et architecture. Pas de statut quotidien.

`docs/decisions/*`
Decisions durables et consequences.

`docs/research/*`
Recherche, sources, hypotheses et inconnues.
Une recherche n'est pas automatiquement une decision.

`docs/experiments/*`
Protocoles et resultats d'experiences.

`docs/baselines/*`
Snapshots de reference lies aux experiences.

`docs/benchmarks/*`
Protocoles benchmark et discipline de mesure. Les runs bruts restent hors Git.

`docs/reference/*`
Compatibilite, confidentialite et reference stable.

`docs/operations/*`
Workflows operatoires reproductibles, y compris historiques.

`docs/audits/*`
Audits dates. Ils ne redefinissent jamais l'etat courant.

`docs/examples/*`
Exemples non canoniques.

La racine `docs/` reste volontairement minimale :
`INDEX.md` et `MAINTENANCE.md` seulement, sauf exception documentee.

`CHANGELOG.md`
Historique date des modifications.

## Sources de verite

Point d'entree : `docs/INDEX.md`.

En cas de conflit :
1. evidence runtime/golden ;
2. ADR accepte ;
3. STATUS pour l'etat ;
4. ROADMAP pour l'ordre ;
5. plan de release ;
6. design ;
7. research ;
8. ancien runbook/changelog.

Ne jamais laisser un vieux runbook redefinir l'etat courant.

## Discipline apres changement significatif

Code :
0. lancer `scripts/dev-check.ps1` avant PR (`-Full` pour le gate local complet) ;
1. formatter ;
2. Clippy -D warnings ;
3. tests workspace ;
4. golden/fixtures si concernes ;
5. release build si surface CLI ;
6. privacy/package check si concernes.

Docs :
1. mettre a jour le document technique directement concerne ;
2. STATUS si l'etat reel change ;
3. ROADMAP seulement si l'ordre/statut d'une phase change ;
4. CHANGELOG pour la trace ;
5. INDEX si un document durable est ajoute/supprime ;
6. ADR si une decision produit/architecture durable change ;
7. regenerer un Context Pack apres un jalon documentaire/produit significatif via `scripts/context/build-context-pack.ps1`.

Eviter de recopier le meme paragraphe dans cinq documents.

## Regles de preuve

- UNKNOWN reste UNKNOWN ;
- NO_EVIDENCE != PASS ;
- source health avant confiance ;
- parent + descendants = RunGroup ;
- terminal status accompagne les tokens ;
- diagnostic PARTIAL declenche fallback ;
- evidence originale jamais modifiee ;
- OBSERVED / DERIVED / INFERRED / UNKNOWN restent distincts ;
- aucune economie causale sans validity suffisante.

## Regles d'optimisation

KPI :
travail utile + qualite par token.

Ne jamais declarer une optimisation uniquement parce que :
- le run a moins de tokens ;
- la sortie est plus courte ;
- le nombre de sous-agents baisse ;
- le cache ratio monte ;
- la latence baisse.

Toute intervention sur contexte/memoire/compaction doit definir
un quality gate avant de comparer les tokens.

Decision :
`decisions/ADR-003-QUALITY-PRESERVING-EFFICIENCY.md`.

## Maintenabilite code

- tokn-domain reste independant de Codex/SQLite/CLI ;
- adapters ne produisent pas de conclusions d'optimisation ;
- partager une logique avant sa troisieme duplication ;
- experiment runners = orchestration, pas couche metier ;
- module Rust > ~350 lignes : revue responsabilites ;
- PowerShell > ~300 lignes : revue ; > ~400 : extraction avant croissance ;
- pas de micro-crates sans frontiere durable.

Audit :
`audits/2026-09-29-MAINTAINABILITY.md`.

## Confidentialite

Par defaut :
- pas de lecture auth.json ;
- pas de secrets sandbox ;
- pas de prompt complet en SQLite ;
- pas de sortie terminal brute en SQLite ;
- pas d'appel reseau depuis Tokn Observer ;
- fixtures reelles sanitisees avant commit.

## Reprise

Etat :
**P0-P9 DONE ; Experiment 002 ACCEPTED ; Measurement Contract V1 FROZEN ; Store foundation DONE ; Local MCP transport prototype ACCEPTED ; M4 Historical Analyzer + Context Ledger CORE ACCEPTED / EVIDENCE-GATED EXTENSIONS ; M4.5 Context Identity + Shadow Foundations STARTED / OBSERVATION-ONLY.**

Lire :
`INDEX.md` -> `../STATUS.md` -> `../ROADMAP.md` -> `plans/IMPLEMENTATION-PATH.md` -> `plans/NEXT-SESSION-CHECKLIST.md` -> `operations/DEVELOPMENT-WORKFLOW.md`.

Le protocole baseline descriptif est `benchmarks/BASELINE-PROTOCOL-V1.md`.
Ne jamais utiliser un ancien runbook/changelog comme etat courant.
