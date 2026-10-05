# Tokn Observer - Status

Derniere mise a jour : **2026-10-04**
Release de travail : **V0.1 Measurement Hardening**
Binaire/Cargo : **V0.1 / 0.1.0**

## Resume

**P0-P9 DONE. Experiment 002 est ACCEPTED. Measurement Contract V1 est FROZEN. Store foundation est DONE. Local MCP transport prototype est ACCEPTED. M4 Historical Analyzer + Context Ledger est CORE ACCEPTED avec EXTENSIONS EVIDENCE-GATED ; Context/Result Identity V0, Cross-Agent Evidence V2, Source Mutation Observation V0, Run-Boundary Source Version V0 et Workspace Git Provenance V0 sont ACCEPTED FOUNDATIONS ; Source Freshness Evidence V0 est ACCEPTED CORROBORATION FOUNDATION ; Cross-Run Comparability V0 et Task Input Identity V0 sont ACCEPTED OBSERVATION FOUNDATIONS ; Cross-Run Source Re-read Evidence V0 est ACCEPTED CHRONOLOGY FOUNDATION. M4.5 Context Identity + Shadow Foundations est STARTED / OBSERVATION-ONLY.**

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
- PowerShell syntax validation : PASS ;
- benchmark manifest validation : PASS ;
- documentation consistency : PASS ;
- git diff --check : PASS ;
- verdict golden INSTRUMENTATION_ONLY ;
- causal_claims_allowed=false ;
- descriptive_metrics_allowed=true.

Build environment note:
Visual Studio Community 2026 is incomplete for native C builds,
but the already-installed Visual Studio Build Tools 2022 toolchain is complete
and is the validated build environment.
Rustup is now installed locally; `rust-toolchain.toml` pins Rust 1.97.1 + rustfmt + Clippy so local checks match CI.

Maintainability/audit checkpoint 2026-10-04 : MATERIAL DEBT CONFIRMED. Wave A code remediation is complete: R-01 JSONL early-EOF/shrink handling, D-01 SQLite connection guards, CI supply-chain hardening and S-05 publication/privacy hardening are fixed and merged. C-06 GitHub branch governance remains an administration-side pending control. Wave B S-02 storage-level MCP read-only open is fixed and merged. S-04 quality-gate process hardening is the current bounded slice: fixed 120 s timeout, 64 KiB retained per stdout/stderr stream while excess output is drained, allowlisted child environment, best-effort common-secret redaction, and Windows timeout tree termination by exact spawned PID. This is process hardening, not a sandbox or cryptographic secrecy guarantee. Remaining audit findings stay pending. See `docs/audits/2026-10-04-CLAUDE-TECHNICAL-AUDIT-RECONCILIATION.md`.
Git local : `main` aligne sur `origin/main` (GitHub).
GitHub governance : `main` est actuellement non protegee (`protected=false`) ; le workflow PR + CI + garde SHA est applique par discipline, mais une protection/ruleset GitHub reste a configurer cote administration.

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
Experiment 001 golden : inchange et replay PASS avec toutes les versions du contrat.

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

## M4 - Historical Analyzer + Context Ledger - core accepte

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
- phase timeline semantique = `NOT_CAPTURED` sans marqueur de phase autoritatif ;
- aucun finding causal ni optimisation active.

Le gate `docs/design/M4-EXIT-GATE.md` classe le coeur M4 comme **ACCEPTED**. Les signaux non directement observables restent des extensions evidence-gated et ne sont pas remplaces par des heuristiques.

## Prochaine action

**Implementation pause / audit checkpoint:** the initial Shadow design/reference/unicode61/pilot sequence through PR #22 is complete. The 2026-10-04 external technical audit has been reconciled against current `main`; no remediation has been applied. Do not start a new backend/foundation or audit fix until the owner approves the remediation/value-spike order in `docs/audits/2026-10-04-CLAUDE-TECHNICAL-AUDIT-RECONCILIATION.md`.

Le design + contrat de mesure du **Shadow Repository Index V0** sont ACCEPTED et `DIRECT_SCAN_V0` est **ACCEPTED REFERENCE FOUNDATION**. Le capability probe SQLite/FTS5 est **ACCEPTED CAPABILITY FOUNDATION / FULL GATES PASS**. `SQLITE_FTS5_UNICODE61_V0` reste **FUNCTIONAL BENCHMARK CANDIDATE / FULL GATES PASS**. Le premier pilote sanitise fige maintenant corpus/query/gold/K/repetitions/seuils avant execution et compare DIRECT_SCAN au candidat via `tokn-observe shadow-index-pilot`. DIRECT_SCAN est `BASELINE_ONLY`; le pilote unicode61 ne peut pas devenir eligible dans ce slice et le fixture connu fait echouer le plancher qualite a 95 %, donc le verdict mesure est **REJECTED pour cette condition unicode61**. Ce verdict ne selectionne aucun autre backend, ne rejette pas trigram et ne mesure aucune economie de tokens. Aucun backend FTS persistant, refresh incremental complet ni injection active de contexte n'existe encore.

