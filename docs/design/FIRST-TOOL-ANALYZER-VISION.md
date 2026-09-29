# Tokn First Tool - Analyzer Vision

Derniere mise a jour : 2026-09-29

## Intention

Le premier outil Tokn ne doit pas rester un simple compteur de tokens.

Sa cible long terme est un profiler/analyzer local du travail d'Astra/Codex :
observer un run, reconstruire ce qui s'est passe, expliquer ou sont partis les tokens,
conserver un historique utile et detecter des opportunites d'optimisation mesurables.

Le premier outil reste d'abord passif :
observer -> reconstruire -> attribuer -> comparer -> expliquer -> recommander.

L'application automatique des optimisations appartient a une phase ulterieure,
afin de ne pas melanger mesure et intervention.
## Questions auxquelles l'Analyzer doit repondre

Pour chaque run :
- quel projet et quel workspace Astra a reellement utilises ?
- quels autres workspaces ont ete crees, copies ou abandonnes ?
- combien de threads et sous-agents ont travaille ?
- combien de tokens chaque agent a consommes ?
- quelles phases ont consomme le plus ?
- quels outils ont produit le plus de contexte modele-visible ?
- quels fichiers ont ete relus plusieurs fois ?
- quelles recherches ont ete repetees ?
- quels appels ont echoue, ete annules ou retries ?
- quel travail local a ete produit avant l'epuisement du quota ?
- la tache a-t-elle ete terminee ou interrompue ?
- la qualite finale est-elle validee ?
## Historique de discussions et sessions

L'Analyzer doit pouvoir reconstruire un historique technique utile des conversations
a partir des rollouts disponibles, sans devenir une base de donnees brute de conversations.

A conserver par defaut :
- identite de session/thread ;
- timestamps et ordre des tours ;
- type de message/evenement ;
- outils utilises ;
- references aux fichiers/projets ;
- tailles et couts tokens ;
- terminal status ;
- liens parent/subagent ;
- resumes structuraux optionnels.

Le texte brut complet des prompts/reponses ne doit pas etre stocke durablement par defaut.
Un mode explicite pourra conserver davantage de contenu si l'utilisateur le demande.
## Detection automatique des projets

L'Analyzer doit construire une notion de ProjectIdentity independante d'un chemin unique.

Signaux possibles :
- workspace root initial ;
- manifests : package.json, Cargo.toml, etc. ;
- Git root, remote et commit quand disponibles ;
- fichiers effectivement lus/ecrits ;
- workdir des outils ;
- lineage de copie de workspace ;
- nom/version de package ;
- hash de marqueurs stables.

But : reconnaitre qu'un nouveau dossier est une nouvelle version du meme projet,
sans supposer que le projet reste toujours au meme chemin.
## Attribution des tokens

L'Analyzer doit separer au minimum :
- parent vs sous-agents ;
- input / cached / uncached / output / reasoning ;
- outils et sorties outils ;
- lectures/search ;
- build/test ;
- modifications ;
- retries/erreurs ;
- contexte partage ou duplique ;
- cache hits/misses quand observables.

L'attribution ne doit pas inventer de causalite.
Chaque resultat doit porter un niveau de preuve : OBSERVED / DERIVED / INFERRED / UNKNOWN.
## Timeline et phases de travail

Un run doit pouvoir etre segmente en phases heuristiques :
- orientation / comprehension ;
- exploration repo ;
- planification ;
- implementation ;
- verification/tests ;
- debugging ;
- rework ;
- finalisation.

Ces phases servent a expliquer le cout, pas a juger le modele.

Exemples :
- combien de tokens avant la premiere modification utile ?
- combien apres le premier test rouge ?
- combien de contexte depense en relecture ?
- combien de travail a ete produit par les sous-agents ?
## Historique multi-run

L'Analyzer doit garder un historique local par projet :
- runs ;
- runtime/model/config ;
- tache ;
- project version/commit ;
- workspace lineage ;
- cout token ;
- terminal status ;
- qualite ;
- fichiers modifies ;
- agents ;
- outils ;
- findings.

Cela permettra comparaison run contre run, tendances, regressions,
detection de comportements repetitifs et validation d'une optimisation sur plusieurs runs.
## Opportunity / Finding Engine

Le premier outil doit pouvoir detecter des opportunites sans les appliquer automatiquement.

Exemples :
- fichier relu N fois par plusieurs agents ;
- meme recherche repetee ;
- sortie outil enorme suivie de tres peu de travail utile ;
- sous-agent qui recharge tout le contexte parent ;
- retry identique ;
- build/test lance trop souvent sans changement pertinent ;
- workspace copie plusieurs fois inutilement ;
- prompts/outils dont la structure casse le cache ;
- contexte non utilise ou reintroduit plusieurs fois.

Chaque finding doit avoir preuve, confiance, cout observe, frequence historique,
impact potentiel borne, risque qualite et proposition d'experience A/B.
## Ce que l'Analyzer ne doit pas faire au debut

Ne pas :
- annoncer une economie theorique comme un gain reel ;
- remplacer UNKNOWN par zero ;
- juger un agent uniquement parce qu'il est couteux ;
- reduire automatiquement le nombre de sous-agents ;
- modifier les prompts/runtime sans experience controlee ;
- stocker toutes les conversations brutes par defaut ;
- fusionner des projets uniquement a partir du nom de dossier.

Une recommandation devient candidate a optimisation seulement apres preuve reproductible.
## Architecture long terme du premier outil

Evidence adapters
-> RunGroup / ProjectIdentity / WorkspaceLineage
-> normalized event store
-> token ledger
-> timeline/phases
-> tool + file activity graph
-> history store
-> attribution engine
-> behavior detectors
-> finding engine
-> reports / CLI / future GUI

Le Context Compiler, la Project Memory active et l'Optimizer restent des couches separees.

## Ordre de construction

1. V0.1 : mesure fiable et preuve.
2. Historical Analyzer : projets, runs, discussions techniques, timelines.
3. Attribution avancee : fichiers/outils/phases/agents.
4. Finding Engine : opportunites avec preuve.
5. Experiment Lab : valider les optimisations.
6. Seulement ensuite : intervention active.