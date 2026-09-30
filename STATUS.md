# Tokn Observer - Status

Derniere mise a jour : **2026-09-30**
Release de travail : **V0.1 Measurement Hardening**
Binaire/Cargo : **V0.0 / 0.0.0**

## Resume

**P0-P7 DONE. P8 est NEXT. P9 reste a faire.**

Le projet est revenu sur une direction unique :
mesurer l'efficacite du contexte sans reduire la capacite d'Astra.

Hard output caps :
**depriorises comme strategie d'optimisation**.
Ils restent une evidence/diagnostic historique.

## Validation technique courante

P7 full gate :
- cargo fmt --check : PASS ;
- Clippy workspace -D warnings : PASS ;
- cargo test --workspace : PASS ;
- build release tokn-observe : PASS ;
- real CLI Experiment 001 validity replay : PASS ;
- verdict INSTRUMENTATION_ONLY ;
- causal_claims_allowed=false ;
- descriptive_metrics_allowed=true.

Build environment note:
Visual Studio Community 2026 is incomplete for native C builds,
but the already-installed Visual Studio Build Tools 2022 toolchain is complete
and is the validated build environment.

Maintainability audit : CLEAN / NO MAJOR REFACTOR REQUIRED.
Git local : main, aucun remote.

## V0.1

P0 - sanitized fixtures : DONE
P1 - source health + fallback : DONE
P2 - RunGroup + agent graph : DONE
P3 - terminal task status : DONE
P4 - session tool adapter : DONE
P5 - dynamic workspace tracking : DONE
P6 - policy evidence/runtime capability research : DONE
P7 - experiment validity reducer : DONE
P8 - runner V0.1 : NEXT
P9 - golden replay/release validation : TODO

P6 est ferme avec une limitation connue :
Codex Desktop 0.158.0-alpha.2.1 ne transmet pas `max_output_tokens`
au PreToolUse Bash de unified exec.

La sonde live JSONL a zero ligne reste une dette diagnostique,
mais elle ne bloque plus V0.1 car l'enforcement des caps n'est pas un objectif produit.

## Golden Experiment 001

Verdict :
**INSTRUMENTATION DISCOVERY SUCCESS / OPTIMIZATION INVALID**

RunGroup :
- 1 parent ;
- 3 sous-agents ;
- 80 usage records.

Tokens :
- input 5,285,737 ;
- cached 5,059,712 ;
- uncached 226,025 ;
- output 26,021 ;
- reasoning 3,018 ;
- logical total 5,311,758.

Attribution :
- parent 2,499,523 (~47.1 %) ;
- descendants 2,812,235 (~52.9 %).

Etat :
- terminal INCOMPLETE_USAGE_LIMIT ;
- diagnostic PARTIAL ;
- policy observed FAIL 58/17 ;
- policy enforcement SUPPORTED_INSUFFICIENT_INPUT ;
- workspace B07-C_WORKING\PROJECT ;
- diff 13 added / 4 modified / 0 removed ;
- verify:local PASS, 814 tests vs 809 au depart ;
- experiment validity attendue : INSTRUMENTATION_ONLY.

## Interpretation cache

Experiment 001 a ~95.72 % de cached input.

Conclusion autorisee :
la majorite des input tokens reportes etait classee cached.

Conclusions interdites :
- le contexte etait optimal ;
- les cached tokens etaient gratuits ;
- ils ne comptaient pas dans le quota produit ;
- tout le contexte etait utile ;
- il faut reduire la fenetre ou la sortie d'Astra.

La prochaine couche d'analyse doit localiser l'uncached growth,
la duplication et la rediscovery avant de proposer une intervention.

## Direction produit

Decision :
`docs/decisions/ADR-003-QUALITY-PRESERVING-EFFICIENCY.md`

Strategie :
`docs/strategy/CONTEXT-EFFICIENCY-STRATEGY.md`

Recherche revue :
`docs/research/2026-09-29-CONTEXT-CACHE-RESEARCH.md`

Axes apres V0.1 :
- Context Ledger ;
- Cache Efficiency Analyzer ;
- Context Duplication Analyzer ;
- Parent/Subagent Context Graph ;
- Tool Evidence Footprint ;
- Project Memory Candidate Detector.

Tous commencent observation-only.

## Prochaine action

Implementer **P8 - Runner V0.1 self-contained**.

Chemin d'execution detaille :
`docs/plans/IMPLEMENTATION-PATH.md`.

P7 fournit maintenant le contrat de validite que le runner doit appeler :
- input machine-readable ;
- five verdict reducer ;
- structured reasons ;
- causal claim gate ;
- Experiment 001 golden fixture ;
- `evaluate-validity` text + JSON.

Objectif P8 :
un seul run produit un dossier de preuves complet sans reparation forensique manuelle.

Ensuite :
P9 golden/release -> Experiment 002.

Aucun nouveau run Astra n'est necessaire pour P7-P9.

## Sources de verite

Etat : `STATUS.md`
Ordre : `ROADMAP.md`
Implementation V0.1 : `docs/plans/V0.1-IMPLEMENTATION-PLAN.md`
Tests : `docs/plans/V0.1-TEST-MATRIX.md`
Documentation map : `docs/INDEX.md`
