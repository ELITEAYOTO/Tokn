# Tokn Architecture

Derniere mise a jour : 2026-09-27

## V0.0 implemente

Pipeline actuel :

Codex evidence
-> source adapter
-> bounded JSONL ingestion
-> normalized observations
-> token/integrity analysis
-> SQLite aggregates
-> CLI report

Crates :
- tokn-domain : types Tokn et provenance ;
- tokn-platform : chemins/process plateforme ;
- tokn-ingest : streaming JSONL borne ;
- tokn-codex : adapters Codex ;
- tokn-analysis : ledger, invariants, attribution ;
- tokn-storage : SQLite local ;
- tokn-report : rendu ;
- tokn-observe : composition CLI.

## Limite V0.0

V0.0 a trop souvent suppose :
- qu'un diagnostic trace est complet ;
- qu'un run correspond a un seul thread ;
- qu'un workspace reste fixe ;
- qu'une absence de mesure vaut zero.

Experiment 001 a invalide ces hypotheses.

## Architecture cible V0.1

Evidence sources
-> source health
-> source adapters
-> normalized RunEvent IR
-> RunGroup parent/subagents
-> tool/workspace graph
-> token ledger
-> terminal reducer
-> policy evidence reducer
-> experiment validity reducer
-> SQLite/reporting

La source est choisie par famille de metriques.
Une trace partielle ne doit jamais ecraser une session saine.

Details :
docs/design/V0.1-MEASUREMENT-ARCHITECTURE.md

## Evolution long terme du premier outil

Apres V0.1, l'architecture doit pouvoir ajouter sans casser le coeur de mesure :
- ProjectIdentity ;
- WorkspaceLineage ;
- historique local RunGroup/Session ;
- timeline et phases ;
- graphe activite fichiers/outils ;
- attribution avancee ;
- behavior detectors ;
- Finding Engine.

Reference :
docs/design/FIRST-TOOL-ANALYZER-VISION.md

L'Analyzer reste passif par defaut. L'Optimizer actif reste une couche separee.

## Regles stables

- domaine independant de Codex/SQLite/CLI ;
- adapters versionnes autour des formats externes ;
- UNKNOWN reste UNKNOWN ;
- provenance conservee ;
- preuves originales immuables ;
- quelques crates stables, petits modules internes ;
- pas de fichier fourre-tout ;
- aucune conclusion d'optimisation dans les adapters.
