# Tokn Roadmap

Derniere mise a jour : **2026-10-03**

## North Star

Court terme : maximiser le travail utile et la qualite obtenus par token, sans reduire arbitrairement la capacite du modele.

North Star long terme : **meme ou meilleure qualite pour un cout agent total plus faible**
(tokens, temps, retries, duplication, failures), avec quality gate avant efficiency.

Ordre obligatoire :
**mesurer -> prouver -> expliquer -> experimenter -> optimiser -> automatiser.**

## M0 - Observer V0.0

Statut : DONE

Acquis :
Codex discovery, JSONL, token accounting, SQLite, diagnostic trace,
attribution V0, simulations offline et premier runner experimental.

## M1 - Experiment 001

Statut : DONE / POST-MORTEM

Valeur :
decouverte des failles de mesure.

Pas de conclusion d'optimisation autorisee.

Golden facts :
voir `docs/experiments/001-POSTMORTEM.md`.

## M2 - V0.1 Measurement Hardening

Statut : DONE

P0 sanitized fixtures : DONE
P1 source health/fallback : DONE
P2 RunGroup/agent graph : DONE
P3 terminal status : DONE
P4 session tool adapter : DONE
P5 dynamic workspace : DONE
P6 policy evidence/runtime capability : DONE
P7 experiment validity reducer : DONE
P8 runner V0.1 : DONE
P9 golden replay/release validation : DONE

### P6 closure

P6 a etabli :
- hint != observation != enforcement ;
- soft policy n'est jamais presentee comme enforced ;
- hooks Desktop/Code Mode sont reellement mediatisees ;
- PreToolUse Bash 0.158 ne recoit pas max_output_tokens ;
- etat correct : SUPPORTED_INSUFFICIENT_INPUT.

Decision produit :
ne pas poursuivre les hard output caps comme axe principal.
La dette JSONL live du probe est non bloquante et reste au backlog.

Plan d'execution detaille depuis P8 jusqu'aux couches post-V0.1 :
`docs/plans/IMPLEMENTATION-PATH.md`.

## M3 - Experiment 002

Statut : DONE / ACCEPTED

But :
valider V0.1 end-to-end sur une petite tache reelle.

Resultat :
- Runner COMPLETE ;
- terminal COMPLETED ;
- quality PASS, 863/863 tests ;
- 2 agents / 49 usage records ;
- workspace correct et diff 1/2/0 ;
- modele `gpt-6.1-sol` observe ;
- configuration_recorded=UNKNOWN ;
- validity INSTRUMENTATION_ONLY ;
- aucune conclusion causale autorisee.

Le FINISH reel a revele un probleme de layout de trace CLI et un probleme
`$LASTEXITCODE` PowerShell. Les preuves ont ete recuperees sans relancer le modele,
puis le fallback session a ete versionne et regression-teste.

## M3.5 - Measurement Contract Freeze

Statut : DONE / FROZEN V1

Resultat :
- `measurement-contract.json` emis par Runner ;
- versions explicites des schemas/evidence/semantiques ;
- Runner refuse les versions workspace/snapshot inconnues ;
- token accounting V1 fige (`uncached = input - cached`, cache-write separe) ;
- ModelRuntimeProfile V1 fige au niveau domaine ;
- replay Experiment 001 verifie toutes les versions sans changer les golden facts.

## M3.6 - Store + ModelRuntimeProfile persistence

Statut : DONE / FOUNDATION

Resultat :
- Store schema V2 versionne et fail-closed ;
- projects/workspaces pseudonymises ;
- runs + agents + usage summaries + known counters ;
- provenance par fingerprint sans chemin brut ;
- ModelRuntimeProfile V1 persiste ;
- commande `store-evidence` idempotente ;
- migration legacy efface physiquement les anciens chemins apres VACUUM ;
- replay Experiment 001 valide Runner -> Store deux fois sans multiplier le run.

L ingestion de l historique rate-limit est maintenant implementee dans M4. Findings et Experiment Lab restent differes jusqu'a un consommateur concret.

## M3.7 - Local MCP integration prototype

Statut : DONE / TRANSPORT PROTOTYPE ACCEPTED

Resultat :
- executable separe `tokn-mcp` ;
- transport stdio process-bound, read-only ;
- `tokn_status` + `tokn_recent_runs` ;
- tests Rust + smoke process release PASS ;
- Codex 0.161.0-alpha.2 accepte l'enregistrement stdio ;
- Codex app-server lance Tokn et decouvre les deux outils, toolsError=null ;
- aucun daemon permanent ;
- aucune logique analytique dupliquee ;
- thread ephemere local cree idle avec 0 turns ;
- `mcpServer/tool/call` appelle `tokn_status` avec succes via Codex ;
- aucune auth utilisateur copiee ou inspectee ;
- aucun turn modele ni quota volontairement consomme pour la validation.

