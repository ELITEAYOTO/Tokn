# Experiment 001 - Runtime output caps

Status: **HISTORICAL EXPERIMENT DESIGN - NOT CURRENT PRODUCT STRATEGY**
Superseded for product direction by `../decisions/ADR-003-QUALITY-PRESERVING-EFFICIENCY.md`.
The measurements remain valid historical evidence.

Derniere mise a jour : 2026-09-27 17:49 +02:00

## But

Tester si des sorties runtime plus bornees peuvent reduire la consommation
de contexte sans reduire la qualite du travail Astra.

Le temps de reponse n'est pas le KPI principal.

Une experience n'est consideree utile que si :
- les tokens baissent ;
- la qualite reste egale ou meilleure ;
- le travail utile reste egal ou superieur.

## Baseline JEM

Bundle : run JEM Ultimate du 2026-09-26.

Mesures runtime observables :
- tools : 26 ;
- tools avec raw token count : 20/26 ;
- raw tool tokens : 64 219 ;
- tools avec output cap : 20/26 ;
- cap-adjusted upper bound actuel : 47 191 ;
- raw output au-dessus du cap : 2.
## Pourquoi cibler file_read et search

Sur les tools runtime mesures :

file_read :
- 8 appels ;
- raw : 46 572 ;
- cap-adjusted upper : 31 520.

search :
- 5 appels ;
- raw : 16 714 ;
- cap-adjusted upper : 14 738.

Les mutations et tests sont faibles en comparaison.
Les 6 actions agent directes ne disposent pas du meme compteur token ;
leur cout reste UNKNOWN dans cette analyse.

## Simulations offline

Scenario conservateur :
- file_read=5000 ;
- search=3000 ;
- upper bound : 38 766 ;
- delta : -8 425 ;
- variation : -17,85 %.

Scenario modere :
- file_read=3000 ;
- search=2000 ;
- upper bound : 29 561 ;
- delta : -17 630 ;
- variation : -37,36 %.
Scenario agressif :
- file_read=2000 ;
- search=1500 ;
- upper bound : 22 166 ;
- delta : -25 025 ;
- variation : -53,03 %.

Ces chiffres sont des WHAT-IF.
Ils ne prouvent pas que les memes reductions seront observees
dans un vrai run et ne disent rien sur la qualite.

## Premier candidat A/B

Pour la premiere experience reelle, utiliser le scenario conservateur.

Raison :
- reduction theorique deja mesurable ;
- changement moins agressif ;
- risque plus faible de couper une information utile.

Cette selection est un point de depart experimental,
pas un verdict sur l'optimum final.

## Conditions d'acceptation

Comparer avec exactement la meme classe de tache autant que possible.

Mesurer :
- input / cached / uncached ;
- output / reasoning ;
- inference calls ;
- tool calls ;
- raw/cap-adjusted tool output ;
- duree observee a titre informatif ;
- tests/build ;
- resultat fonctionnel ;
- regressions ;
- evaluation humaine du travail produit.
## Regle de decision

Ne jamais conclure "meilleur" uniquement parce que les tokens baissent.

Si les tokens baissent mais que :
- Astra manque une information importante ;
- il boucle davantage ;
- il produit plus d'erreurs ;
- il realise moins de travail ;
- il doit relire plusieurs fois la meme chose ;

alors l'experience est consideree non validee.

## Commande de simulation

Exemple conservateur :

tokn-observe simulate-caps <bundle> --cap file_read=5000 --cap search=3000

La commande est offline et ne modifie pas Codex/Astra.

## Runtime comparability note

The older baseline and the candidate also use different Codex Desktop builds.

Baseline environment observed:
- Codex Desktop: 26.924.1866.0
- embedded codex: codex-cli 0.158.0-alpha.2

Current candidate environment before the run:
- Codex Desktop: 26.924.2738.0
- embedded codex: codex-cli 0.158.0-alpha.2.1

The candidate launcher records the exact versions again at start time.

Therefore old-baseline vs candidate deltas must not be interpreted as the
effect of Tokn alone. A later same-task / frozen-workspace / same-runtime A/B
is required for causal attribution.
