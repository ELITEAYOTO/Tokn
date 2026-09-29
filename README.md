# Tokn Observer

Tokn est un profiler local de Codex/Astra destine a mesurer puis reduire
la consommation de tokens sans reduire la qualite ni le travail utile.

Version binaire actuelle : **V0.0** (workspace Cargo 0.0.0)
Version de travail : **V0.1 Measurement Hardening**

Etat V0.1 au 2026-09-29 :
**P0-P5 implementes et valides ; P6 Policy placement/evidence est en cours.**

## Principe

observer -> verifier la preuve -> mesurer -> comparer -> seulement ensuite optimiser

Tokn refuse maintenant comme principe de projet :
- de transformer UNKNOWN en zero ;
- de traiter NO_EVIDENCE comme PASS ;
- d'annoncer une economie sans experience valide ;
- d'ignorer les sous-agents ;
- de supposer qu'un workspace de sortie est le workspace d'entree.

## KPI

Le KPI n'est pas la vitesse.

Le KPI est :
**travail utile et qualite obtenus par token consomme.**

Un run plus long peut etre meilleur s'il realise davantage de travail
avec le meme quota ou moins de tokens.

## Ce que V0.0 sait deja faire

- decouvrir Codex Desktop et ses sessions ;
- lire les rollouts JSONL en streaming ;
- importer token_usage_record ;
- maintenir des invariants token ;
- stocker des agregats SQLite ;
- importer un vrai diagnostic Rollout Trace ;
- verifier une baseline avec trace-reduce ;
- attribuer temporellement outils et croissance d'input ;
- distinguer raw tool output et cap-adjusted upper bound ;
- simuler des caps offline ;
- capturer un run experimental avec scripts start/finish/recover.

## Ce qu'Experiment 001 a revele

Experiment 001 n'a pas valide l'optimisation.

Il a revele que :
- un diagnostic trace peut etre partiel ;
- les standard session rollouts peuvent contenir le vrai run complet ;
- plus de 50 % du cout logique peut venir des sous-agents ;
- le task terminal status doit etre capture ;
- un run peut creer son propre workspace de sortie ;
- AGENTS.md est un policy hint, pas une enforcement garantie ;
- zero observation ne peut pas prouver la compliance.

Post-mortem :
docs/experiments/001-POSTMORTEM.md

## Priorite actuelle

Continuer **Tokn V0.1** a partir de **P6 - Policy placement and evidence**.

P0-P5 sont maintenant implementes et valides offline/replay :
- fixtures sanitisees ;
- source health + fallback diagnostic -> sessions ;
- RunGroup parent + sous-agents ;
- terminal status ;
- session tools/caps + NO_EVIDENCE/FAIL/INCOMPLETE_EVIDENCE ;
- dynamic workspace tracking + resolver fail-closed ;
- fallback historique stable via `session_meta.timestamp` + `archived_sessions` ;
- `analyze-run --output-json` pour une preuve parent/sous-agents machine-readable ;
- `FINISH` derive `session-evidence.json` du vrai RunGroup, sans heuristique `LastWriteTime` ;
- nettoyage conservateur du `AGENTS.md` temporaire dans un workspace copie par Astra.

Plan :
docs/plans/V0.1-IMPLEMENTATION-PLAN.md

Matrice de tests :
docs/plans/V0.1-TEST-MATRIX.md

Architecture cible :
docs/design/V0.1-MEASUREMENT-ARCHITECTURE.md

## Prochain run Astra

Ne pas lancer Experiment 002 tant que V0.1 P0-P9 n'est pas valide.

Experiment 002 servira uniquement a valider l'instrumentation end-to-end.

Plan :
docs/experiments/002-INSTRUMENTATION-VALIDATION.md

Le premier vrai A/B causal est reserve a Experiment 003.

## Build et validation

Depuis :
E:\Tokn\V0-CodexTkn-Consume\tool

Utiliser :
- scripts\test.ps1
- scripts\build-release.ps1
- scripts\package.ps1

Binaire release :
target\release\tokn-observe.exe

## Donnees runtime

Donnees Tokn :
%LOCALAPPDATA%\Tokn\Observer

Preuves Codex :
jamais modifiees par Tokn.

## Architecture de crates

- tokn-domain
- tokn-platform
- tokn-ingest
- tokn-codex
- tokn-analysis
- tokn-storage
- tokn-report
- app tokn-observe

Les formats Codex restent derriere adapters.

## Documentation

Commencer par :
docs/INDEX.md

Puis :
1. STATUS.md
2. ROADMAP.md
3. docs/experiments/001-POSTMORTEM.md
4. docs/plans/V0.1-IMPLEMENTATION-PLAN.md
5. docs/plans/V0.1-TEST-MATRIX.md
6. docs/design/V0.1-MEASUREMENT-ARCHITECTURE.md

## Scope du premier outil a terme

Apres V0.1, Tokn Observer doit evoluer en Analyzer historique :
- detection automatique des projets/workspaces ;
- historique local des runs et sessions ;
- historique technique des discussions sans stockage brut obligatoire ;
- timeline des phases de travail ;
- attribution tokens par agent, outil, fichier et phase ;
- detection des relectures, recherches dupliquees, retries et rework ;
- comparaison multi-run ;
- findings d'optimisation avec preuve, confiance et impact potentiel.

Vision :
docs/design/FIRST-TOOL-ANALYZER-VISION.md

Pas avant validation de la mesure :
- Tree-sitter ;
- RAG ;
- embeddings ;
- project memory active ;
- context compiler ;
- patch engine ;
- daemon ;
- GUI ;
- optimisation active.

L'Analyzer mesure et recommande. L'Optimizer actif reste une couche separee.
