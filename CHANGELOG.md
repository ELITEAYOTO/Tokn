# Tokn — Journal de développement

Ce fichier trace les modifications significatives avec date et heure locale.

## 2026-09-26 14:35 +02:00

### Vérification après interruption

- récupération de l'historique Remote Desktop Commander ;
- confirmation du workspace dans `E:\Tokn\V0-CodexTkn-Consume\tool` ;
- confirmation du renommage interne en Tokn ;
- vérification de l'arborescence des crates et scripts ;
- relecture du README, architecture, token-accounting et scripts de build.
### Validation technique

Commande de validation complète relancée :

```text
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo build --release --locked -p tokn-observe
```

Résultat : **PASS**.

Le binaire release a été généré avec succès.
### Vérification runtime

`tokn-observe doctor --dev` a confirmé :

- Codex Desktop : détecté ;
- Codex Desktop version : `codex-cli 0.158.0-alpha.2` ;
- deux installations Codex VS Code également détectées ;
- capabilities : session-rollouts, trace-reduce, prompt-input, app-server ;
- racine sessions : détectée ;
- 136 rollouts locaux trouvés.

### Documentation ajoutée

- `ROADMAP.md` ;
- `STATUS.md` ;
- `CHANGELOG.md` ;
- `docs\MAINTENANCE.md` ;
- README enrichi avec les règles de suivi.
## Travail effectué avant la coupure

- création du workspace multi-crates ;
- implémentation de `tokn-observe` ;
- détection Codex ;
- ingestion JSONL streaming/bornée ;
- fixtures minimal / cached usage / unknown event / truncated tail ;
- extraction `last_token_usage` ;
- Token Ledger initial ;
- invariants token de base ;
- stockage SQLite initial ;
- reporting texte ;
- scripts MSVC/test/build/package ;
- smoke test réel sur un rollout Codex ;
- renommage complet `pb-*` -> `tokn-*`.

Le détail fin reste visible dans l'historique de fichiers et les commits futurs.

## 2026-09-26 15:01 +02:00

### Robustesse Token Ledger

- d?duplication d?plac?e dans `tokn-analysis` ;
- ?tats Accepted / Duplicate / Conflict / Invalid ;
- doublon exact non recompt? ;
- conflit explicitement compt? ;
- invariant invalide non r?par? silencieusement ;
- `ordinary_uncached` corrig? pour respecter UNKNOWN.

### Idempotence

- hash BLAKE3 du snapshot ajout? ;
- run/source IDs d?terministes ;
- test hash stable ;
- test SQLite idempotent ;
- m?me rollout import? deux fois -> m?me `run_id` ;
- SQLite : 1 run, 1 source.

### Diagnostic Rollout Trace

- aucun bundle r?el trouv? ;
- fixture synth?tique cr??e ;
- extraction inference/response/thread/turn/provider ajout?e ;
- import fixture : 4 records, 2 usages, 1 doublon, 1 conflit.

### Validation

- cargo fmt : PASS ;
- cargo clippy `-D warnings` : PASS ;
- cargo test workspace : PASS.

## 2026-09-26 - Preparation du premier run JEM Ultimate trace

- clarification du KPI principal : qualite/travail utile par token, pas vitesse ;
- creation de `scripts/trace/start-jem-trace.ps1` ;
- creation de `scripts/trace/finish-latest-trace.ps1` ;
- creation de `START-JEM-TRACE.cmd` ;
- creation de `FINISH-JEM-TRACE.cmd` ;
- creation de `docs/jem-trace-workflow.md` ;
- validation syntaxique des deux scripts PowerShell : PASS ;
- aucun processus Codex n'a ete arrete automatiquement ;
- aucun projet JEM n'a ete modifie.

## 2026-09-26 - Premiere baseline JEM Ultimate reelle

- vrai Rollout Trace capture sur un run Astra/JEM Ultimate ;
- 221 records bruts ;
- 30 inference calls demarres ;
- 25 completes, 4 annules, 1 echoue ;
- 26 tool calls ;
- adapter diagnostic corrige pour suivre `response_payload.path` vers `payloads/N.json` ;
- test reproduisant la forme reelle du bundle ajoute ;
- totaux Tokn == reducer officiel ;
- ajout de `cache_write_input_tokens` au ledger agrege, SQLite et report ;
- ajout de `uncached input` au report ;
- migration SQLite retrocompatible ajoutee ;
- baseline documentee dans `docs/baselines/2026-09-26-jem-ultimate.md`.

## 2026-09-26 - Attribution Engine V0

- ajout de `InferenceObservation` et `ToolObservation` ;
- parseur des request/result payloads du bundle diagnostic ;
- ajout de `tokn-analysis::build_attribution` ;
- ajout de la commande `tokn-observe analyze` ;
- rapport top windows par tool output et input growth ;
- comptage des tools non attribues ;
- test d'attribution ajoute ;
- validation reelle sur le run JEM : PASS.

## 2026-09-26 - Attribution Engine V0

