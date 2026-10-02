# Tokn Observer

[![CI](https://github.com/ELITEAYOTO/Tokn/actions/workflows/ci.yml/badge.svg)](https://github.com/ELITEAYOTO/Tokn/actions/workflows/ci.yml)

Tokn est un profiler/analyzer local pour Codex/Astra.
Son objectif est d'augmenter le travail utile et la qualite obtenus par token,
sans brider la capacite du modele.

Version binaire actuelle : **V0.1** (workspace Cargo 0.1.0)
Release : **V0.1 Measurement Hardening**

Etat au 2026-10-01 :
**P0-P9 DONE ; Experiment 002 ACCEPTED ; Measurement Contract V1 FROZEN ; Store foundation DONE ; Local MCP transport prototype ACCEPTED ; Historical Analyzer + Context Ledger IN PROGRESS.**

## Principe

observer -> verifier la preuve -> mesurer -> comparer -> seulement ensuite optimiser

Regles non negociables :
- UNKNOWN reste UNKNOWN ;
- NO_EVIDENCE != PASS ;
- parent seul != run complet si des sous-agents existent ;
- output workspace != input workspace tant que ce n'est pas prouve ;
- aucune economie causale sans experience valide ;
- aucune reduction de contexte n'est un gain si la qualite baisse.

## KPI

**Travail utile et qualite par token consomme.**

Tokn ne cherche pas a rendre Astra artificiellement plus court.
La strategie produit est de reduire le travail contextuel inutile :
relectures, recherches dupliquees, contexte duplique entre agents,
rediscovery inter-run, cache instable et evidence outil repetee.

Decision :
docs/decisions/ADR-003-QUALITY-PRESERVING-EFFICIENCY.md

Strategie :
docs/strategy/CONTEXT-EFFICIENCY-STRATEGY.md

## Etat V0.1

DONE :
- P0 fixtures sanitisees ;
- P1 source health + fallback ;
- P2 RunGroup + agent graph ;
- P3 terminal status ;
- P4 session tool adapter + policy observation ;
- P5 dynamic workspace tracking ;
- P6 policy hint/observation/enforcement + limite hook documentee ;
- P7 experiment validity reducer + structured causal claim gate ;
- P8 runner V0.1 self-contained ;
- P9 golden replay + release validation.

NEXT :
- Historical Analyzer + Context Ledger observation-only.

Experiment 002 a valide l'instrumentation sur un run reel :
pipeline COMPLETE, terminal COMPLETED, quality PASS, verdict INSTRUMENTATION_ONLY.
Le run ne prouve aucune economie et n'autorise aucune conclusion causale.

## P6 - conclusion

Le runtime Codex Desktop 0.158.0-alpha.2.1 mediatise les nested exec via PreToolUse/Bash,
mais ne transmet pas `max_output_tokens` au callback.

Etat : `SUPPORTED_INSUFFICIENT_INPUT`.

Cette limite est documentee.
Le hard output-cap enforcement n'est plus un objectif produit V0.1.
Les caps restent un signal diagnostique historique.

La sonde JSONL live a zero ligne reste une dette diagnostique non bloquante.

## Experiment 001 - golden facts

- 4 sessions : 1 parent + 3 sous-agents ;
- 80 usage records ;
- input 5,285,737 ;
- cached 5,059,712 ;
- uncached 226,025 ;
- output 26,021 ;
- reasoning 3,018 ;
- logical total 5,311,758 ;
- descendants ~52.9 % ;
- terminal INCOMPLETE_USAGE_LIMIT ;
- diagnostic PARTIAL ;
- policy observation FAIL : 58 cibles / 17 violations ;
- workspace B07-C selectionne ;
- diff 13 ajoutes / 4 modifies / 0 supprimes ;
- verify:local PASS ;
- verdict INSTRUMENTATION_ONLY.

Le ratio cached/input est ~95.72 %.
Il ne prouve ni que le contexte est utile, ni que le cache est optimal.

## Direction apres V0.1

Historical Analyzer + Context Ledger :
- historique runs/projets ;
- attribution parent/sous-agents ;
- cached/uncached par tour/agent ;
- repeated reads/searches ;
- duplication de contexte ;
- tool evidence footprint ;
- cache/context-management observations ;
- candidats Project Memory avec provenance.

Puis Finding Engine -> Experiment Lab -> seulement ensuite optimisation active.

## Build

Depuis la racine du repository.

Validation :
- `scripts\test.ps1`
- `scripts\build-release.ps1`
- `scripts\package.ps1`

Binaire :
`target\release\tokn-observe.exe`

## Documentation

Point d'entree unique :
`docs/INDEX.md`

Pour reprendre le developpement :
1. `STATUS.md`
2. `ROADMAP.md`
3. `docs/plans/V0.1-IMPLEMENTATION-PLAN.md`
4. `docs/plans/NEXT-SESSION-CHECKLIST.md`

Ne pas relire tous les documents historiques pour reprendre le travail.

## Licence

Tokn est distribue sous double licence **MIT OR Apache-2.0**.
Voir `LICENSE-MIT` et `LICENSE-APACHE`.

## M4 activity history - 2026-10-02

The current accepted M4 slice adds a privacy-safe ToolActivityHistory and ActivityTimeline foundation:
- raw command/workdir/error text is not persisted;
- activity is ordered inside each agent only; cross-agent order remains UNKNOWN unless evidenced;
- exact repeated-operation fingerprints are observations, not waste/savings claims;
- `tokn-observe activity-timeline` exposes the offline timeline;
- legacy report uncached accounting is aligned with frozen V1 semantics: input - cached, while cache-write remains separate.

Long-term direction remains observation-first. Claude/Cowork and a Context Compiler are planned future runtimes/layers, not current support claims.