Cross-Run Source Re-read Evidence V0 reste **ACCEPTED CHRONOLOGY FOUNDATION** : `run_created_at_unix` n'est jamais une chronologie runtime et rediscovery/redundancy/freshness restent `NOT_PROVEN`. Task Input Identity V0 garde `delivery_status=NOT_PROVEN`. La compaction detaillee et la phase timeline restent `NOT_CAPTURED` tant qu'aucune preuve directe fixtureable n'existe.

Cross-Run ajoute `TASK_INPUT_IDENTITY`, mais meme quand tous les axes observes passent, `causal_claims_status=NOT_ESTABLISHED` : l'identite d'artefact ne prouve pas la livraison runtime, les dependances externes ni le controle d'une variable primaire. Source Freshness Evidence garde en parallele `freshness_status=NOT_PROVEN` et `invalidation_status=NOT_PROVEN`.
Ne pas persister les tool outputs bruts par defaut uniquement pour obtenir une identite.
Ne pas creer artificiellement du per-turn a partir des agregats V1.
Ne pas commencer M5 Findings + Opportunity Analyzer tant que ces observations historiques ne sont pas fiables.

En parallele, Benchmark Baseline Protocol V1 reste PREPARED/ACCEPTED pour la variance native. Shadow Index Benchmark Protocol V0 + `Tokn ShadowIndexMeasurement V1` figent maintenant la mesure retrieval/build/refresh avant choix de backend. Aucun de ces protocoles ne pretend a une economie de tokens. Le premier A/B causal reste Experiment 003.

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
- Source Mutation Window V0 joins exact same-source same-thread read-before -> completed mutation -> read-after sequence evidence and reports exact content equality/difference only under complete unambiguous coverage; cross-thread/intervening mutation ambiguity fails to `UNKNOWN`, and causality remains `NOT_PROVEN`;
- Source Freshness Evidence V0 joins mutation-window chronology + exact SourceVersion BEFORE/AFTER + Workspace Git HEAD/dirty provenance without comparing ContentFingerprint and SourceVersionFingerprint directly; it may report change+reread corroboration, but freshness/invalidation remain `NOT_PROVEN`;
- Cross-Run Comparability V0 compares explicit baseline/candidate runs on captured project/contract/runtime/BEFORE-source/BEFORE-Git scope, never compares project-scoped fingerprints across projects, and keeps `causal_claims_status=NOT_ESTABLISHED`;
- `tokn-observe source-mutation-history` + `tokn-observe source-mutation-window-history` + `tokn-observe source-freshness-evidence` + `tokn-observe cross-run-comparison` read-only JSON surfaces;
- Run-Boundary Source Version V0 from Runner BEFORE/AFTER workspace snapshots;
- SourceVersionHistory V1 stores project-scoped `ver-v1-*` fingerprints keyed by the same SourceStableId, never raw paths or raw snapshot SHA-256 values;
- source boundary reducer reports `UNCHANGED_OBSERVED` / `CHANGED_OBSERVED` only with both boundaries, otherwise `UNKNOWN`;
- `tokn-observe source-version-history` read-only JSON surface;
- Workspace Git Provenance V0 from Runner BEFORE/AFTER ProjectSnapshot evidence;
- `workspace_git_provenance_v1` stores only coverage, dirty state and project-scoped `git-v1-*` HEAD fingerprints, never raw Git SHA values;
- legacy snapshots without Git evidence map to `NOT_CAPTURED`, failed/non-Git observation maps to `UNKNOWN`;
- `tokn-observe workspace-git-provenance-history` read-only JSON surface;
- `tokn-observe cross-agent-evidence` read-only JSON analysis surface.

M4 core exit state:
- Cross-Run Comparability V0 remains ACCEPTED and causal claims stay `NOT_ESTABLISHED`;
- Cross-Run Source Re-read Evidence V0 remains ACCEPTED and rediscovery/redundancy/freshness stay `NOT_PROVEN`;
- semantic phase timeline and detailed compaction chronology remain `NOT_CAPTURED` until direct fixtureable evidence exists;
- runtime task delivery remains `NOT_PROVEN`; broader source kinds stay evidence-gated;
- these missing signals do not block M4.5 and must be added only from direct evidence.

Stability / benchmark readiness 2026-10-03:
- local Rust 1.97.1 toolchain installed and aligned with CI;
- local Rust resource guard defaults to 2 Cargo jobs / 2 test threads; heavy exceptional work should use TOKN_CARGO_JOBS=1 and run sequentially;
- RustSec `cargo audit` PASS on 2026-10-04 for the current Cargo.lock (96 dependencies); this remains a point-in-time manual audit, not a CI/supply-chain gate or substitute for release-time SBOM/dependency review;
- `scripts/dev-check.ps1` is the canonical local pre-PR gate;
- PowerShell syntax validation is part of CI;
- packaging is clean-worktree + tracked-files-only, with package-wide privacy scanning and provenance;
- Benchmark Baseline Protocol V1 + BenchmarkManifest V1 are accepted for descriptive native baselines;
- Shadow Repository Index V0 design + Shadow Index Benchmark Protocol V0 + ShadowIndexMeasurement V1 are accepted pre-implementation M4.5 contracts; implementation/backend selection remain pending measurement;
- repository-admin action still recommended: protect `main` with required PR + CI before merge.

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
