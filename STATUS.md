# Tokn Observer - Etat du projet

Derniere mise a jour : **2026-09-29 19:01 +02:00**

## KPI principal

Tokn ne cherche pas a rendre Astra plus rapide.

KPI principal :
**maximiser le travail utile et la qualite obtenue par token consomme.**

Un run plus long est acceptable si la qualite est egale ou meilleure
et si le quota permet davantage de travail utile.

## Version actuelle

Binaire actuel :
Tokn Observer V0.0

Prochaine release de travail :
**V0.1 Measurement Hardening**

V0.1 est prioritaire avant toute nouvelle optimisation active.

Audit maintenabilite 2026-09-29 :
**CLEAN / NO MAJOR REFACTOR REQUIRED**.

Nettoyage preventif realise :
- fallback session-root centralise dans la CLI ;
- duplication retiree de analyze-run / check-caps / resolve-workspace ;
- dependance serde_json inutile retiree de tokn-analysis ;
- replay Experiment 001 conserve les memes resultats apres refactor.

Maintenance baseline :
- repository Git local initialise dans V0-CodexTkn-Consume/tool ;
- branche principale locale : main ;
- aucun remote configure ni push effectue ;
- le workflow CI est maintenant versionne avec le code.

Progression V0.1 :
**P0-P5 DONE ; P6 Policy placement/evidence IN PROGRESS.**

P6 en cours :
- types de domaine + aggregation RunGroup : PASS ;
- scanner instructions + lectures explicites AGENTS.md : PASS ;
- `inspect-policy` texte/JSON : PASS ;
- integration FINISH Experiment 001 : PASS ;
- replay reel : hint PRESENT, instructions 0/4, repository reads 4/4 (5 lectures), observed FAIL 58 cibles / 17 violations, enforcement NOT_PROVEN ;
- placement generique au vrai workspace Codex + recherche hook d'enforcement : RESTENT A FAIRE.

## Ce qui est valide aujourd'hui

Build/tooling :
- cargo fmt : PASS ;
- Clippy -D warnings : PASS ;
- tests workspace : PASS ;
- build release : PASS ;
- packaging : PASS.

Session rollouts :
- decouverte reelle : PASS ;
- ingestion JSONL : PASS ;
- token_usage_record : PASS ;
- SQLite : PASS ;
- idempotence : PASS.

Diagnostic Rollout Trace :
- baseline reelle 2026-09-26 : PASS ;
- totaux Tokn == trace-reduce sur cette baseline ;
- attribution outils V0 : PASS sur cette baseline.

Simulation :
- raw vs cap-adjusted : PASS ;
- simulate-caps : PASS ;
- check-caps : implementation presente.

V0.1 Measurement Hardening :
- P0 fixtures sanitisees : PASS ;
- P1 source health + fallback : PASS ;
- fallback historique base sur session_meta.timestamp, mtime seulement en secours ;
- sessions et archived_sessions indexes ;
- P2 RunGroup parent + descendants : PASS ;
- P3 terminal INCOMPLETE_USAGE_LIMIT : PASS ;
- P4 session tool adapter : PASS, y compris plusieurs exec_command dans une enveloppe Code Mode ;
- cap policy : PASS / FAIL / NO_EVIDENCE / INCOMPLETE_EVIDENCE ;
- Experiment 001 cap replay : 58 outils cibles, 17 violations, 0 unknown, 0 parse failure => FAIL ;
- P5 dynamic workspace tracking : PASS ;
- resolver Experiment 001 => B07-C_WORKING\PROJECT SELECTED ;
- watch-root generique exclu du scoring ;
- runner START capture workspace-before ; FINISH resout le vrai workspace avant diff/verify ;
- `verify:local` cible le workspace resolu ;
- `analyze-run --output-json` : PASS sur Experiment 001, 1 parent + 3 sous-agents ;
- `session-evidence.json` derive maintenant du RunGroup exact, sans scan `LastWriteTime` ;
- AGENTS.md temporaire est nettoye automatiquement dans un workspace copie uniquement quand la preuve est exacte ; sinon Tokn preserve et avertit ;
- START dry-run complet : PASS et cleanup confirme ;
- build release courant : PASS.