- ajout de InferenceObservation et ToolObservation ;
- parseur request/result payloads du bundle diagnostic ;
- ajout de tokn-analysis::build_attribution ;
- ajout de la commande tokn-observe analyze ;
- rapport top windows par tool output et input growth ;
- comptage des tools non attribues ;
- test d'attribution ajoute ;
- validation reelle sur le run JEM : PASS.

## 2026-09-27 - Raw vs cap + simulate-caps

- distinction raw tool output / output cap ;
- ajout max_output_tokens aux observations ;
- ajout result_output_chars ;
- attribution cap-adjusted upper bound ;
- comparateur A/B enrichi avec cap-adjusted metrics ;
- ajout de `tokn-observe simulate-caps` ;
- overrides generiques `--cap CATEGORY=TOKENS` ;
- tests unitaires simulation et parsing ;
- trois simulations offline validees sur la baseline JEM ;
- aucune modification active de Codex/Astra.

## 2026-09-27 - Experiment 001 real-run harness

- added check-caps command and cap-policy evidence model ;
- added project SHA-256 snapshot/diff scripts ;
- added conservative candidate policy overlay ;
- added exact prompt capture and clipboard workflow ;
- added runtime/environment version capture ;
- added automatic JEM verify:local after the model run ;
- added session evidence path/hash capture ;
- added crash/power-loss recovery ;
- added START/FINISH/RECOVER/EDIT user wrappers ;
- added Experiment 001 real-run runbook ;
- package now includes a clean experiment-kit without traces or run data.

## 2026-09-27 20:10 +02:00 - Experiment 001 post-mortem and V0.1 planning

### Forensic recovery

- candidate diagnostic trace found PARTIAL: 4 protocol/bookkeeping records only ;
- recovered real run from 4 standard Codex session rollouts ;
- grouped one parent + three subagents manually for post-mortem ;
- recovered 80 usage records ;
- input 5,285,737 ;
- cached input 5,059,712 ;
- uncached input 226,025 ;
- output 26,021 ;
- reasoning 3,018 ;
- logical total 5,311,758 ;
- subagents account for ~52.9 % of logical total ;
- all four threads ended with usage_limit_exceeded.

### Workspace findings

- original Experiment 001 snapshot watched B07-B_WORKING/PROJECT ;
- Astra correctly created B07-B_FROZEN and B07-C_WORKING ;
- recovered candidate diff: 13 added, 4 modified, 0 removed ;
- manual verify:local on real B07-C: PASS ;
- observed test count increased from 809 to 814.

### Policy findings

- temporary PROJECT/AGENTS.md was not present in session base_instructions ;
- parent explicitly discovered/read it as repository evidence ;
- soft policy was not reliably observed across tool requests ;
- most exec calls exposed no explicit max_output_tokens ;
- Experiment 001 optimization result classified INVALID / NOT TESTED.

### V0.1 design

Added:
- docs/INDEX.md ;
- docs/experiments/001-POSTMORTEM.md ;
- docs/design/V0.1-MEASUREMENT-ARCHITECTURE.md ;
- docs/design/EVIDENCE-AND-COVERAGE.md ;
- docs/design/SESSION-ROLLOUT-ADAPTER.md ;
- docs/design/AGENT-COST-ATTRIBUTION.md ;
- docs/design/WORKSPACE-TRACKING.md ;
- docs/design/POLICY-MODEL.md ;
- docs/design/EXPERIMENT-VALIDITY.md ;
- docs/decisions/ADR-001-V0.1-EVIDENCE-FALLBACK.md ;
- docs/decisions/ADR-002-NO-EVIDENCE-IS-NOT-PASS.md ;
- docs/plans/V0.1-IMPLEMENTATION-PLAN.md ;
- docs/plans/V0.1-TEST-MATRIX.md ;
- docs/plans/NEXT-SESSION-CHECKLIST.md ;
- docs/experiments/002-INSTRUMENTATION-VALIDATION.md.

Rewritten for current truth:
- README.md ;
- STATUS.md ;
- ROADMAP.md ;
- docs/architecture.md ;
- docs/token-accounting.md ;
- docs/diagnostic-trace.md ;
- docs/attribution-engine.md ;
- docs/MAINTENANCE.md.

### Accepted decisions

- standard session rollouts are mandatory fallback evidence ;
- NO_EVIDENCE can never be PASS ;
- parent/subagents must be aggregated as one RunGroup ;
- terminal task status is mandatory ;
- dynamic output workspaces must be tracked ;
- policy hint, observed compliance and enforcement are distinct ;
- Experiment 002 is blocked until V0.1 P0-P9 pass.

## 2026-09-28 18:01 +02:00 - V0.1 P0-P5 implementation and validation

### P0-P3 measurement foundation

- sanitized V0.1 fixtures added and privacy tests kept green ;
- diagnostic source health implemented ;
- PARTIAL diagnostic -> standard session fallback implemented ;
- session identity fixed to the first session_meta when parent metadata is replayed later ;
- RunGroup + agent graph implemented with nested-depth support ;
- Experiment 001 golden replay recovered automatically: 1 parent + 3 subagents ;
- 80 usage records, logical total 5,311,758, uncached input 226,025 ;
- terminal state recovered as INCOMPLETE_USAGE_LIMIT ;
- historical fallback hardened to use session_meta.timestamp first ;
- session discovery now covers both sessions and archived_sessions.

