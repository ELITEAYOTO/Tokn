# Maintenance du projet Tokn

Derniere mise a jour : 2026-09-29 19:01 +02:00

## Objectif

Une nouvelle session de developpement doit pouvoir reprendre Tokn
sans dependre de la memoire d'une conversation.

## Sources de verite

Ordre de lecture :
1. docs/INDEX.md
2. STATUS.md
3. ROADMAP.md
4. docs/experiments/001-POSTMORTEM.md
5. docs/plans/V0.1-IMPLEMENTATION-PLAN.md
6. docs/plans/V0.1-TEST-MATRIX.md

## Documents obligatoires a maintenir

- README.md : usage et priorite actuelle ;
- STATUS.md : etat reel valide/manquant ;
- ROADMAP.md : ordre des milestones ;
- CHANGELOG.md : modifications datees ;
- docs/INDEX.md : carte documentaire ;
- docs/architecture.md : architecture implemente/cible ;
- docs/token-accounting.md : invariants ;
- docs/diagnostic-trace.md : evidence diagnostic ;
- docs/attribution-engine.md : attribution ;
- docs/privacy.md : confidentialite ;
- docs/compatibility.md : compatibilite Codex.

Pour V0.1 maintenir aussi :
- docs/design/* ;
- docs/plans/* ;
- docs/experiments/* ;
- docs/decisions/*.

## Discipline apres modification significative

1. formatter ;
2. Clippy warnings interdits ;
3. tests workspace ;
4. fixtures/golden replay si concerne ;
5. build release si surface CLI change ;
6. package privacy check ;
7. CHANGELOG ;
8. STATUS ;
9. ROADMAP si phase change ;
10. doc technique concernee.

Une fonctionnalite n'est pas DONE parce qu'elle compile.

## Garde-fous de maintenabilite

Architecture :
- conserver tokn-domain independant de Codex, SQLite et CLI ;
- ne pas mettre de conclusions d'optimisation dans les adapters ;
- centraliser une logique partagee avant sa troisieme duplication ;
- garder les experiment runners comme orchestration, pas comme couche metier permanente.

Seuils de revue :
- module Rust > ~350 lignes de production : revue des responsabilites ;
- script PowerShell > ~300 lignes : revue ; > ~400 lignes : extraire avant nouvelle croissance ;
- tests colocalises autorises si le module reste coherent ;
- taille totale d'un fichier seule != raison suffisante pour refactorer.

Robustesse :
- nouveaux unwrap/expect/panic en production seulement si invariant explicite ;
- toute nouvelle abstraction doit supprimer une duplication reelle ou isoler une responsabilite durable ;
- pas de micro-crates sans frontiere architecturale claire.

Audit courant :
docs/MAINTAINABILITY-AUDIT-2026-09-29.md

## Regles de preuve

- UNKNOWN reste UNKNOWN ;
- NO_EVIDENCE n'est jamais PASS ;
- source health avant confiance dans les metriques ;
- parent seul n'est pas le run complet si subagents existent ;
- terminal status accompagne toujours les tokens ;
- diagnostic trace partiel declenche fallback ;
- evidence originale jamais modifiee.

## Regles d'experimentation

Avant tout A/B :
- task capturee ;
- workspace de depart connu ;
- output workspace resolu ;
- runtime/model versions enregistres ;
- policy level connu ;
- quality gate correct ;
- terminal status connu ;
- experiment validity calculee.

Aucune economie n'est declaree si le verdict n'autorise pas une comparaison causale.

## Confidentialite

Par defaut :
- pas de lecture auth.json ;
- pas de secrets sandbox ;
- pas de prompt complet en SQLite ;
- pas de sortie terminal brute en SQLite ;
- pas d'appel reseau ;
- runtime Tokn sous %LOCALAPPDATA%\Tokn\Observer.

Les fixtures V0.1 issues de vrais rollouts doivent etre sanitisees
avant d'entrer dans le repository.

## Reprise de developpement

Etat actuel :
- V0.1 P0-P5 : DONE ;
- prochaine action officielle : **V0.1 P6 - Policy placement and evidence**.

Ne pas recommencer P0-P5 sauf regression.
Ne pas commencer par Experiment 002.

Points de reprise importants :
- diagnostic PARTIAL doit fallback vers sessions ;
- session discovery couvre sessions + archived_sessions ;
- fallback historique utilise session_meta.timestamp avant mtime ;
- RunGroup inclut parent + descendants ;
- NO_EVIDENCE n'est jamais PASS ;
- workspace resolver est fail-closed et doit preceder quality gate ;
- session evidence doit venir du RunGroup exact, pas d'une fenetre `LastWriteTime` ;
- une policy experimentale copiee dans un nouveau workspace ne doit etre retiree automatiquement que si son identite est prouvee.

Checklist :
docs/plans/NEXT-SESSION-CHECKLIST.md
