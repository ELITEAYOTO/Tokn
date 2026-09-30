# Token Accounting

Derniere mise a jour : 2026-09-29

## Invariants de base

- cached input <= input ;
- reasoning output <= output ;
- cached input est inclus dans input ;
- reasoning output est inclus dans output ;
- logical total = input + output ;
- une donnee absente reste UNKNOWN ;
- une observation invalide n'est jamais reparee silencieusement.

Ordinary uncached :
si input et cached existent,
ordinary_uncached = input - cached.

Si cache-write existe, sa semantique est conservee separement
et n'est jamais additionnee comme un nouveau cout logique.

cached/input est un indicateur de reutilisation reportee, pas un score de qualite.
Tokn distingue :
- tokens logiques ;
- cached vs uncached ;
- cout API quand un tarif applicable est explicitement choisi ;
- quota produit Codex/ChatGPT, qui ne doit pas etre deduit du pricing API.

## Deduplication

Session rollout :
utiliser une identite de reponse fiable quand disponible.

Diagnostic trace :
priorite inference_call_id,
puis provider + response_id,
puis response_id.

Meme identite + meme usage :
Duplicate, non recompte.

Meme identite + usage different :
Conflict, non recompte silencieusement.

Sans identite fiable :
pas de fusion agressive.

## V0.1 - RunGroup accounting

Un run Astra peut contenir plusieurs threads.

Tokn V0.1 agrege :
- parent ;
- sous-agents directs ;
- descendants futurs.

Chaque thread est compte une seule fois.

RunGroup logical total =
somme des logical totals de tous les membres uniques.

Le parent n'est pas utilise comme proxy du run complet.

Experiment 001 golden facts :
- parent logical : 2,499,523 ;
- descendants logical : 2,812,235 ;
- total : 5,311,758.

## Source health

Une source PARTIAL ne peut pas fournir un zero autoritaire
pour une metrique qu'elle n'observe pas.

Exemple :
diagnostic trace usage_count=0
et session rollout usage_count>0

=> le zero diagnostic est absence de couverture,
pas preuve de zero consommation.

## Terminal status

La comptabilite d'un run doit toujours etre accompagnee de son statut terminal.

Un run INCOMPLETE_USAGE_LIMIT peut etre mesure,
mais il ne doit pas etre compare comme un succes fonctionnel complet.

## Cross-source reconciliation

Pour une meme famille de metriques :
- conserver chaque observation et sa provenance ;
- comparer les sources ;
- signaler accord/conflit ;
- selectionner une autorite selon regles documentees.

Ne jamais additionner aveuglement les memes appels observes par deux sources.

## Idempotence

Le hash du snapshot/source reste deterministe.
Reimporter exactement la meme preuve doit produire le meme run/source logique.

## Reference

Voir :
`V0.1-MEASUREMENT-ARCHITECTURE.md`
`EVIDENCE-AND-COVERAGE.md`
`AGENT-COST-ATTRIBUTION.md`