### P4 session tool/cap evidence

- Code Mode exec envelope parser added ;
- multiple tools.exec_command calls inside one Code Mode envelope supported ;
- command/workdir/max_output_tokens extracted from standard session rollouts ;
- non-exec Code Mode calls no longer count as parse failures ;
- PASS / FAIL / NO_EVIDENCE / INCOMPLETE_EVIDENCE enforced ;
- zero target can no longer be PASS ;
- Experiment 001 replay: 58 targeted tools, 17 violations, 0 unknown, 0 parse failures => FAIL.

### P5 dynamic workspace tracking

- shallow workspace inventory script added ;
- PowerShell 5.1 collection and UTF-8 BOM issues fixed ;
- workspace resolver added with tool-workdir evidence and fail-closed ambiguity handling ;
- generic watch-root evidence excluded from scoring ;
- Experiment 001 replay selects B07-C_WORKING\PROJECT ;
- START captures workspace-before ;
- FINISH captures workspace-after and resolves the actual output workspace before diff/verify ;
- verify:local now targets the resolved workspace ;
- TOKN_RUN_GROUP.txt and workspace-resolution artifacts integrated into run output.

### Validation

- cargo fmt: PASS ;
- Clippy -D warnings: PASS ;
- cargo test --workspace: PASS ;
- release build: PASS ;
- P5 release replay: SELECTED B07-C_WORKING\PROJECT ;
- START Experiment 001 dry-run: PASS while Codex remained open ;
- no real Codex run was launched by the dry-run ;
- next phase: P6 Policy placement and evidence.

## 2026-09-29 19:01 +02:00 - V0.1 P5 final hardening

- reprise apres coupures Remote Desktop Commander avec verification de l'etat reel des fichiers ;
- P5 replay historique rendu stable via session_meta.timestamp, mtime seulement en secours ;
- discovery etendue a .codex/sessions + .codex/archived_sessions ;
- filtre du watch root generique dans le scoring workspace ;
- replay Experiment 001 : B07-C_WORKING/PROJECT reste SELECTED, autres workspaces sans bruit artificiel ;
- analyze-run accepte maintenant --output-json ;
- RunGroup JSON valide sur Experiment 001 : 1 parent + 3 sous-agents, terminal INCOMPLETE_USAGE_LIMIT ;
- FINISH derive session-evidence.json des source_path exacts du RunGroup ;
- FINISH utilise le workspace resolu pour snapshot, diff et verify:local ;
- nettoyage conservateur du AGENTS.md experimental dans un workspace copie par Astra ;
- START capture workspace-before et FINISH capture workspace-after + workspace-resolution ;
- PowerShell syntax check START/FINISH/inventory/recover : PASS ;
- cargo fmt --check : PASS ;
- Clippy -D warnings : PASS ;
- cargo test --workspace : PASS ;
- cargo build --release -p tokn-observe : PASS ;
- START Experiment 001 -DryRun : PASS ;
- cleanup dry-run confirme : aucun ACTIVE-RUN, run temporaire ou trace temporaire restant ;
- aucune nouvelle experience Astra reelle lancee ;
- prochaine phase officielle : P6 Policy placement/evidence.

## 2026-09-29 - Maintainability audit and preventive cleanup

### Audit result

- project structure reviewed before continuing P6 ;
- verdict: CLEAN / NO MAJOR REFACTOR REQUIRED ;
- internal crate dependency direction remains layered and acyclic ;
- largest Rust files reviewed by production/test split ;
- finish-exp001-candidate.ps1 identified as temporary large experiment runner to be generalized in P8 ;
- only two production expect() calls found, both protected by prior invariants ;
- no broad rewrite recommended.

### Preventive refactor

- session-root fallback resolution centralized in CLI common.rs ;
- duplicated fallback implementations removed from analyze-run, check-caps and resolve-workspace ;
- unused serde_json dependency removed from tokn-analysis ;
- no behavior change intended.

### Validation after refactor

- cargo fmt : PASS ;
- Clippy -D warnings : PASS ;
- cargo test --workspace : PASS ;
- Experiment 001 analyze-run replay : PASS, 4 agents, logical total 5,311,758 ;
- Experiment 001 check-caps : FAIL as expected, 58 targets / 17 violations / complete evidence ;
- workspace resolver : B07-C_WORKING/PROJECT SELECTED.

### Maintenance gap

- no .git repository currently exists in V0-CodexTkn-Consume or tool ;
- CI workflow is present and valid but cannot provide history/remote protection until Git is initialized ;
- workspace Cargo version remains 0.0.0 while V0.1 is under active hardening ; version normalization deferred to P9.

### Documentation

- added docs/MAINTAINABILITY-AUDIT-2026-09-29.md ;
- maintainability thresholds added to docs/MAINTENANCE.md.

