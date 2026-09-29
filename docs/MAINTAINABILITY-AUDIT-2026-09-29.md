# Tokn Maintainability Audit - 2026-09-29

## Verdict

Status: CLEAN / NO MAJOR REFACTOR REQUIRED.

Tokn V0.1 is structurally healthy enough to continue P6-P9 without a broad rewrite.
The architecture is layered, tests are strong, evidence logic is separated from reporting,
and the latest replay still reproduces Experiment 001 after the cleanup.

A small preventive refactor was completed during this audit:
- session-root fallback logic centralized in CLI common helpers;
- duplicated fallback code removed from analyze-run, check-caps and resolve-workspace;
- unused serde_json dependency removed from tokn-analysis.

No behavior regression was observed.
## Validation after cleanup

- cargo fmt: PASS
- cargo clippy --workspace --all-targets -- -D warnings: PASS
- cargo test --workspace: PASS
- Experiment 001 analyze-run replay: PASS
- parent + 3 subagents recovered: PASS
- logical tokens: 5,311,758
- cap policy replay: FAIL as expected, with complete evidence
- workspace resolver: B07-C_WORKING/PROJECT SELECTED

These checks confirm that the cleanup did not alter the measured behavior.
## Architecture health

Internal dependency direction:
- tokn-domain: no internal dependency;
- tokn-ingest -> tokn-domain;
- tokn-platform -> tokn-domain;
- tokn-analysis -> tokn-domain;
- tokn-storage -> tokn-domain;
- tokn-codex -> domain + ingest + platform;
- tokn-report -> analysis + domain + storage;
- tokn-observe composes the crates.

No dependency cycle was observed.
The domain remains independent from Codex, SQLite and CLI concerns.
This is the most important architectural property to preserve.
## Largest Rust modules

Current review points:
- session/code_mode.rs: 429 total lines, about 329 production + 100 tests;
- analysis/workspace.rs: 418 total, about 287 production + 131 tests;
- diagnostic/profile.rs: 364 total, about 329 production + 35 tests;
- analysis/attribution.rs: 346 total, about 284 production + 62 tests.

Decision:
- no split required today;
- code_mode.rs is cohesive: one Code Mode parser plus tests;
- workspace.rs is cohesive: inventory scoring/resolution plus tests;
- diagnostic/profile.rs is the closest future split candidate if it grows;
- attribution.rs remains acceptable.

Soft rule: review a Rust module when production code exceeds about 350 lines.
Hard rule: do not allow a new unrelated responsibility to enter a large module just because it already exists.
## PowerShell runner

finish-exp001-candidate.ps1 is currently about 333 lines.

This is larger than desired, but it is deliberately experiment-specific and P8 is expected
to replace/generalize the runner. Splitting it aggressively now would likely create disposable abstractions.

Decision:
- do not expand this script with unrelated analysis logic;
- P6/P7 analysis belongs in Rust crates;
- refactor/genericize runner behavior during P8;
- if the script grows beyond about 400 lines before P8, stop and extract helpers first.
## Production panic/expect review

Only two production expect() calls were found:
- tokn-analysis/src/attribution.rs;
- tokn-analysis/src/comparison.rs.

Both are used after prior filtering that guarantees usage is present.
They are not an immediate reliability problem.

Rule: new production unwrap/expect/panic calls require an explicit invariant or should return Result/Option.
## Current maintenance gaps

### 1. Git baseline

Resolved on 2026-09-29.

A local Git repository is now initialized in V0-CodexTkn-Consume/tool on branch main.
No remote is configured and no push was performed.

The existing CI workflow is versioned with the source and covers:
- rustfmt;
- Clippy with warnings denied;
- all workspace tests;
- release build.

This gives the project local history, diffs and rollback before P6 grows further.

### 2. Package version is still 0.0.0

Documentation and artifact naming already call this V0.1-dev while Cargo workspace version is 0.0.0.
This is not a code-quality problem, but it can become confusing.

Recommendation: keep 0.0.0 during active hardening if desired, then move to a single explicit version
during P9 release validation, for example 0.1.0 or a documented prerelease.
## Maintainability rules going forward

Before marking a phase DONE:
1. cargo fmt --check passes;
2. Clippy strict passes;
3. all workspace tests pass;
4. affected golden/replay fixtures pass;
5. docs/STATUS/ROADMAP/CHANGELOG are updated when behavior changes;
6. no duplicated source-selection/fallback logic is introduced;
7. UNKNOWN remains UNKNOWN;
8. adapters do not contain optimization conclusions;
9. experiment-specific PowerShell does not become the permanent business-logic layer;
10. new cross-cutting concepts enter the domain first, then adapters/analysis/reporting.

Review thresholds:
- Rust module >350 production lines: review responsibility boundaries;
- PowerShell script >300 lines: review; >400 lines: split before further growth;
- same algorithm copied to a second command: consider shared helper;
- same algorithm copied to a third location: refactor before continuing.

Tests may remain colocated when they explain a parser well; file total line count alone is not a refactor trigger.
## Refactor decisions

Do now:
- centralize session-root fallback: DONE;
- remove unused dependency: DONE;
- keep P6 logic in Rust rather than growing FINISH script: IN EFFECT;
- document Analyzer long-term boundaries: DONE.

Do later:
- generic runner extraction: P8;
- package/version normalization: P9;
- split diagnostic/profile.rs only if new responsibilities are added;
- split Code Mode parser only if additional tool-language parsers appear.

Do not do:
- rewrite crates;
- merge crates to reduce file count;
- create many tiny crates for every feature;
- refactor solely to reduce line counts.
## Final assessment

Tokn is clean enough to continue development.

The project does not currently need a broad refactor.
The correct strategy is targeted preventive refactoring plus strong gates.

Priority maintenance action outside code structure: DONE.
Git history is now available locally before the P6 implementation continues.