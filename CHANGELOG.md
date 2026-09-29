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