## 2026-09-29 - P6 policy evidence pipeline started

### Maintenance baseline

- initialized a local Git repository in tool/ on branch main ;
- no remote configured and no push performed ;
- first baseline commit created before further P6 growth ;
- applied rustfmt cleanup found during the handoff audit.

### P6 implementation

- added PolicyObservationStatus and PolicyObservationSummary to the domain ;
- added RunGroup-level policy evidence aggregation ;
- added policy evidence text renderer ;
- added `inspect-policy` CLI with text + JSON output ;
- report separates policy_hint, policy_observed and policy_enforced ;
- Experiment 001 FINISH now emits TOKN_POLICY_EVIDENCE.txt and tokn-policy-evidence.json ;
- START records the policy marker and initial placement path.

### Real Experiment 001 replay

- policy_hint: PRESENT ;
- policy marker in session instructions: 0 / 4 threads ;
- explicit policy-file reads: 4 / 4 threads, 5 reads total ;
- policy_observed: FAIL ;
- targeted calls: 58 ;
- compliant: 41 ;
- violations: 17 ;
- unknown: 0 ;
- policy_enforced: NOT_PROVEN.

### Validation

- PowerShell syntax START/FINISH: PASS ;
- cargo fmt --check: PASS ;
- Clippy -D warnings: PASS ;
- cargo test --workspace: PASS ;
- release build: PASS.

P6 remains IN PROGRESS: generic multi-placement lifecycle and supported enforcement-hook research are still required.

## 2026-09-29 21:23 +02:00 - P6 placement + hook contract hardening

### Policy placement

- added reusable policy-placement.ps1 helper ;
- per-path backup, installed SHA-256 and conservative restoration ;
- START now places the hint at PROJECT and workspace watch root ;
- FINISH and RECOVER support multi-placement and legacy runs ;
- copied output workspaces are restored only on exact known overlay hash ;
- unexpected AGENTS.md content is preserved rather than silently removed ;
- full START -DryRun passes with both placements restored and no residue.

### Hook research

- local Codex 0.158.0-alpha.2.1 reports hooks as stable ;
- public Codex docs confirm PreToolUse can mediate Bash and nested Code Mode calls ;
- public contract guarantees tool_input.command ;
- public contract does not currently guarantee Tokn's needed max_output_tokens field ;

- enforcement status therefore refined to SUPPORTED_UNVERIFIED ;
- hidden hook-pre-tool-use prototype remains disabled by default ;
- synthetic handler tests cover compliant, missing-cap, over-cap and non-target commands ;
- optional JSONL audit stores only IDs/category/caps/decision, never command text ;
- example hooks.json is documentation-only and is not installed into Codex.

### Validation

- PowerShell syntax START / FINISH / RECOVER / policy-placement: PASS ;
- synthetic install/restore lifecycle: PASS ;
- hook example JSON parse: PASS ;
- dry-run cleanup: PASS, no ACTIVE-RUN/run/trace residue ;
- E:\BlockBench-Plugin\AGENTS.md absent after dry-run ;
- source PROJECT\AGENTS.md absent after dry-run ;
- no real Codex/Astra experiment launched in this pass.

P6 remains IN PROGRESS until one controlled real PreToolUse callback proves the actual payload contract.

### Final validation for this P6 pass

- cargo fmt --check: PASS ;
- Clippy -D warnings: PASS ;
- cargo test --workspace: PASS ;
- cargo build --release -p tokn-observe: PASS ;
- historical inspect-policy replay: FAIL 58/17 as expected ;
- historical enforcement classification: SUPPORTED_UNVERIFIED.

## 2026-09-29 21:48 +02:00 - P6 real-hook probe harness

- added observation-only hook-probe-pre-tool-use command ;
- probe logs callback schema/IDs/category and cap presence, never command text ;
- added disposable p6-hook-probe workspace ;
- added START / FINISH / RECOVER P6 hook-probe wrappers ;
- START installs a temporary user hooks.json only when none already exists ;
- probe prompt is copied automatically to clipboard and Codex Desktop is launched ;
- FINISH restores/removes the temporary hook and emits summary.json ;
- synthetic FINISH test: PASS ;
- probe START dry-run and cleanup: PASS ;
- no real Codex callback captured yet.

## 2026-09-29 22:00 +02:00 - First real P6 hook probe diagnosis

- first real probe finished with 0 callbacks ;
- Astra run itself was successfully recovered from standard session rollouts ;
- root + subagent used Code Mode custom tool `exec` and nested `tools.exec_command` ;
- nested read calls explicitly requested `max_output_tokens: 5000` ;
- runtime execution events report source `unified_exec_startup` ;
- current OpenAI hook docs confirm unified exec / exec_command is covered by PreToolUse and matches `Bash` ;
- therefore the original matcher was correct ;
- most likely cause of 0 callbacks: temporary non-managed hook was not reviewed/trusted ;
- probe now also records SessionStart so activation can be distinguished from tool coverage ;
- START instructions now require explicit hook trust before the test turn.

## 2026-09-29 - P6 live hook contract resolved for Desktop 0.158