## Baseline JEM 2026-09-26

Diagnostic healthy.

- input : 3,316,765 ;
- cached : 3,168,896 ;
- uncached : 147,869 ;
- output : 16,062 ;
- reasoning : 3,125 ;
- logical total : 3,332,827 ;
- tools : 26 ;
- raw tool tokens : 64,219 ;
- cap-adjusted upper : 47,191.

Voir :
docs/baselines/2026-09-26-jem-ultimate.md

## Experiment 001 - resultat reel

Verdict :
**INSTRUMENTATION DISCOVERY SUCCESS**
**OPTIMIZATION EXPERIMENT INVALID**

Le run a reellement travaille sur JEM mais a termine par usage_limit_exceeded.

4 sessions liees :
- 1 parent ;
- 3 sous-agents.

Totaux recuperes depuis les session rollouts :
- usage records : 80 ;
- input : 5,285,737 ;
- cached : 5,059,712 ;
- uncached : 226,025 ;
- output : 26,021 ;
- reasoning : 3,018 ;
- logical total : 5,311,758.

Parent :
2,499,523 logical (~47.1 %).

Sous-agents :
2,812,235 logical (~52.9 %).

Terminal :
INCOMPLETE_USAGE_LIMIT.

## Experiment 001 - problemes Tokn decouverts

1. Diagnostic trace partiel
Le bundle candidat ne contenait que 4 evenements de protocole
et aucun usage/tool exploitable.

2. Faux PASS policy
0 outil observe a ete interprete comme compliant.
C'est interdit en V0.1.

3. Mauvais workspace surveille
Tokn suivait B07-B_WORKING/PROJECT.
Astra a correctement cree et travaille dans B07-C_WORKING.

4. Mauvais quality-gate target
Le finisher historique a verifie le projet B07-B d'origine.
Le vrai B07-C a ensuite ete verifie manuellement et PASS.

5. Policy non enforcee
Le AGENTS.md temporaire n'etait pas dans base_instructions.
La plupart des appels exec ne declaraient aucun max_output_tokens.

6. Sous-agents non agreges automatiquement
Plus de la moitie du cout logique etait dans les descendants.

## Etat reel B07-C apres quota

Comparaison B07-B_FROZEN -> B07-C_WORKING :
- 13 fichiers ajoutes ;
- 4 modifies ;
- 0 supprimes.

verify:local sur le vrai B07-C :
PASS.

Tests :
814 PASS, contre 809 au depart.

La tache reste incomplete :
le Quick Fix Framework complet n'a pas ete implemente avant epuisement du quota.

## Priorite unique actuelle

**Continuer Tokn V0.1 a partir de P6 avant de relancer une experience d'optimisation.**

Termine :
P0 fixtures ;
P1 source health/fallback ;
P2 RunGroup + sous-agents ;
P3 terminal status ;
P4 session tools/caps ;
P5 dynamic workspace.

Suite :
P6 policy evidence ;
P7 experiment validity ;
P8 runner V0.1 ;
P9 validation/golden replay ;
P10 Experiment 002.

Source de verite :
docs/plans/V0.1-IMPLEMENTATION-PLAN.md

## Interdictions jusqu'a V0.1

Ne pas :
- annoncer une economie observee depuis Experiment 001 ;
- traiter une absence de preuve comme zero ;
- afficher PASS avec zero cible ;
- comparer parent seul contre un run multi-agent ;
- lancer Experiment 002 avant P0-P9 ;
- ajouter RAG/embeddings/GUI avant stabilisation de la mesure.

## Documents a lire

1. docs/INDEX.md
2. docs/experiments/001-POSTMORTEM.md
3. docs/plans/V0.1-IMPLEMENTATION-PLAN.md
4. docs/plans/V0.1-TEST-MATRIX.md
5. docs/design/V0.1-MEASUREMENT-ARCHITECTURE.md
