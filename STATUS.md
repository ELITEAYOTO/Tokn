# Tokn Observer - Status

Derniere mise a jour : **2026-10-03**
Release de travail : **V0.1 Measurement Hardening**
Binaire/Cargo : **V0.1 / 0.1.0**

## Resume

**P0-P9 DONE. Experiment 002 est ACCEPTED. Measurement Contract V1 est FROZEN. Store foundation est DONE. Local MCP transport prototype est ACCEPTED. Historical Analyzer + Context Ledger est IN PROGRESS ; Context/Result Identity V0, Cross-Agent Evidence V2, Source Mutation Observation V0 et Run-Boundary Source Version V0 sont ACCEPTED FOUNDATIONS.**

Le projet est revenu sur une direction unique :
mesurer l'efficacite du contexte sans reduire la capacite d'Astra.

Hard output caps :
**depriorises comme strategie d'optimisation**.
Ils restent une evidence/diagnostic historique.

## Validation technique courante

P9 V0.1 release gate :
- cargo fmt --check : PASS ;
- Clippy workspace/all-targets -D warnings : PASS ;
- cargo test --workspace : PASS ;
- build release tokn-observe : PASS ;
- sanitized fixture/privacy tests : PASS ;
- deterministic Experiment 001 Runner golden replay : PASS ;
- package privacy validation : PASS ;
- documentation consistency : PASS ;
- git diff --check : PASS ;
- verdict golden INSTRUMENTATION_ONLY ;
- causal_claims_allowed=false ;
- descriptive_metrics_allowed=true.

Build environment note:
Visual Studio Community 2026 is incomplete for native C builds,
but the already-installed Visual Studio Build Tools 2022 toolchain is complete
and is the validated build environment.

Maintainability audit : CLEAN / NO MAJOR REFACTOR REQUIRED.
Git local : `main` aligne sur `origin/main` (GitHub).

## V0.1

P0 - sanitized fixtures : DONE
P1 - source health + fallback : DONE
P2 - RunGroup + agent graph : DONE
P3 - terminal task status : DONE
P4 - session tool adapter : DONE
P5 - dynamic workspace tracking : DONE
P6 - policy evidence/runtime capability research : DONE
P7 - experiment validity reducer : DONE
P8 - runner V0.1 : DONE
P9 - golden replay/release validation : DONE

P6 est ferme avec une limitation connue :
Codex Desktop 0.158.0-alpha.2.1 ne transmet pas `max_output_tokens`
au PreToolUse Bash de unified exec.

La sonde live JSONL a zero ligne reste une dette diagnostique,
mais elle ne bloque plus V0.1 car l'enforcement des caps n'est pas un objectif produit.

## Experiment 002 - run reel accepte

Verdict :
**INSTRUMENTATION_ONLY / ACCEPTED FOR MEASUREMENT VALIDATION**

Run reel du 2026-10-01 :
- runtime observe : `codex-cli 0.161.0-alpha.2` ;
- modele observe : `gpt-6.1-sol` ;
- 1 parent + 1 sous-agent ;
- 49 usage records ;
- input 3,328,582 ;
- cached input 3,168,256 ;
- output 13,394 ;
- reasoning output 3,734 ;
- terminal parent : COMPLETED ;
- pipeline Runner : COMPLETE ;
- quality gate : PASS, `verify:local`, 863/863 tests ;
- workspace : `JEM_Ultimate_v0.12.1_B07-C_WORKING\\PROJECT` ;
- diff : 1 added / 2 modified / 0 removed ;
- validity : INSTRUMENTATION_ONLY ;
- causal_claims_allowed=false ;
- descriptive_metrics_allowed=true ;
- configuration_recorded=UNKNOWN.

Le premier FINISH a revele deux defauts de harness :
1. `$LASTEXITCODE` ne devait pas etre teste apres un script PowerShell interne ;
2. la trace CLI place le bundle diagnostic sous un sous-dossier `trace-*`,
   et ce layout n'est pas directement consommable par le resolver V0.1.

Les preuves originales ont ete conservees.
Le run n'a pas ete relance : Tokn a recupere le rollout Codex persiste parent + sous-agent,
puis a rejoue le Runner de maniere deterministe.
Le fallback session est maintenant versionne et couvert par CI.

Cette recuperation ne transforme pas Experiment 002 en experience causale.
Elle valide uniquement la chaine de mesure et documente explicitement la dette de configuration runtime.

## Golden Experiment 001

Verdict :
**INSTRUMENTATION DISCOVERY SUCCESS / OPTIMIZATION INVALID**

