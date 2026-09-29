# Tokn Attribution Engine

Derniere mise a jour : 2026-09-29

## But

Relier les couts modele aux activites locales sans transformer
une correlation temporelle en causalite certaine.

## V0.0

Le moteur sait analyser un diagnostic bundle healthy et produire :
- inferences ;
- deltas input ;
- uncached input ;
- tools ;
- raw tool tokens ;
- cap-adjusted upper bounds ;
- request bytes/items ;
- top windows.

Baseline JEM 2026-09-26 :
validation reelle PASS.

## Limite revelee par Experiment 001

Le diagnostic bundle candidat etait PARTIAL.
Les vrais appels existaient dans les standard session rollouts.

V0.1 doit donc attribuer depuis plusieurs surfaces.

## V0.1 sources

Diagnostic trace healthy :
- inference-level details ;
- referenced payloads ;
- raw/result token metadata quand disponible.

Standard session rollout :
- token_usage_record ;
- custom_tool_call ;
- custom_tool_call_output ;
- parent/subagent relationships ;
- task terminal status.

Filesystem observer :
- workspaces crees ;
- fichiers modifies ;
- quality target.

## Agent attribution

Le rapport doit separer :
- parent cost ;
- descendant cost ;
- per-agent tools ;
- duplicated reads/searches ;
- concurrent spans.

Experiment 001 :
les sous-agents representaient environ 52.9 % du logical total.

Ils deviennent donc un axe de mesure prioritaire.

## Cap evidence

Pour une categorie cible :
- cap explicite conforme ;
- cap explicite violation ;
- cap absent/illisible = UNKNOWN.

Zero appels cibles :
NO_EVIDENCE.

Des appels avec cap inconnu :
INCOMPLETE_EVIDENCE.

## Model-visible versus runtime-only

V0.1 continue de distinguer :
- donnees brutes produites localement ;
- borne model-visible quand connue ;
- contexte logique compte par provider.

Ces valeurs ne sont pas interchangeables.

## Prochaine couche d'analyse

Apres V0.1 et Experiment 002 :
- cached/uncached par agent/tour ;
- relectures/search dupliquees ;
- evidence partagee parent/sous-agents ;
- repeated discovery inter-run ;
- tool evidence footprint.

Ce sont d'abord des findings observation-only.
Ne pas reduire automatiquement le nombre d'agents ni la capacite du modele.

Voir : strategy/CONTEXT-EFFICIENCY-STRATEGY.md
