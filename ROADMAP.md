# Tokn Roadmap

Derniere mise a jour : **2026-10-01**

## North Star

Maximiser le travail utile et la qualite obtenus par token,
sans reduire arbitrairement la capacite d'Astra.

Ordre obligatoire :
**mesurer -> prouver -> expliquer -> experimenter -> optimiser -> automatiser.**

## M0 - Observer V0.0

Statut : DONE

Acquis :
Codex discovery, JSONL, token accounting, SQLite, diagnostic trace,
attribution V0, simulations offline et premier runner experimental.

## M1 - Experiment 001

Statut : DONE / POST-MORTEM

Valeur :
decouverte des failles de mesure.

Pas de conclusion d'optimisation autorisee.

Golden facts :
voir `docs/experiments/001-POSTMORTEM.md`.

## M2 - V0.1 Measurement Hardening

Statut : DONE

P0 sanitized fixtures : DONE
P1 source health/fallback : DONE
P2 RunGroup/agent graph : DONE
P3 terminal status : DONE
P4 session tool adapter : DONE
P5 dynamic workspace : DONE
P6 policy evidence/runtime capability : DONE
P7 experiment validity reducer : DONE
P8 runner V0.1 : DONE
P9 golden replay/release validation : DONE

### P6 closure

P6 a etabli :
- hint != observation != enforcement ;
- soft policy n'est jamais presentee comme enforced ;
- hooks Desktop/Code Mode sont reellement mediatisees ;
- PreToolUse Bash 0.158 ne recoit pas max_output_tokens ;
- etat correct : SUPPORTED_INSUFFICIENT_INPUT.

Decision produit :
ne pas poursuivre les hard output caps comme axe principal.
La dette JSONL live du probe est non bloquante et reste au backlog.

Plan d'execution detaille depuis P8 jusqu'aux couches post-V0.1 :
`docs/plans/IMPLEMENTATION-PATH.md`.

## M3 - Experiment 002

Statut : DONE / ACCEPTED

But :
valider V0.1 end-to-end sur une petite tache reelle.

Resultat :
- Runner COMPLETE ;
- terminal COMPLETED ;
- quality PASS, 863/863 tests ;
- 2 agents / 49 usage records ;
- workspace correct et diff 1/2/0 ;
- modele `gpt-6.1-sol` observe ;
- configuration_recorded=UNKNOWN ;
- validity INSTRUMENTATION_ONLY ;
- aucune conclusion causale autorisee.

Le FINISH reel a revele un probleme de layout de trace CLI et un probleme
`$LASTEXITCODE` PowerShell. Les preuves ont ete recuperees sans relancer le modele,
puis le fallback session a ete versionne et regression-teste.

## M3.5 - Measurement Contract Freeze

Statut : DONE / FROZEN V1

Resultat :
- `measurement-contract.json` emis par Runner ;
- versions explicites des schemas/evidence/semantiques ;
- Runner refuse les versions workspace/snapshot inconnues ;
- token accounting V1 fige (`uncached = input - cached`, cache-write separe) ;
- ModelRuntimeProfile V1 fige au niveau domaine ;
- replay Experiment 001 verifie toutes les versions sans changer les golden facts.

## M3.6 - Store + ModelRuntimeProfile persistence

Statut : NEXT

But :
persister le minimum fiable pour projects/workspaces, runs, agents,
usage summaries, provenance et ModelRuntimeProfile V1.

## M3.7 - Local MCP integration prototype

Statut : PLANNED AFTER M3.6

But :
exposer le moteur/store local via un adapter MCP process-bound mince,
sans dupliquer la logique analytique et sans daemon permanent par defaut.

## M4 - Historical Analyzer + Context Ledger

Statut : PLANNED AFTER STORE FOUNDATION

But :
construire l'historique fiable par projet/run/agent/phase.

Capacites :
- ProjectIdentity et WorkspaceLineage ;
- historique multi-run ;
- cached/uncached/input/output/reasoning par agent/tour ;
- timeline de phases ;
- file/tool activity graph ;
- repeated reads/searches/retries ;
- parent/subagent shared evidence ;
- terminal + quality history.

## M5 - Context Efficiency Findings

Statut : PLANNED

Observation-only.

Findings cibles :
- uncached growth anormal ;
- cache churn quand observable ;
- repeated file/search evidence ;
- contexte duplique parent/sous-agents ;
- instructions globales qui provoquent des lectures inutiles ;
- tool evidence repetee ;
- project facts rediscovered entre runs ;
- compaction/context-management suivie de rediscovery.

Chaque finding :
preuve + provenance + confiance + cout observe + frequence +
impact potentiel borne + risque qualite + proposition A/B.

## M6 - Experiment Lab / Experiment 003

Statut : PLANNED

Experiment 003 sera le premier A/B causal d'optimisation.

La variable sera choisie a partir d'un finding mesure.
Ce n'est plus par defaut un output cap.

Conditions :
- meme tache ;
- workspace initial gele ;
- runtime/model/config comparables ;
- une variable principale ;
- qualite acceptee avant comparaison tokens ;
- terminal status compatible ;
- validity = VALID_FOR_CAUSAL_AB.

## M7 - Advisor

Statut : FUTURE

Transforme les findings historiques en recommandations.
Aucune modification automatique.

## M8 - Context Compiler / Project Memory

Statut : FUTURE / ACTIVE LAYER

Seulement apres validation repetee des findings :
- contexte structure ;
- project memory avec provenance/fraicheur/invalidation ;
- evidence compression si prouvee ;
- cache-aware construction si la surface le permet ;
- plugin/daemon/GUI si utiles.

Cette couche reste separee de l'Analyzer.

## Regle de passage

Une phase est DONE uniquement si :
- tests techniques PASS ;
- evidence suffisante ;
- documentation synchronisee ;
- aucune affirmation plus forte que la preuve ;
- aucune regression des golden facts.