- controlled Desktop probe used hooks explicitly trusted in packaged Codex CLI ;
- Codex local logs prove feature.hooks=true and hook/started -> hook/completed during nested Code Mode exec calls ;
- rollouts prove model-side tools.exec_command calls requested max_output_tokens=6000 ;
- exact OpenAI tag rust-v0.158.0-alpha.2.1 proves ExecCommandHandler maps unified exec to Bash but forwards only tool_input.command to PreToolUse ;
- max_output_tokens is therefore unavailable to this callback on the current Desktop runtime ;
- added SUPPORTED_INSUFFICIENT_INPUT enforcement state ;
- Experiment 001 replay now reports policy_enforced=SUPPORTED_INSUFFICIENT_INPUT while preserving 58 targets / 17 violations ;
- active hook prototype now treats a missing cap as UNOBSERVABLE/fail-open instead of a false DENY ;
- probe now logs event/top-level metadata before event-specific assumptions ;
- FINISH no longer infers inactive/untrusted hooks from an empty Tokn audit alone ;
- temporary hook/debug artifacts cleaned ; no ~/.codex/hooks.json or active P6 probe remains.

## 2026-09-29 - Documentation reset and context-efficiency strategy

- integrated the deep-research findings as a reviewed research document rather than an unverified source of truth ;
- accepted ADR-003: optimize context efficiency, not model capability ;
- hard output caps moved from product strategy to historical/diagnostic evidence ;
- P6 closed with the documented SUPPORTED_INSUFFICIENT_INPUT runtime limitation ;
- live zero-row hook audit retained as non-blocking diagnostic debt ;
- P7 restored as the single next implementation phase: Experiment Validity Reducer ;
- P8 remains the self-contained V0.1 runner and P9 the golden/release gate ;
- added Context Efficiency Strategy: Context Ledger, cache analysis, duplication analysis, parent/subagent graph, tool evidence footprint and Project Memory candidates ;
- Experiment 001 cached/input ratio documented as ~95.72% with explicit limits on interpretation ;
- README / STATUS / ROADMAP / docs INDEX / maintenance / V0.1 plan / test matrix / next-session checklist synchronized ;
- historical Experiment 001 cap/runbook/baseline documents explicitly labeled historical ;
- documentation governance now separates status, roadmap, plans, design, decisions, research and experiments.

## 2026-09-29 - P7 experiment validity reducer started

- added ADR-004 experiment validity and structured causal claim gate ;
- added machine-readable validity domain model with PASS / FAIL / UNKNOWN / NOT_REQUIRED ;
- added five verdicts: VALID_FOR_CAUSAL_AB, VALID_FOR_DESCRIPTIVE_COMPARISON, INSTRUMENTATION_ONLY, INVALID_CAPTURE, INCOMPLETE_TASK ;
- added conservative reducer with explicit verdict precedence and structured reason codes ;
- added Experiment 001 validity fixture and golden reducer test ;
- added text renderer with explicit causal winner/savings BLOCKED/ALLOWED gate ;
- added `tokn-observe evaluate-validity <input.json> --output-json <path>` command ;
- Experiment 001 harness result: INSTRUMENTATION_ONLY, causal_claims_allowed=false, descriptive_metrics_allowed=true ;
- core validation: cargo fmt PASS, tokn-domain + tokn-analysis 26 tests PASS, Clippy -D warnings PASS ;
- validity renderer standalone Rust test PASS and CLI source standalone compile PASS ;
- Visual Studio Community 2026 was incomplete for native C builds, but existing Visual Studio Build Tools 2022 provided the complete validated toolchain ;
- full workspace tests PASS, workspace Clippy -D warnings PASS, release tokn-observe build PASS ;
- real release CLI fixture matrix PASS: causal, descriptive-only, incomplete-task, invalid-capture, plus Experiment 001 golden ;
- P7 closed DONE ; P8 is NEXT.

## 2026-09-30 - Hybrid architecture and Context Pack groundwork

- accepted ADR-005: Rust Engine remains the analytical source of truth; integrations stay thin ;
- added target architecture for Engine / Store / runtime adapters / future Desktop / future Optimizer ;
- added external research roadmap that does not block P8/P9 by default ;
- added Codex capability matrix separating OBSERVED, RESEARCH and UNKNOWN ;
- added compact OPEN-QUESTIONS registry ;
- added Context Pack manifest template with Git-as-source-of-truth and integrity rules ;
- corrected Experiment 002 dependency wording from P7-P9 to P8-P9 now that P7 is DONE ;
- documentation index updated for the new durable references.

## 2026-09-30 - Documentation and workspace reorganization

- kept the Git repository root minimal and moved the Context Pack template under docs/reference ;
- moved flat technical docs into design/, reference/, operations/ and audits/ ;
- documented the canonical outer workspace layout ;
- moved historical raw research inputs and pre-Tokn scripts into archive/ ;
- moved manual .cmd wrappers into launchers/ and updated tracked references/package lookup ;
- moved Context Pack snapshots under the project-level context-packs/ directory ;
- preserved experiments/, traces/ and artifacts/ as non-canonical runtime/evidence areas ;
- no runtime evidence or experiment data was deleted.

