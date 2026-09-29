# Codex Hook Enforcement - Tokn P6

Derniere mise a jour : 2026-09-29

## Decision

Codex 0.158.0-alpha.2.1 expose une feature `hooks` stable.
`PreToolUse` avec le matcher `Bash` est un point de mediation supporte pour les commandes shell.
Les appels d'outils imbriques via Code Mode passent egalement par cette decision.

Cela prouve qu'une interception runtime supportee existe sans patcher Codex.
Cela ne prouve pas encore que la policy Tokn sur `max_output_tokens` peut etre enforcee :
la documentation publique garantit `tool_input.command`, mais ne garantit pas actuellement
que `max_output_tokens` soit present dans le payload PreToolUse.

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
- SUPPORTED_NOT_ACTIVE : le contrat policy/runtime a ete valide, mais Tokn ne l'a pas active sur ce run.
- NOT_PROVEN : un mecanisme semble actif mais la preuve du run est insuffisante.
- ENFORCED : preuve run-scoped suffisante que le mecanisme a mediatise la surface cible.

Experiment 001 est SUPPORTED_UNVERIFIED :
- meme runtime local : codex-cli 0.158.0-alpha.2.1 ;
- hooks disponibles ;
- aucun hook Tokn n'etait configure pour ce run ;
- la visibilite de `max_output_tokens` dans le vrai callback n'a pas ete capturee ;
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
- refuse un cap absent ou superieur a la policy ;
- ignore build/test, write, git et autres familles non ciblees.

Important : `tool_input.max_output_tokens` est une hypothese de prototype,
pas encore un champ garanti par le contrat public PreToolUse.

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

Sans ces preuves, rester SUPPORTED_UNVERIFIED, SUPPORTED_NOT_ACTIVE ou NOT_PROVEN selon le niveau de preuve atteint.

## Strategie d'activation

P6 ne modifie pas automatiquement ~/.codex/hooks.json.

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
