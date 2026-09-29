# Baseline JEM Ultimate - 2026-09-26

Status: **HISTORICAL BASELINE / GOLDEN REFERENCE**
Output-cap sections below describe the original investigation, not the current optimization strategy.

## But

Premiere vraie baseline Tokn sur un run Astra de developpement reel dans JEM Ultimate.

Objectif Tokn :

**maximiser le travail utile et la qualite obtenue par token consomme**, pas rendre Astra plus rapide.

Le run observe a consomme le budget Astra en environ 10 minutes d'apres l'observation utilisateur.

## Source

Bundle diagnostic :

`E:\Tokn\V0-CodexTkn-Consume\traces\20260926-154229-jem-ultimate\trace-5511a673-ce04-4cb4-9a81-0ae0f48d528b-01a0869c-780e-7592-afd8-109363f2e508`

Run Tokn :

`run-5dd0a3ef5a5ba318cbf7a637`

## Validation de la mesure

Le reducer officiel Codex et Tokn donnent exactement les memes totaux observes :

- input tokens : **3 316 765**
- cached input : **3 168 896**
- cache write input : **0**
- uncached input : **147 869**
- output tokens : **16 062**
- reasoning output : **3 125**
- logical total : **3 332 827**
- cached share : **95,54 %**
- usage records : **25**

Tokn lit maintenant les tokens directement depuis les payloads references par le bundle brut.
Le reducer officiel reste un oracle de comparaison, pas une dependance comptable obligatoire.

## Activite modele

Trace reduite :

- inference calls demarres : **30**
- completed : **25**
- cancelled : **4**
- failed : **1**
- modele : **gpt-6-astra**
- Codex turns : **1**
- compactions : **0**

Les appels annules et echoues n'exposent pas de `usage` final dans le state reduit.
Les 25 appels completes portent donc toute la consommation token observee ci-dessus.

Span observe entre premier demarrage et derniere fin d'inference :
**614,46 s**, soit environ **10 min 14 s**.

Volume moyen d'input observe :
environ **323 871 input tokens/minute** sur ce span.

## Taille des appels

Sur les 25 appels completes :

- moyenne input : **132 670,6**
- mediane input : **137 654**
- minimum input : **93 768**
- maximum input : **160 725**
- appels >= 140k input : **11 / 25**

Premiers cinq inputs :

`93 768, 94 938, 94 993, 105 207, 110 251`

Derniers cinq inputs :

`158 465, 159 911, 160 493, 160 610, 160 725`

Constat descriptif :
le contexte d'entree grossit fortement au fil du run.

## Uncached input

Sur les 25 appels completes :

- moyenne uncached : **5 914,8**
- mediane uncached : **2 601**
- minimum uncached : **273**
- maximum uncached : **71 624**

Le premier appel represente a lui seul environ **71,6k tokens non caches**.

Le cache fonctionne donc tres bien sur ce run, mais il n'empeche pas un volume cumule d'input enorme parce que les appels sont nombreux et tres gros.

## Outils

- tool calls : **26**
- exec_command : **19**
- assign_agent_task : **3**
- send_message : **2**
- list_agents : **1**
- write_stdin : **1**
- code cells : **19**
- terminal operations : **20**

## Premiers enseignements

Ce run ne montre pas un echec du prompt cache : le cache hit observe est de 95,54 %.

Les premieres pistes a mesurer avant toute optimisation active sont plutot :

1. croissance continue du contexte au cours d'un seul Codex turn ;
2. grand nombre d'inference calls sur une periode courte ;
3. reexposition repetee d'un contexte de 100k a 160k tokens ;
4. sorties outils ou lectures susceptibles d'alimenter cette croissance ;
5. sous-agents et boucles outils a evaluer en termes de travail utile obtenu.

Aucune de ces pistes ne doit encore etre transformee en optimisation sans test A/B.

## Regle pour la suite

Une optimisation Tokn sera acceptee uniquement si elle montre :

- moins de tokens consommes ;
- qualite egale ou meilleure ;
- travail utile egal ou superieur ;
- pas de degradation cachee du raisonnement ;
- temps de reponse pouvant etre plus long si le rendement token est meilleur.

## Attribution Engine V0

Le premier rapport d'attribution automatique sur ce run donne :
- tool original tokens : **64 219** ;
- exec_command : **63 869** tokens exposes ;
- tools non attribues a une inference complete : **20 402** tokens ;
- 26 tool calls au total.

Exemple marquant :
un exec_command de **10 239 tokens** est suivi d'une inference dont
l'input logique augmente de **10 214 tokens** et l'uncached input vaut **10 359**.

Cette relation est une correlation temporelle, pas encore une causalite prouvee.

## Correction d'interpretation

La croissance de input_tokens ne signifie pas que Codex renvoie physiquement
160k tokens complets sur le fil a chaque appel.

Exemple observe :
certains appels proches de 160k input utilisent un request payload local
de seulement quelques Ko et parfois un seul item.

La bonne distinction est :
1. contexte logique vu/comptabilise par le modele ;
2. cached input ;
3. uncached input ;
4. payload local reel ;
5. sorties outils ajoutees.

Cette distinction devient une contrainte de conception Tokn.

## Raw output vs output caps

Analyse completee le 2026-09-27.

Sur les 20 tools runtime avec mesure :
- raw tokens : 64 219 ;
- cap-adjusted upper : 47 191 ;
- 2 sorties brutes depassent leur cap.

file_read :
- raw 46 572 ;
- cap upper 31 520.

search :
- raw 16 714 ;
- cap upper 14 738.

Les 6 actions agent directes n'exposent pas le meme compteur ;
leur cout reste UNKNOWN dans cette comparaison.
