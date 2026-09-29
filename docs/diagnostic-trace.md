# Tokn - Diagnostic Rollout Trace

Derniere mise a jour : 2026-09-27

## Etat reel

Tokn sait importer un vrai diagnostic Rollout Trace complet.

Baseline JEM 2026-09-26 :
- HEALTHY de fait ;
- usages tokens lus ;
- outils lus ;
- totaux identiques a trace-reduce.

Experiment 001 2026-09-27 :
- diagnostic bundle present ;
- trace.jsonl : 4 records seulement ;
- rollout_started : 1 ;
- thread_started : 1 ;
- protocol_event_observed : 2 ;
- inference/tool usage : 0.

Ce bundle est **PARTIAL**.

## Nouvelle regle V0.1

La presence de manifest.json + trace.jsonl ne suffit plus a declarer une trace exploitable.

Tokn doit calculer SourceHealth :
- HEALTHY ;
- PARTIAL ;
- EMPTY ;
- MALFORMED ;
- UNSUPPORTED.

Une trace PARTIAL declenche un fallback vers les standard session rollouts
pour les familles de metriques disponibles.

## Reducer officiel

trace-reduce reste un oracle de corroboration quand le bundle diagnostic
contient les donnees necessaires.

Il n'est pas le moteur metier de Tokn.

## Provenance

Tokn conserve :
- source file ;
- offsets/lines ;
- source hash ;
- source kind ;
- health status ;
- coverage.

Voir :
docs/design/EVIDENCE-AND-COVERAGE.md
docs/decisions/ADR-001-V0.1-EVIDENCE-FALLBACK.md