## 2026-09-30 - Reproducible Context Pack generator

- added scripts/context/build-context-pack.ps1 ;
- canonical Context Packs now require a clean worktree and are emitted under outer context-packs/ ;
- pack manifest records source branch/commit and per-file SHA-256 inventory ;
- maintenance and manifest template now point to the reproducible generator.

## 2026-09-30 - Codex plugin and telemetry research pass

- verified current portable plugin packaging and compatibility fallback from official OpenAI docs ;
- verified local/repo marketplace model and current Codex plugin CLI commands ;
- verified plugin hook loading/trust model and current PreCompact hook documentation ;
- observed current local runtime: codex-cli 0.161.0-alpha.2 / OpenAI.Codex 26.928.1915.0 ;
- observed direct session JSONL token, cache-write, context-window, model/effort and agent-link metadata ;
- verified user-facing Codex /status allowance visibility while keeping programmatic remaining budget UNKNOWN ;
- updated research roadmap, capability matrix, open questions and target architecture ;
- no research result blocks P8/P9.

## 2026-09-30 - R5-R8 integration research closure

- observed OpenAI-installed Codex plugins launching local MCP processes through command/args/cwd compatibility manifests ;
- selected a process-bound MCP adapter as the preferred Tokn local integration prototype after P8/P9, without committing to a daemon ;
- observed machine-readable rate_limits in local token_count events without persisting personal quota values ;
- matched those fields to current openai/codex RateLimitSnapshot source semantics ;
- defined the accepted ModelRuntimeProfile minimum schema and privacy exclusions ;
- documented the local-first plugin/MCP privacy boundary from official OpenAI guidance ;
- kept current context occupancy and Tokn-specific explicit skill invocation as open research questions ;
- P8/P9 remain unblocked and are again the next product work.

## 2026-09-30 - Implementation path frozen

- added docs/plans/IMPLEMENTATION-PATH.md as the operational path from P8 through post-V0.1 layers ;
- synchronized STATUS, ROADMAP, NEXT-SESSION-CHECKLIST and docs/INDEX ;
- updated target architecture with the preferred process-bound local MCP prototype after P8/P9 ;
- kept Store/plugin/GUI/active optimization explicitly out of P8 ;
- immediate engineering work is now P8 Runner V0.1 only.

## 2026-09-30 - P8 Runner core evidence slice

- added shared RunnerRequest / RunnerResult domain contract ;
- added `tokn-observe runner <request.json>` ;
- runner now orchestrates root-session resolution, RunGroup reconstruction and workspace resolution ;
- writes normalized request, run-group, workspace-resolution and runner-result evidence artifacts ;
- adds explicit CORE_EVIDENCE_READY / BLOCKED states so partial P8 progress cannot be mistaken for completion ;
- offline sanitized fixture smoke test PASS ;
- P8 remains IN PROGRESS until source health, policy, quality, validity and recovery are integrated.

## 2026-09-30 - P8 source-health persistence

- Runner now persists source-health.json before RunGroup analysis ;
- source kind, source health, fallback recovery state and root session are recorded ;
- RunnerResult exposes source_health_status and fallback_recovered ;
- offline fixture validation PASS.

## 2026-09-30 - P8 policy evidence orchestration

- Runner now accepts optional typed policy evidence configuration ;
- reuses P6 session policy inspection and P4 cap-policy observation across the RunGroup ;
- persists policy-evidence.json without inferring enforcement from observed compliance ;
- RunnerResult exposes policy observation/enforcement status separately ;
- shared policy status mapping was deduplicated between inspect-policy and Runner ;
- offline policy smoke test PASS.

## 2026-09-30 - P8 quality gate orchestration

- added typed Runner quality request/report contract ;
- executes quality commands directly in the resolved output workspace ;
- persists quality-gate.json with PASS/FAIL/UNAVAILABLE/NOT_REQUIRED distinction ;
- required unavailable gates block the pipeline without erasing evidence ;
- offline PASS/FAIL/UNAVAILABLE smoke tests PASS.

## 2026-09-30 - P8 validity reducer orchestration

- Runner now builds and persists validity-input.json from measured evidence plus explicit non-inferable hints ;
- calls the P7 validity reducer directly and persists validity-report.json ;
- exposes verdict and claim gates in RunnerResult ;
- runtime version is derived from AgentEvidence rather than AgentNode ;
- offline instrumentation smoke test produces INSTRUMENTATION_ONLY with causal claims blocked.

## 2026-09-30 - P8 Runner evidence lifecycle completed

- added historical-compatible Rust workspace snapshot diff support ;
- Runner imports before/after snapshots and validates the after snapshot against the resolved workspace ;
- added normalized self-contained session-evidence.json without raw prompt copying ;
- added recovery-report.json and safe cleanup of partial Runner-owned artifacts ;
- completed evidence remains immutable and refuses overwrite ;
- full synthetic pipeline now reaches COMPLETE with no pending steps ;
- mismatch and interruption recovery cases validated ;
- P8 remains IN PROGRESS pending the historical Experiment 001 golden replay.