RunGroup :
- 1 parent ;
- 3 sous-agents ;
- 80 usage records.

Tokens :
- input 5,285,737 ;
- cached 5,059,712 ;
- uncached 226,025 ;
- output 26,021 ;
- reasoning 3,018 ;
- logical total 5,311,758.

Attribution :
- parent 2,499,523 (~47.1 %) ;
- descendants 2,812,235 (~52.9 %).

Etat :
- terminal INCOMPLETE_USAGE_LIMIT ;
- diagnostic PARTIAL ;
- policy observed FAIL 58/17 ;
- policy enforcement SUPPORTED_INSUFFICIENT_INPUT ;
- workspace B07-C_WORKING\PROJECT ;
- diff 13 added / 4 modified / 0 removed ;
- verify:local PASS, 814 tests vs 809 au depart ;
- experiment validity attendue : INSTRUMENTATION_ONLY.

## Interpretation cache

Experiment 001 a ~95.72 % de cached input.

Conclusion autorisee :
la majorite des input tokens reportes etait classee cached.

Conclusions interdites :
- le contexte etait optimal ;
- les cached tokens etaient gratuits ;
- ils ne comptaient pas dans le quota produit ;
- tout le contexte etait utile ;
- il faut reduire la fenetre ou la sortie d'Astra.

La prochaine couche d'analyse doit localiser l'uncached growth,
la duplication et la rediscovery avant de proposer une intervention.

## Direction produit

Decision :
`docs/decisions/ADR-003-QUALITY-PRESERVING-EFFICIENCY.md`

Strategie :
`docs/strategy/CONTEXT-EFFICIENCY-STRATEGY.md`

Recherche revue :
`docs/research/2026-09-29-CONTEXT-CACHE-RESEARCH.md`

Axes apres V0.1 :
- Context Ledger ;
- Cache Efficiency Analyzer ;
- Context Duplication Analyzer ;
- Parent/Subagent Context Graph ;
- Tool Evidence Footprint ;
- Project Memory Candidate Detector.

Tous commencent observation-only.

## Measurement Contract V1 - freeze termine

Contrat : `tokn.measurement.v0.1`.

Le Runner emet maintenant `measurement-contract.json` et le replay golden verrouille
les versions V1 des schemas, de l'evidence layout et des semantiques de mesure.

Points figes :
- RunnerRequest / RunnerResult ;
- evidence folder layout ;
- RunGroup identity ;
- token accounting (`ordinary_uncached = input - cached`, cache-write separe) ;
- source health/provenance ;
- terminal status ;
- workspace inventory/resolution/snapshot/diff ;
- quality/policy/recovery/validity contracts ;
- ModelRuntimeProfile V1 domain contract ;
- analyzer semantics version.

Versions workspace/snapshot inconnues : rejet fail-closed.
Experiment 001 golden : inchangé et replay PASS avec toutes les versions du contrat.

## Store V2 foundation - termine

Le Store V2 persiste maintenant :
- project/workspace identities pseudonymisees ;
- runs + agents ;
- usage summaries avec compteurs de couverture ;
- provenance par fingerprint ;
- ModelRuntimeProfile V1.

`tokn-observe store-evidence` ingere un dossier Runner V1 sans modifier le contrat Runner.
L'ingestion est idempotente et le replay Experiment 001 la rejoue deux fois avec succes.

Privacy : aucun chemin workspace/session brut n'est persiste dans le Store V2.
Les anciens chemins du schema legacy sont pseudonymises puis physiquement nettoyes par VACUUM lors de la migration V2.
Les versions futures inconnues sont refusees fail-closed.

L ingestion de l historique rate-limit est maintenant acceptee dans M4. Findings et Experiment Lab persistence restent differes.

## Local MCP transport prototype - accepte

Implementation : `apps/tokn-mcp`.

Validation :
- unit tests + Clippy strict PASS ;
- vrai process release stdio PASS ;
- `tokn_status` + `tokn_recent_runs` + `tokn_context_ledger` read-only ;
- enregistrement stdio accepte par Codex 0.161.0-alpha.2 dans un `CODEX_HOME` isole ;
- `codex app-server` lance Tokn, lit `serverInfo`, les capabilities et les trois tools ;
- `toolsError=null` ;
- config utilisateur normale non modifiee ;
- thread ephemere cree avec etat idle et 0 turns ;
- `mcpServer/tool/call` appelle reellement `tokn_status` puis `tokn_context_ledger` via Codex ;
- retour confirme `tokn-mcp 0.1.0`, stdio, read_only=true, Store schema 2 et Context Ledger schema 1 ;
- le ledger expose explicitement per-turn = `NOT_CAPTURED` et current retained context = `UNKNOWN` ;
- aucune auth utilisateur copiee ou inspectee ;
- aucun turn modele ni quota volontairement consomme.