## M4 - Historical Analyzer + Context Ledger

Statut : IN PROGRESS

But :
construire l'historique fiable par projet/run/agent/phase.

Valide dans le slice courant :
- ProjectIdentity et WorkspaceLineage ;
- historique multi-run par project/workspace/run/agent ;
- cached/uncached/cache-write/input/output/reasoning par run/agent avec coverage explicite ;
- terminal + quality + validity + runtime profile + provenance ;
- integrity issues sans reparation silencieuse ;
- CLI `tokn-observe context-ledger` ;
- MCP read-only `tokn_context_ledger`, valide standalone et via Codex 0.161.0-alpha.2 ;
- per-turn reste `NOT_CAPTURED` dans Measurement Contract V1 ;
- current retained-context occupancy reste `UNKNOWN`.

Reste M4 :
- granularite par tour seulement si un futur contrat la capture directement ;
- rate-limit snapshots over time : ACCEPTED FOUNDATION ;
- timeline de phases ;
- file/tool activity graph : ACCEPTED FOUNDATION ;
- repeated reads/searches/retries : exact-repeat foundation ACCEPTED ;
- automatic ModelRuntimeProfile compatibility reducer : ACCEPTED FOUNDATION ;
- parent/subagent exact operation overlap : ACCEPTED FOUNDATION ;
- compaction events quand observables ;
- exact result/content identity foundation : ACCEPTED when directly observable ;
- duplicate evidence may be reported only when complete observed result identity agrees ;
- SourceStableId file-read V0 + observed content evolution history : ACCEPTED FOUNDATION ;
- source mutation-operation timing + SourceMutationHistory V1 : ACCEPTED FOUNDATION, effect remains `NOT_VERIFIED` ;
- run-boundary source version/hash from Runner BEFORE/AFTER snapshots + SourceVersionHistory V1 : ACCEPTED FOUNDATION ;
- Workspace Git Provenance V0 (boundary HEAD fingerprint + dirty + explicit coverage) : ACCEPTED FOUNDATION ;
- verified mutation effect/freshness, broader symbol/range identity, compaction/rediscovery and explicit cross-run comparison remain.

## M4.5 - Context Identity + Shadow Foundations

Statut : FOUNDATION STARTED / OBSERVATION-ONLY.

Purpose: bridge M4 evidence to later optimization without changing Astra behavior.

Context Identity / Context Twin V0 seed:
- privacy-safe evidence/context IDs for directly observed items;
- project-scoped keyed content/result fingerprints when content is transiently available;
- SourceStableId file V0 accepted for conservative directly observed reads; broader source kinds/range/symbol identity only when provable ;
- ContentFingerprint privacy-safe pour la version/resultat observe, separe de l'identite stable ;
- run-boundary source version/hash + workspace Git HEAD/dirty provenance are accepted when directly observed in Runner snapshots; range/symbol identity remains evidence-gated;
- mutation-operation timing is accepted evidence; source-specific causality, freshness and verified invalidation state remain explicit/unknown until directly provable;
- agent/run distribution only when directly observed;
- `UNKNOWN` / `NOT_CAPTURED` when delivery, freshness or result identity cannot be proven.

This foundation must not require durable raw tool-output storage.
It is the preferred basis for duplicate-evidence proof, rediscovery analysis and later memory.

Shadow foundations:
- local repo/symbol index, content hashes and Git incremental invalidation;
- Shadow Context Retrieval Engine: lexical/BM25 first, then AST/LSP/graph;
- embeddings only if measured recall/value justifies them;
- Shadow Edit Strategy Analyzer without changing Astra behavior;
- Context Package candidates with provenance, freshness and expand/fallback paths.

Guardrails:
- Astra keeps semantic decisions; Tokn handles deterministic, verifiable mechanics;
- no claim that Astra still remembers a context merely because it was supplied earlier;
- every active future Context Compiler path keeps expand/fallback;
- no hard context cap, reasoning reduction or arbitrary output cap as default optimization.

## M5 - Findings + Opportunity Analyzer V0

Statut : PLANNED / OBSERVATION-ONLY.

Findings cibles:
- uncached growth anormal;
- cache churn quand observable;
- repeated file/search evidence;
- duplicate evidence only when result/context identity is proven;
- project facts rediscovered entre runs;
- compaction/context-management suivie de rediscovery;
- oversized/repeated tool evidence.

Each finding must preserve evidence, provenance, confidence, compatible runtime scope and quality risk.

Opportunity Analyzer V0 then estimates separately:
- addressable surface;
- theoretical upper bound;
- realistic bounded potential;
- frequency;
- evidence confidence;
- preservation/quality risk;
- implementation complexity;
- experiment priority.

`DEPRIORITIZE` is a valid output. A technically elegant optimization is not automatically worth building.

## M6 - Counterfactual / Policy / Experiment Lab

Statut : PLANNED.

