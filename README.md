# Tokn Observer

[![CI](https://github.com/ELITEAYOTO/Tokn/actions/workflows/ci.yml/badge.svg)](https://github.com/ELITEAYOTO/Tokn/actions/workflows/ci.yml)

Tokn est un moteur local-first d'observabilite et d'analyse pour les coding agents.
Codex/Astra est le premier runtime valide, pas une dependance du domaine Tokn.
L'objectif actuel est d'augmenter le travail utile et la qualite par token sans brider le modele ;
la North Star long terme est la meme ou meilleure qualite pour un cout agent total plus faible.

Version binaire actuelle : **V0.1** (workspace Cargo 0.1.0)
Release : **V0.1 Measurement Hardening**

Etat au 2026-10-03 :
**P0-P9 DONE ; Experiment 002 ACCEPTED ; Measurement Contract V1 FROZEN ; Store foundation DONE ; Local MCP transport prototype ACCEPTED ; Historical Analyzer + Context Ledger IN PROGRESS.**

## Principe

observer -> verifier la preuve -> mesurer -> comparer -> seulement ensuite optimiser

Regles non negociables :
- UNKNOWN reste UNKNOWN ;
- NO_EVIDENCE != PASS ;
- parent seul != run complet si des sous-agents existent ;
- output workspace != input workspace tant que ce n'est pas prouve ;
- aucune economie causale sans experience valide ;
- aucune reduction de contexte n'est un gain si la qualite baisse ;
- le Core reste provider-neutral et les details runtime/provider restent dans des adapters minces ;
- une capability absente reste UNKNOWN/NOT_CAPTURED selon la preuve ;
- les semantiques tokens OpenAI ne deviennent jamais un contrat universel par defaut.

## KPI actuel / North Star long terme

**KPI actuel : travail utile et qualite par token consomme.**

**North Star long terme : meme ou meilleure qualite pour un cout agent total plus faible**
(tokens, temps, retries, duplication, failures), toujours avec qualite comme contrainte.

Tokn ne cherche pas a rendre Astra artificiellement plus court.
La strategie produit est de reduire le travail contextuel inutile :
relectures, recherches dupliquees, contexte duplique entre agents,
rediscovery inter-run, cache instable et evidence outil repetee.

Decision :
docs/decisions/ADR-003-QUALITY-PRESERVING-EFFICIENCY.md

Strategie :
docs/strategy/CONTEXT-EFFICIENCY-STRATEGY.md

Architecture multi-runtime :
docs/design/MULTI-RUNTIME-CORE-AND-DATA-ARCHITECTURE.md

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

Ordre actuel :
- M4 Historical Analyzer + Context Ledger ;
- M4 exact Context/Result Identity + SourceStableId file V0 + observed content evolution + source mutation timing + run-boundary SourceVersionHistory V1 ACCEPTED ; M4.5 freshness/Context Twin + Shadow Retrieval/Edit ensuite ;
- M5 Findings + Opportunity Analyzer ;
- M6 counterfactual/policy lab puis Experiment 003 causal A/B ;
- M7 Advisor ;
- M8 Selective Context Compiler / Project Memory ;
- M9 AutoLab offline-first ;
- M10 learned policies seulement si les methodes simples plafonnent.

Architecture long terme : un Core analytique provider-neutral consomme des preuves normalisees produites par des Runtime Adapters minces. Le prochain runtime ne doit pas provoquer une seconde implementation de Tokn Analysis.

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
1. `STATUS.md` - etat reel ;
2. `ROADMAP.md` - ordre et gates ;
3. `docs/plans/IMPLEMENTATION-PATH.md` - chemin d execution courant ;
4. `docs/plans/NEXT-SESSION-CHECKLIST.md` - prochaine action concrete ;
5. `docs/design/MULTI-RUNTIME-CORE-AND-DATA-ARCHITECTURE.md` - contraintes architecturales a respecter.

Les plans/tests V0.1 restent des references historiques/validation. Ne pas relire tous les documents historiques pour reprendre le travail.

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

## M4 source mutation observation - 2026-10-03

The accepted mutation-timing slice extends ToolActivityHistory to V3 with optional rollout `observed_at` evidence.
- conservative single-target `Set-Content` / `Add-Content` operations can resolve the same project-scoped SourceStableId used by file reads;
- `tokn-observe source-mutation-history` exposes observed mutation-operation timing as `OBSERVED` or `NOT_CAPTURED`;
- `tool_status=completed` is preserved but does not prove that the file changed; `effect_status` remains `NOT_VERIFIED`;
- no raw path, raw command output or durable tool-result content is added by this slice;
- no freshness, staleness, invalidation-effect or safe-reuse claim follows from operation timing alone.

## M4 run-boundary source version evidence - 2026-10-03

SourceVersionHistory V1 records privacy-safe source versions directly observed in Runner workspace snapshots.
- BEFORE/AFTER file versions are derived from workspace snapshot SHA-256 evidence, never from mutation intent;
- the same workspace-relative logical file reuses the project-scoped SourceStableId;
- durable `ver-v1-*` fingerprints are project-scoped and domain-separated; raw relative paths and raw snapshot SHA-256 values are not persisted;
- missing one boundary stays `UNKNOWN`; Tokn does not infer ADDED/REMOVED;
- `tokn-observe source-version-history` exposes the observed boundary records; the analysis reducer can classify `UNCHANGED_OBSERVED`, `CHANGED_OBSERVED` or `UNKNOWN`;
- no Git-commit provenance, mutation causality, freshness/staleness, invalidation effect or safe-reuse claim is implied.

Long-term direction remains observation-first. Multi-runtime support is an architectural target, not a current support claim. Before runtime #2: Runtime Adapter Contract V1, Token Semantics V2, Capability Manifest V1 and sanitized conformance fixtures. See docs/design/MULTI-RUNTIME-CORE-AND-DATA-ARCHITECTURE.md and ADR-006.