## M4 - Historical Analyzer + Context Ledger - en cours

Slice valide :
- `HistoricalSnapshot` V1 depuis Store V2, filtrable par project/workspace ;
- historique multi-run run/agent avec WorkspaceLineage ;
- token ledger coverage-aware : input/cached/cache-write/output/reasoning ;
- ordinary uncached et logical total seulement quand les entrees requises sont completes ;
- terminal, quality, validity, runtime profile et provenance conserves a cote de l'usage ;
- incoherences historiques signalees comme integrity issues, jamais reparees silencieusement ;
- CLI `tokn-observe context-ledger` ;
- MCP read-only `tokn_context_ledger` ;
- validation standalone + vrai Codex 0.161.0-alpha.2 PASS, thread ephemere idle / 0 turns.

Limites explicites du contrat V1 :
- per-turn ledger = `NOT_CAPTURED` ;
- current retained-context occupancy = `UNKNOWN` ;
- aucun finding causal ni optimisation active.

## Prochaine action

Continuer **M4** a partir des fondations d'identite, de mutation-operation et de version source aux frontieres du run maintenant acceptees :
provenance additionnelle (dont Git commit seulement quand directement observable), puis semantiques de fraicheur/invalidation fail-closed, compaction/rediscovery et comparaison cross-run explicite.

La mutation-operation peut maintenant etre datee quand le rollout fournit un timestamp, mais son effet reste `NOT_VERIFIED`. Cette fondation doit preparer un Context Twin V0 sans pretendre connaitre la memoire interne d'Astra.
Ne pas persister les tool outputs bruts par defaut uniquement pour obtenir une identite.
Ne pas creer artificiellement du per-turn a partir des agregats V1.
Ne pas commencer M5 Findings + Opportunity Analyzer tant que ces observations historiques ne sont pas fiables.

Chemin d'execution detaille :
`docs/plans/IMPLEMENTATION-PATH.md`.

## Sources de verite

Etat : `STATUS.md`
Ordre : `ROADMAP.md`
Implementation V0.1 : `docs/plans/V0.1-IMPLEMENTATION-PLAN.md`
Tests : `docs/plans/V0.1-TEST-MATRIX.md`
Documentation map : `docs/INDEX.md`

## P8 implementation progress - 2026-09-30

First Runner slice implemented:
- shared RunnerRequest / RunnerResult contract ;
- `tokn-observe runner <request.json>` CLI ;
- session root resolution + RunGroup reconstruction ;
- workspace resolution from before/after inventory ;
- deterministic evidence artifacts: runner-request.json, source-health.json, run-group.json, workspace-resolution.json, optional policy-evidence.json, runner-result.json ;
- explicit CORE_EVIDENCE_READY / BLOCKED pipeline state ;
- no overwrite of Runner-owned evidence artifacts ;
- offline fixture smoke test PASS.

Implementation P8 est feature-complete et fermee.
Le replay Experiment 001 est maintenant valide comme gate P9.

## P8 final implementation slice - 2026-09-30

Delivered:
- normalized self-contained session-evidence.json from exact RunGroup members ;
- historical-compatible SHA-256 project snapshot diff reducer in Rust ;
- imported before/after snapshots plus workspace-diff.json ;
- after snapshot root must match the P5-selected workspace or the pipeline is BLOCKED ;
- partial Runner artifacts are recoverable when runner-result.json is absent ;
- completed evidence is immutable by default and overwrite is refused ;
- Runner V0.1 does not install temporary policy files, so recovery records policy_placements_mutated=false ;
- complete fixture pipeline now reports COMPLETE with no pending steps.

P8 is DONE: the Runner contract, complete evidence lifecycle, fail-closed behavior and recovery acceptance tests pass. The historical Experiment 001 golden replay is a P9 release-validation gate.

## Update 2026-10-02 - M4 activity slice accepted

GitHub validation for branch `m4-activity-history` at `fa1d831` is PASS:
- publication privacy PASS;
- rustfmt / Clippy -D warnings / workspace tests PASS;
- release build + provenance PASS;
- MCP smoke PASS;
- Experiment 001 golden replay PASS;
- Experiment 002 source-selection regression PASS;
- documentation consistency PASS.