## 2026-09-30 - P8 closed, P9 deterministic replay started

- P8 Runner V0.1 is now DONE from implementation and offline acceptance evidence.
- Historical Experiment 001 whole-run replay is owned by P9 release validation.
- RunnerRequest now accepts optional `session_candidates` for deterministic offline RunGroup reconstruction.
- Normal execution still uses the Codex session index when `session_candidates` is empty.
- Current live B07-C workspace no longer reproduces the historical 13/4/0 snapshot diff (current observation: 11/8/0), so P9 will use a repo-contained immutable golden fixture rather than mutable external workspace state.

## 2026-09-30 - V0.1 Measurement Hardening release gate completed

- normalized workspace Cargo version from 0.0.0 to 0.1.0 ;
- added sanitized repo-contained Experiment 001 Runner golden fixture with explicit provenance ;
- deterministic golden replay preserves accepted aggregate facts without relying on mutable external workspaces ;
- package privacy validator now checks the golden fixture and validation scripts ;
- added documentation consistency validator for canonical active status files ;
- P9 release gates pass: fmt, clippy all-targets, workspace tests, release build, golden replay, package privacy, docs consistency and git diff check ;
- P0-P9 are DONE ; V0.1 Measurement Hardening is complete ;
- Experiment 002 Instrumentation Validation is NEXT.

## 2026-10-01 - Experiment 002 preflight hardening

- versioned the Experiment 002 harness under `scripts/experiment/002/` while keeping real runs/traces outside Git ;
- added a clean-HEAD provisioner with a SHA-256 harness manifest and generated external launchers ;
- release builds now emit `tokn-observe.provenance.json` with source commit, version and binary SHA-256 ;
- START refuses harness drift, dirty Git, binary/provenance mismatch or a changed frozen baseline ;
- START now verifies the complete 833/832/1 baseline before the targeted workflow failure ;
- added run-scoped runtime/model/config observation from diagnostic trace evidence ;
- model evidence can be marked PASS when observed, while configuration completeness remains UNKNOWN until ModelRuntimeProfile is frozen ;
- no real Experiment 002 Codex/Astra run has been launched by this hardening pass.

## 2026-10-01 - Context Pack status de-duplication

- removed stale hard-coded milestone text from the Context Pack manifest generator ;
- generated packs now point to the exact-commit STATUS.md and ROADMAP.md for project state instead of duplicating mutable status.

## 2026-10-01 - GitHub publication hardening

- added GitHub Actions CI for publication privacy, fmt/clippy/tests, release build, Experiment 001 golden replay and documentation consistency ;
- added repository publication privacy validation for common credentials, raw run artifacts and user-home paths ;
- aligned the repository with its declared MIT OR Apache-2.0 license by adding both license texts ;
- hardened .gitignore against local credentials, keys, traces and real experiment run directories ;
- added CODEOWNERS for @ELITEAYOTO and removed the machine-specific build root from README.

## 2026-10-01 - Experiment 002 real-run acceptance and FINISH hardening

- executed the bounded real Experiment 002 task on the frozen B07-C workspace ;
- observed `codex-cli 0.161.0-alpha.2` and `gpt-6.1-sol` from run-scoped evidence ;
- Runner recovery from persisted Codex rollouts produced 2 agents / 49 usage records, terminal COMPLETED, pipeline COMPLETE and INSTRUMENTATION_ONLY ;
- quality gate `npm run verify:local` passed with 863/863 tests ;
- workspace diff was 1 added / 2 modified / 0 removed ;
- causal claims remain blocked and configuration completeness remains UNKNOWN ;
- first FINISH attempt exposed PowerShell LASTEXITCODE handling and nested CLI diagnostic-bundle layout defects ;
- no model rerun was used for recovery: the immutable parent rollout plus descendant were replayed through Runner ;
- added deterministic Experiment 002 source selection with diagnostic-to-session fallback ;
- added CI regression coverage for the fallback path ;
- Measurement Contract Freeze is now the next roadmap phase.

## 2026-10-01 - Measurement Contract V1 freeze

- froze `tokn.measurement.v0.1` after the accepted real Experiment 002 run ;
- Runner now emits `measurement-contract.json` in every evidence folder ;
- versioned Runner request/result, evidence layout, session evidence, RunGroup, token accounting, source health, terminal status, workspace inventory/resolution/snapshot/diff, policy, quality, recovery, validity, ModelRuntimeProfile and analyzer semantics ;
- Runner now rejects unsupported workspace inventory and project snapshot schema versions ;
- froze ordinary uncached input as `input - cached`; cache-write remains a separate metric and is not subtracted from ordinary uncached ;
- added the serializable ModelRuntimeProfile V1 domain contract while leaving persistence for the next phase ;
- Experiment 001 golden replay now asserts every frozen contract version while preserving all historical golden facts ;
- Store + ModelRuntimeProfile persistence is now the next roadmap phase.

