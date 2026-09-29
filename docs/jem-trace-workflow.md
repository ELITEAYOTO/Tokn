# JEM Ultimate — premier run réel tracé avec Tokn

## Objectif

Tokn ne cherche pas à rendre Astra plus rapide.

Le KPI principal est :

**maximiser le travail utile et la qualité obtenue par token consommé.**

Un run plus long est acceptable s'il consomme moins de tokens pour une qualité égale ou meilleure.

Le problème à mesurer : le budget de tokens Astra peut être consommé en environ 7 à 10 minutes pendant un développement intensif.

## Pourquoi JEM Ultimate est un bon test

JEM Ultimate est une charge réelle :
- grand projet ;
- lectures de fichiers ;
- recherches ;
- outils ;
- tests ;
- possibles sous-agents ;
- nombreuses boucles modèle/outils.

Cette baseline est plus utile qu'un micro-benchmark artificiel.

## Démarrage simple

1. Fermer complètement Codex Desktop.
2. Double-cliquer :

`E:\Tokn\V0-CodexTkn-Consume\START-JEM-TRACE.cmd`

3. Codex Desktop démarre en mode diagnostic.
4. Ouvrir JEM Ultimate et lancer le vrai prompt Astra.

## Pendant le run

Utiliser Astra normalement.

Ne simplifie pas artificiellement le prompt pour Tokn : on veut observer le vrai workflow.

Le temps brut n'est pas la cible principale. Laisse Astra travailler autant que nécessaire.

## Après le run

Quand Astra a terminé :

1. fermer complètement Codex Desktop ;
2. double-cliquer :

`E:\Tokn\V0-CodexTkn-Consume\FINISH-JEM-TRACE.cmd`

Tokn cherchera le bundle, l'importera, affichera un rapport puis lancera le reducer officiel vers un cache séparé.

## Ce que Tokn cherchera

- input tokens ;
- cached input ;
- cache-write input ;
- output ;
- reasoning output ;
- inference calls ;
- threads/sous-agents ;
- répétitions ;
- tools ;
- compactions ;
- pertes de cache.

Le but est d'éliminer les tokens qui n'améliorent pas le résultat :
relectures évitables, contexte redondant, sorties outils trop grandes, recherches répétées, sous-agents redondants et pertes de cache.

## Confidentialité

Les Rollout Traces peuvent contenir des données sensibles du projet.

Ne publie pas :
- `E:\Tokn\V0-CodexTkn-Consume\traces`
- `%LOCALAPPDATA%\Tokn\Observer`