Accepted M4 additions:
- ToolActivityHistory V3 additive schema, ajoutant `observed_at` optionnel aux preuves V2;
- privacy-safe project-scoped operation/workdir fingerprints;
- ActivityTimeline V1 per agent;
- exact repeated-operation observations across runs/threads;
- no raw command/workdir/parse-error persistence;
- synthetic-secret persistence regression;
- `tokn-observe activity-timeline` CLI;
- automatic ModelRuntimeProfile compatibility reducer with PASS/FAIL/UNKNOWN evidence semantics;
- causal compatibility is PASS only when required runtime/model/config evidence is complete and equal;
- RateLimitHistory V1 with privacy-minimized limit/window/reset evidence;
- `tokn-observe rate-limit-history` filtered historical query;
- no credit balance, plan/account identity or `limit_name` retained in Tokn rate-limit evidence;
- Cross-Agent Evidence V2 for same-run exact operation overlap across distinct threads;
- parent/child, ancestor/descendant, sibling and unknown-lineage relation reporting;
- ToolActivityHistory schema V3 with separate `source_stable_id` / `content_fingerprint` evidence, explicit coverage and optional rollout `observed_at`;
- privacy-safe project-scoped exact result fingerprints only when one observed tool result maps unambiguously to one normalized operation;
- `OBSERVED` / `PARTIAL` / `NOT_CAPTURED` / `UNKNOWN` result-identity semantics;
- Cross-Agent result comparison is `SAME` or `DIFFERENT` only under complete observed identity, otherwise `UNKNOWN`;
- raw tool outputs are never persisted by the identity slice;
- SourceStableId file V0 derived from workspace-relative simple `Get-Content` evidence under the selected workspace;
- source IDs use a project-scoped source-specific derivation domain distinct from content fingerprints;
- SourceIdentityHistory V1 reports `UNCHANGED_OBSERVED` / `CHANGED_OBSERVED` only under complete exact content identity, otherwise `UNKNOWN`;
- `tokn-observe source-identity-history` read-only JSON surface;
- Source Mutation Observation V0: conservative `Set-Content` / `Add-Content` target identity plus rollout timing when directly observed;
- SourceMutationHistory V1 preserves `tool_status` but keeps `effect_status=NOT_VERIFIED`;
- `tokn-observe source-mutation-history` read-only JSON surface;
- Run-Boundary Source Version V0 from Runner BEFORE/AFTER workspace snapshots;
- SourceVersionHistory V1 stores project-scoped `ver-v1-*` fingerprints keyed by the same SourceStableId, never raw paths or raw snapshot SHA-256 values;
- source boundary reducer reports `UNCHANGED_OBSERVED` / `CHANGED_OBSERVED` only with both boundaries, otherwise `UNKNOWN`;
- `tokn-observe source-version-history` read-only JSON surface;
- `tokn-observe cross-agent-evidence` read-only JSON analysis surface.

Next M4 work:
- Git-commit/additional provenance and verified mutation effect only when directly observable;
- freshness/invalidation semantics that preserve `UNKNOWN` / `NOT_CAPTURED` instead of inferring staleness;
- broader source kinds plus compaction/rediscovery only when directly observable;
- explicit cross-run comparison primitives.

Before Experiment 003: characterize run-to-run variance, predeclare multidimensional quality gates, and measure Tokn overhead for any injected/default-active integration.

Before public binary distribution: threat model, retention/purge/export, dependency audit/SBOM, signing/update integrity and hostile parser/privacy corpus.

Long-term strategy review 2026-10-03 ACCEPTED as direction:
- M4.5 Context Identity / Context Twin seed before active retrieval;
- M5 includes Opportunity Analyzer V0;
- deterministic counterfactual replay + policy schema precede expensive A/B;
- AutoLab/ML stays a later offline-first track, not current M4-M6 scope.
Multi-runtime architecture review 2026-10-03 ACCEPTED as design direction:
- one provider-neutral analytical Core, many thin Runtime Adapters;
- capability/evidence-driven analysis instead of provider-name branching in generic reducers;
- runtime / model / provider / host / configuration remain distinct identities;
- `SourceStableId` and privacy-safe `ContentFingerprint` stay distinct in future Context Identity;
- before runtime #2: Runtime Adapter Contract V1, Token Semantics V2, Runtime Capability Manifest V1 and sanitized conformance fixtures;
- runtime #2 validates the abstractions; runtime #3 is the maturity test before any public Adapter SDK;
- normalized-event design is a logical/rebuildability direction, not a Store V2 rewrite mandate;
- UI/MCP/query surfaces should consume shared Core/Query contracts rather than couple directly to SQLite.