## 2026-10-01 - Store V2 foundation and ModelRuntimeProfile persistence

- added additive SQLite Store schema V2 for privacy-preserving projects/workspaces, runs, agents, runtime profiles, provenance and prepared rate-limit snapshots ;
- added `tokn-observe store-evidence` to ingest immutable Runner V1 evidence without changing RunnerRequest V1 ;
- project/workspace/source/profile identities are pseudonymized before persistence ;
- usage totals keep known counters so UNKNOWN coverage is not collapsed into zero ;
- persisted ModelRuntimeProfile V1 JSON with schema validation and idempotent profile identity ;
- legacy import paths are now replaced by private source references ;
- one-time V2 migration pseudonymizes legacy paths and runs VACUUM so removed paths do not remain in free SQLite pages ;
- unknown future Store schema versions are refused fail-closed ;
- Experiment 001 release replay now ingests Store evidence twice and verifies no raw golden workspace/session path is present in the database ;
- Store foundation is DONE; thin command-launched Local MCP integration prototype is NEXT.

## 2026-10-01 - Local stdio MCP transport prototype

- added separate `tokn-mcp` executable as a process-bound read-only stdio MCP server ;
- added `tokn_status` and `tokn_recent_runs` backed by shared Store V2 APIs ;
- centralized the default Observer database path in `tokn-platform` ;
- added read-only Store counts and recent-run query APIs ;
- added structured MCP initialize/ping/tools/list/tools/call handling and errors ;
- added unit tests plus a real release-process stdio smoke ;
- release build now emits independent provenance for `tokn-observe` and `tokn-mcp` while preserving the historical observer provenance file ;
- GitHub CI now runs the MCP process smoke after release build ;
- isolated Codex 0.161.0-alpha.2 registration validation accepted the stdio command server ;
- Codex app-server launched `tokn-mcp`, initialized it and discovered `tokn_status` + `tokn_recent_runs` with `toolsError=null` ;
- target-runtime discovery is reproducible with `scripts/validation/check-codex-mcp-runtime.ps1` and does not mutate normal Codex config ;
- direct `mcpServer/tool/call` through an authenticated Codex thread remains an explicit non-blocking debt; no user auth was copied and no model turn was started for this validation ;
- Local MCP transport prototype is accepted; Historical Analyzer + Context Ledger is NEXT.

## 2026-10-01 - Codex direct MCP host tool-call validation

- strengthened `scripts/validation/check-codex-mcp-runtime.ps1` to use the native Codex app-server interactively instead of wrapper timing heuristics ;
- validation now creates an isolated disposable `CODEX_HOME`, registers only Tokn and discovers the server through `mcpServerStatus/list` ;
- the target Codex runtime creates an ephemeral idle thread with zero turns and directly calls `tokn_status` through `mcpServer/tool/call` ;
- the returned payload confirms `tokn-mcp` 0.1.0, stdio transport, read-only mode and Store schema V2 ;
- no user authentication material is copied or inspected and no model turn is started ;
- the previous direct-host-tool-call validation debt is closed ; Historical Analyzer + Context Ledger remains NEXT.


## 2026-10-01 - Historical Context Ledger V1 and read-only MCP exposure

- added HistoricalSnapshot V1 records shared by Store and Analyzer without changing RunnerRequest V1 ;
- added read-only Store V2 history queries for project/workspace/run/agent/provenance/runtime-profile plus workspace ancestry ;
- added Context Ledger schema V1 with coverage-aware input/cached/cache-write/output/reasoning metrics and integrity issues ;
- ordinary uncached and logical totals remain unavailable unless their required source coverage is complete ;
- per-turn granularity is explicitly NOT_CAPTURED in Measurement Contract V1 and current retained-context occupancy remains UNKNOWN ;
- added `tokn-observe context-ledger` with canonical privacy-preserving project/workspace filters ;
- added read-only MCP tool `tokn_context_ledger` backed by shared Store + Analysis logic ;
- release smoke discovers all three MCP tools and validates Context Ledger boundaries without leaking the database path ;
- target `codex-cli 0.161.0-alpha.2` directly invoked `tokn_status` and `tokn_context_ledger` through app-server on an isolated idle zero-turn thread ;
- full fmt/clippy/tests, release build, MCP smoke, Experiment 001 replay, Experiment 002 regression and documentation checks passed on the validation branch ;
- M4 is IN PROGRESS: tool/file activity, repeated reads/searches/retries, shared/duplicate evidence, compaction/rediscovery and explicit cross-run comparison remain before Findings.

## 2026-10-02 - M4 activity history foundation

- added privacy-safe ToolActivityHistory V1 and ActivityTimeline V1;
- added exact repeated-operation observations without waste/savings claims;
- added activity-timeline CLI;
- added privacy regression for private paths/commands and a runtime-constructed synthetic secret;
- fixed legacy uncached reporting to frozen V1 semantics;
- documented shadow Context Retrieval / Edit Strategy direction and long-term Context Compiler guardrails;
- reviewed external Claude architecture advice against the current repository rather than adopting it blindly.
