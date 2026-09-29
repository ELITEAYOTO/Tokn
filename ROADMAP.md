# Tokn Roadmap

Derniere mise a jour : **2026-09-29 21:23 +02:00**

## Vision

Tokn doit augmenter le travail utile obtenu par quota de tokens
sans reduire la qualite d'Astra.

Ordre de construction :
**mesurer correctement -> prouver -> optimiser -> automatiser.**

## Milestone A - Observer V0.0

Statut : **DONE / EN PRODUCTION EXPERIMENTALE**

Acquis :
- workspace Rust multi-crates ;
- Codex discovery ;
- streaming JSONL ;
- token accounting ;
- SQLite ;
- diagnostic bundles ;
- trace-reduce oracle ;
- attribution V0 ;
- simulate-caps ;
- check-caps ;
- runner Experiment 001.

Limite :
V0.0 suppose encore trop fortement que le diagnostic trace et le workspace initial
representent tout le run.

## Milestone B - Experiment 001

Statut : **DONE / POST-MORTEM COMPLETE**

Resultat :
- utile pour decouvrir les failles de mesure ;
- invalide pour conclure sur l'optimisation.

Decouvertes majeures :
- diagnostic trace peut etre partiel ;
- session rollouts contiennent le vrai run ;
- sous-agents peuvent depasser 50 % du cout ;
- workspace peut etre cree dynamiquement ;
- usage_limit_exceeded doit etre un statut terminal ;
- policy hint != policy enforcement ;
- NO_EVIDENCE != PASS.

Document :
docs/experiments/001-POSTMORTEM.md

## Milestone C - Tokn V0.1 Measurement Hardening

Statut : **IN PROGRESS / P0-P5 DONE / P6 IN PROGRESS**

### P0 - Fixtures privees -> sanitisees
Statut : DONE

### P1 - Source health + fallback
Statut : DONE

### P2 - RunGroup + agent graph
Statut : DONE

### P3 - Terminal task status
Statut : DONE

### P4 - Standard session tool adapter
Statut : DONE

### P5 - Dynamic workspace tracking
Statut : DONE

Validation finale : resolver fail-closed, replay historique stable, START/FINISH integres, quality gate sur workspace resolu, RunGroup JSON et session evidence exacte.

### P6 - Policy model/evidence
Statut : IN PROGRESS

Deja implemente :
- modeles PolicyPlacement / SessionPolicyEvidence / PolicyEvidenceReport ;
- policy_hint / policy_observed / policy_enforced separes dans le rapport ;
- scanner rollout pour signature dans les instructions et lectures explicites AGENTS.md ;
- aggregation de la preuve sur toutes les sessions du RunGroup ;
- commande `inspect-policy` avec sortie texte + JSON ;
- integration FINISH Experiment 001 ;
- placement/restauration multi-racine par hash sur PROJECT + workspace watch root ;
- START/FINISH/RECOVER retrocompatibles avec les anciens runs ;
- prototype `PreToolUse` privacy-first avec audit JSONL compact ;
- replay reel : hint PRESENT, 0 instruction threads, 4 repository-read threads, observed FAIL 58/17 ;
- hooks Desktop reellement trusted/active ; lifecycle hook observe pendant les exec Code Mode ;
- source OpenAI taggee `rust-v0.158.0-alpha.2.1` : PreToolUse `Bash` ne recoit que `tool_input.command` ;
- `max_output_tokens` existe dans l'appel Code Mode mais est retire avant le callback ;
- enforcement classe SUPPORTED_INSUFFICIENT_INPUT pour ce mecanisme/runtime ;
- cap absent dans le prototype => UNOBSERVABLE/fail-open.

A terminer :
- corriger la sonde live qui produit encore zero ligne malgre le lifecycle hook prouve ;
- rechercher/prototyper un autre point de mediation supporte si un hard cap runtime reste necessaire ;
- conserver la compatibilite versionnee : ne jamais supposer que le contrat d'un futur Codex est identique a 0.158 ;
- ne passer a SUPPORTED_NOT_ACTIVE / ENFORCED qu'avec preuve runtime correspondante.

### P7 - Experiment validity reducer
Statut : TODO

### P8 - Runner V0.1
Statut : TODO

### P9 - Golden replay + release validation
Statut : TODO

Plan detaille :
docs/plans/V0.1-IMPLEMENTATION-PLAN.md

## Milestone D - Experiment 002

Statut : **BLOCKED BY V0.1 P0-P9**

But :
valider l'instrumentation end-to-end sur une vraie petite tache Astra.

Ce run ne doit pas encore servir a prouver des economies.

Document :
docs/experiments/002-INSTRUMENTATION-VALIDATION.md

## Milestone E - Experiment 003

Statut : **PLANNED**

Premier vrai A/B causal.

Conditions :
- meme workspace initial gele ;
- meme tache ;
- meme runtime/model/config ;
- run controle ;
- run candidat avec UNE variable d'optimisation ;
- qualite acceptee avant comparaison tokens.

## Milestone F - Historical Analyzer (PREMIER OUTIL)

Statut : **FUTURE APRES V0.1**

But :
transformer l'Observer en profiler historique du travail Astra/Codex.

Capacites cibles :
- ProjectIdentity independante d'un chemin unique ;
- historique local des runs par projet ;
- historique technique des sessions/discussions ;
- timeline et phases de travail ;
- parent/sous-agents ;
- graphe fichiers/outils/workspaces ;
- attribution tokens par agent, outil, phase et projet ;
- detection des relectures, recherches dupliquees, retries et rework ;
- comparaison de runs et regressions.

Document :
docs/design/FIRST-TOOL-ANALYZER-VISION.md

## Milestone G - Optimization Advisor (PREMIER OUTIL)

Statut : **FUTURE APRES HISTORICAL ANALYZER**

Le premier outil detecte et explique des opportunites sans les appliquer automatiquement.

Chaque finding doit porter :
- preuve et provenance ;
- confiance ;
- cout observe ;
- frequence historique ;
- impact potentiel borne ;
- risque qualite ;
- proposition d'experience A/B.

Axes possibles :
- sorties outils trop larges ;
- lectures/search dupliquees ;
- contexte recharge entre agents ;
- retries inutiles ;
- cache inefficace ;
- builds/tests repetitifs ;
- workspaces copies inutilement.

Aucun gain n'est annonce sans mesure A/B.

## Milestone H - Context Compiler / Project Memory / Active Optimizer

Statut : **FUTURE / COUCHE SEPAREE**

Apres validation des findings du premier outil :
- Tree-sitter ;
- repo graph ;
- local embeddings si utiles ;
- memory project ;
- context IR ;
- budget solver ;
- patch engine ;
- macro-tools ;
- daemon ;
- GUI/control center ;
- optimisations actives.

Cette couche reste separee de l'Analyzer afin de ne pas contaminer la mesure.

## Regle de roadmap

Une phase ne passe a DONE que si :
- tests techniques PASS ;
- evidence reelle suffisante ;
- documentation mise a jour ;
- aucune affirmation plus forte que les preuves disponibles.