Before expensive causal A/B, add a small deterministic offline layer:
- native Astra baseline always retained;
- counterfactual replay for eligible bounded interventions;
- PolicyCandidate / PolicyGenome V0 with schema, scope, runtime constraints, provenance and evidence references;
- Policy Registry lifecycle at least `DISCOVERED -> OFFLINE_PROMISING -> SHADOW -> EXPERIMENTAL`;
- offline estimates explicitly separated from causal results.

Experiment 003 remains the first causal optimization A/B.

Conditions:
- same task and frozen starting workspace;
- compatible runtime/model/config;
- one primary intervention;
- quality gate PASS before efficiency comparison;
- terminal status compatible;
- validity = VALID_FOR_CAUSAL_AB;
- rollback/fallback available for any active experimental path.

Quality is a constraint. Efficiency is the objective inside the acceptable-quality region.

## M7 - Advisor

Statut : FUTURE.

Turns validated recurring findings/opportunities into recommendations.
No automatic mutation.
Every recommendation explains why, evidence, confidence, runtime scope, bounded impact and risk.

## M8 - Selective Context Compiler / Project Memory

Statut : FUTURE / ACTIVE LAYER.

Only for mechanisms with repeated causal wins:
- structured context packages;
- project memory with provenance/freshness/invalidation;
- selective evidence reuse/compression with expand/fallback;
- cache-aware construction when the surface permits it;
- deterministic Tool Compiler operations only where semantics remain Astra-owned.

## M9 - AutoLab V0 / Data and Replay Foundation

Statut : FUTURE / OFFLINE-FIRST.

Do not start before M4-M6 evidence and experiment contracts are stable.

Foundation:
- Experience Bank built from structured/rebuildable Store evidence where possible;
- Feature Store separating observed features from derived labels;
- DatasetManifest with schema/runtime/task/filter/privacy metadata;
- train/validation/holdout discipline, including time/project/runtime holdouts where applicable;
- deterministic replay and Offline Fidelity measurement;
- Policy Registry + Drift Detector;
- bounded raw retention, purge/export/migration policy;
- native baseline remains an eligible policy.

No ML model is required for M9.

## M10 - Adaptive Policy Learning

Statut : FUTURE / RESEARCH.

Only if simpler deterministic/statistical approaches plateau and enough data exists:
1. contextual bandit;
2. Bayesian optimization;
3. learning-to-rank;
4. calibrated surrogate model;
5. evolutionary search OFFLINE ONLY;
6. optional specialized LLM analyst only if structured methods are insufficient.

No learned policy self-deploys. Shadow, causal A/B, quality gate, runtime compatibility, rollback and monitoring remain mandatory promotion gates.

## Cross-cutting benchmark-readiness gate

Status: BASELINE PROTOCOL PREPARED / NON-CAUSAL.

This gate does not renumber M4-M10. It may run in parallel with observation-only M4 work.

Before Experiment 003:
- collect a native baseline corpus with frozen repo/task manifests;
- characterize run-to-run variance before fixing sample size;
- preserve token semantics separately (cached, ordinary uncached, cache-write, output, reasoning);
- predeclare quality and exclusion rules;
- measure Tokn CPU/RAM/disk/latency overhead when injected/default-active;
- randomize/interleave future A/B order when provider cache cannot be reset;
- publish only sanitized manifests/metrics/aggregates, never raw private runs.

`docs/benchmarks/BASELINE-PROTOCOL-V1.md` is the canonical protocol.
`benchmarks/manifest.schema.json` is BenchmarkManifest V1.
A passive observer baseline is descriptive only; Experiment 003 remains the first causal optimization A/B.

## Cross-cutting multi-runtime gates

These gates do not renumber M9/M10.

Now / M4-M4.5:
- all new evidence/context types use runtime-neutral naming where possible;
- provider-specific parsing/semantics stay outside generic reducers;
- capability/evidence checks are preferred to runtime-name checks;
- do not perform a speculative adapter refactor while only Codex is real evidence.

Before runtime #2:
- Runtime Adapter Contract V1;
- Token Semantics V2;
- Runtime Capability Manifest V1;
- sanitized Conformance Fixture Kit + contract tests.

Runtime #2 must test and correct the Core/Adapter separation using real evidence.
Runtime #3 is the architecture maturity milestone: principal reducers should need little or no provider-specific modification.
A public Adapter SDK is deferred until at least 2-3 real adapters have exercised the internal contract.

Canonical design: `docs/design/MULTI-RUNTIME-CORE-AND-DATA-ARCHITECTURE.md` and ADR-006.
## Regle de passage

Une phase est DONE uniquement si:
- tests techniques PASS;
- evidence suffisante;
- documentation synchronisee;
- aucune affirmation plus forte que la preuve;
- aucune regression des golden facts.

Research direction does not override current evidence. New components enter the critical path only when their prerequisite/opportunity is measured.
