# Codex Hook Enforcement - Tokn P6

Status: **CLOSED RESEARCH / COMPATIBILITY REFERENCE**
Derniere mise a jour : 2026-09-29

Ce document conserve la conclusion technique P6.
Il ne definit plus la strategie principale d'optimisation Tokn.
Voir `../decisions/ADR-003-QUALITY-PRESERVING-EFFICIENCY.md`.

## Decision

Codex 0.158.0-alpha.2.1 expose une feature `hooks` stable.
`PreToolUse` avec le matcher `Bash` est un point de mediation supporte pour les commandes shell.
Les appels d'outils imbriques via Code Mode passent egalement par cette decision.

Cela prouve qu'une interception runtime supportee existe sans patcher Codex.
La source OpenAI exacte du runtime Desktop `rust-v0.158.0-alpha.2.1` prouve aussi une limite :
`exec_command` conserve `max_output_tokens` pour l'execution, mais construit le payload
PreToolUse uniquement avec `tool_input.command`. Le cap est donc retire avant le hook.
La policy Tokn sur `max_output_tokens` n'est pas enforceable par ce callback sur ce runtime.

Reference publique :
OpenAI Codex Hooks documentation, section PreToolUse / Tool coverage.

## Limite de la conclusion

Un hook disponible n'est pas un hook actif.
Un hook actif n'est pas automatiquement une preuve d'enforcement complete.

Les docs Codex indiquent aussi que certains chemins specialises peuvent
sortir du chemin de hooks standard. L'enforcement Tokn doit donc toujours
etre qualifie par la surface cible declaree.

## Etats Tokn

- UNAVAILABLE : aucun mecanisme supporte n'est disponible pour la surface cible.
- SUPPORTED_UNVERIFIED : un point de mediation existe, mais le contrat necessaire a cette policy n'est pas encore prouve.
- SUPPORTED_INSUFFICIENT_INPUT : le point de mediation existe, mais le runtime ne transmet pas une entree requise par la policy.
- SUPPORTED_NOT_ACTIVE : le contrat policy/runtime a ete valide, mais Tokn ne l'a pas active sur ce run.
- NOT_PROVEN : un mecanisme semble actif mais la preuve du run est insuffisante.
- ENFORCED : preuve run-scoped suffisante que le mecanisme a mediatise la surface cible.

Experiment 001 est maintenant classe SUPPORTED_INSUFFICIENT_INPUT :
- meme runtime local : codex-cli 0.158.0-alpha.2.1 ;
- hooks disponibles ;
- aucun hook Tokn n'etait configure pour ce run ;
- la source taggee de ce runtime prouve que le PreToolUse Bash ne recoit que `command` ;
- `max_output_tokens` n'est donc pas observable a ce point de mediation ;
- le soft policy AGENTS.md reste uniquement un hint.

## Prototype actuel

Commande interne :
`tokn-observe hook-pre-tool-use`

Entree :
objet JSON PreToolUse sur stdin.

Comportement du prototype sur payload synthetique :
- classifie la commande avec le meme classifieur que l'Analyzer ;
- ne cible actuellement que file_read et search ;
- accepte un cap explicite conforme s'il existe dans `tool_input.max_output_tokens` ;
- marque un cap absent UNOBSERVABLE sans bloquer ; refuse uniquement un cap explicitement superieur a la policy ;
- ignore build/test, write, git et autres familles non ciblees.

Important : sur Codex Desktop 0.158.0-alpha.2.1, `tool_input.max_output_tokens`
est prouve absent du payload PreToolUse construit pour unified exec.
Le prototype traite donc un cap absent comme UNOBSERVABLE et laisse passer l'appel ;
il ne doit jamais transformer une entree runtime indisponible en violation.

## Audit privacy-first

Option interne :
`--audit-jsonl <path>`

La trace conserve uniquement :
- session_id ;
- turn_id ;
- tool_use_id ;
- categorie ;
- cap requis ;
- cap observe ;
- decision ALLOW / DENY.

Le texte de la commande n'est pas stocke dans cet audit.

## Critere ENFORCED

Tokn ne doit produire ENFORCED que si le run prouve au minimum :
1. version/runtime compatible ;
2. configuration de hook et hash connus ;
3. hook approuve/trusted et actif avant le run ;
4. matcher couvrant la surface cible ;
5. audit Tokn montrant les decisions du hook ;
6. absence d'erreur/timeout de hook connue sur cette surface ;
7. toute violation observee est bloquee avant execution.

Sans ces preuves, rester SUPPORTED_UNVERIFIED, SUPPORTED_INSUFFICIENT_INPUT, SUPPORTED_NOT_ACTIVE ou NOT_PROVEN selon le niveau de preuve atteint.

## Strategie d'activation

Les runs normaux P6 ne laissent aucun hook installe. Le probe controle peut poser temporairement ~/.codex/hooks.json, puis FINISH/RECOVER le retire ou restaure l'etat precedent.

Ordre :
1. garder le handler desactive par defaut ;
2. valider le contrat avec des payloads synthetiques ;
3. produire un template non actif ;
4. capturer un vrai callback lors d'une validation controlee ;
5. verifier le comportement en Code Mode ;
6. seulement ensuite autoriser un runner experimental a activer/restaurer le hook.

Cette separation preserve la mesure :
le mecanisme d'enforcement est versionne et auditable,
mais il n'est pas injecte silencieusement dans les sessions normales.

## Real Desktop probe findings - 2026-09-29

Two controlled Desktop/Astra probes were performed. The second probe used explicitly trusted hooks:
`PreToolUse Installed=1 Active=1` and `SessionStart Installed=1 Active=1`.

The Tokn JSONL adapter still wrote zero rows, but Codex local logs prove the runtime hook path was active:
- the real Astra turn reports `feature.hooks=true` ;
- several `hook/started` -> `hook/completed` pairs occur at the nested exec calls ;
- the rollout proves Code Mode requested `max_output_tokens: 6000`.

The exact OpenAI source tag `rust-v0.158.0-alpha.2.1` resolves the policy question independently of the adapter bug:
`ExecCommandHandler::pre_tool_use_payload` maps unified exec to canonical `Bash`, but serializes only
`{ "command": args.cmd }`. It does not copy `max_output_tokens` into `tool_input`.

Therefore:
- hook activation on the Desktop runtime is proven ;
- nested Code Mode mediation is proven by lifecycle logs ;
- cap visibility is disproven for this PreToolUse contract ;
- the Tokn adapter's zero-row bug remains diagnostic debt, not evidence that hooks were inactive ;
- hard cap enforcement would require another supported mediation point or a future runtime contract ;
- Tokn ne recherche pas ce point en priorite : les output caps restent diagnostic-only sauf nouveau besoin produit prouve.
