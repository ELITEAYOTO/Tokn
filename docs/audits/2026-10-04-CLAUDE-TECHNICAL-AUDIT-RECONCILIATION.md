# Tokn technical audit reconciliation - 2026-10-04

Status: **VERIFIED AGAINST CURRENT MAIN / DOCUMENTATION-ONLY / NO REMEDIATION APPLIED**

## Scope

External source reviewed:
- owner-supplied Claude audit `AUDIT-TOKN-2026-10-04.md`;
- audited repository state: `c9d49c2` (around PR #20);
- audit type: static AI-assisted review, explicitly non-certifying.

Current repository baseline used for this reconciliation:
- repository: `ELITEAYOTO/Tokn`;
- current main: `e554c2e557570c858ff8d9dce498be0b82df6169`;
- PR #22 merged and post-merge CI #72 PASS;
- worktree clean and local `main == origin/main` at reconciliation start;
- M4.5 sanitized Shadow Index pilot accepted as a measurement harness, while `SQLITE_FTS5_UNICODE61_V0` is REJECTED for the frozen sanitized quality condition;
- no persistent FTS backend, active context injection, or token-savings claim exists.

This document reconciles the external audit with current code, local tooling, GitHub state and current canonical documentation. It is a backlog/evidence document, not an implementation decision.

Status vocabulary:
- `CONFIRMED`: current code/config directly reproduces the gap;
- `PARTIAL`: part of the finding is resolved or newer work narrowed it, but material debt remains;
- `SUPERSEDED`: current code/evidence invalidates the old claim;
- `PROSPECTIVE`: not a current vulnerability; it becomes relevant only if the planned feature is built;
- `SETTING_NOT_VERIFIED`: repository code supports the finding, but an account/repository security setting could not be authoritatively read.

## Phase-1 closure before audit review

The interrupted M4.5 slice was completed before this review:
- frozen sanitized Git corpus + 8 gold queries + K={1,5,10} + 5 timing repetitions;
- predeclared 0.95 Recall@5/MRR ratios;
- DIRECT_SCAN baseline: Recall@5=1.0, MRR=1.0 on the fixture;
- unicode61 candidate: Recall@5 ~= 0.786, MRR ~= 0.857;
- unicode61 quality gate `FAIL`, verdict `REJECTED` for this measured condition;
- PR #22 exact-SHA CI PASS, merge PASS, post-merge CI #72 PASS.

This is useful retrieval-quality evidence, but it is not evidence of user-visible token savings or product value.

## Local development environment - verified 2026-10-04

| Component | Local state | Current assessment |
| --- | --- | --- |
| Rust project toolchain | 1.97.1, pinned by `rust-toolchain.toml` | Validated and aligned with CI; one stable release behind 1.98.1, not an urgent bump. |
| rustup | 1.29.1 | Installed; no global default toolchain configured. Repo-local override works. |
| rustfmt / Clippy | installed for 1.97.1 | Required and working. |
| Visual Studio Build Tools 2022 | 17.14.27, complete VC x64 toolchain | Working; behind current 17.14 servicing release. |
| Windows PowerShell | 5.1 | Present, but not sufficient to reproduce every CI PowerShell behavior. |
| PowerShell 7 / `pwsh` | missing | Recommended local parity tool. |
| Git for Windows | 2.51.0 | Working; newer maintained release exists. |
| GitHub CLI | 2.98.0 | Working; newer release exists. |
| `cargo-audit` | 0.22.2 | Installed; current `Cargo.lock` audit PASS, 96 dependencies scanned. |
| `cargo-deny` | missing | Recommended supply-chain gate. |
| `cargo-mutants` | missing | Recommended after first security/integrity wave. |
| `cargo-llvm-cov` | missing | Recommended for coverage reporting. |
| `cargo-fuzz` | missing | Later hostile-parser work; Windows/MSVC is not the first environment to optimize for it. |
| gitleaks | missing | Recommended for full-history secret scanning. |
| Python | 3.11.9 | Not a Tokn build blocker. |
| Node.js | 22.15.1 | Not a Tokn build blocker. |
| CMake | 4.3.4 | Not currently on Tokn's critical build path. |
| Ninja | 1.13.2 | Present; not currently required by the validated build path. |
| clang | not on PATH | Not required by the current MSVC build; relevant later for some fuzzing workflows. |

Local Git is now configured with a GitHub `noreply` author email for future commits. Historical public commit metadata is a separate already-published issue and is not rewritten by this review.

## Finding reconciliation - security and privacy

| ID | Status | Current-main evidence / correction |
| --- | --- | --- |
| S-01 pseudonymization | CONFIRMED | `private_id` still uses unkeyed BLAKE3 and project identity remains dictionary-confirmable if a database is obtained. Current wording should be understood as local pseudonymization, not cryptographic protection against an adversary holding the DB. |
| S-02 MCP read-only claim | CONFIRMED | `tokn-mcp` still calls `Database::open`, which may create/migrate/VACUUM. Read-only describes exposed tools, not the SQLite connection. |
| S-03 PreToolUse hook | CONFIRMED | Parse/audit errors propagate, audit JSONL uses multiple writes, and a deny can depend on heuristic command classification. The historical runtime still does not provide the cap needed for the intended enforcement. |
| S-04 quality-gate process | CONFIRMED | `ProcessCommand::new(...).output()` remains unbounded by timeout/output and inherits the environment; stdout/stderr are persisted in quality evidence. |
| S-05 publication guard | CONFIRMED | HEAD-only scan, fail-open read catch, Windows-centric path pattern and `scripts/rename_tokn.py` remain. No full-history secret scanner is configured. |
| S-06 public Git metadata | PARTIAL | Local Git now uses GitHub `noreply`, so future local commits are aligned. Historical public identity/email metadata remains; history rewrite would break documented SHAs and is not recommended casually. |
| S-07 Git subprocess hardening | CONFIRMED | DIRECT_SCAN still resolves `git` through PATH without a centralized minimal environment/config isolation/timeout wrapper. |
| S-08 future HTTP MCP / persistent index | PROSPECTIVE | No remote MCP and no persistent FTS backend exist today. Threat-model and sensitive-index gates must precede either feature. |
| S-09 vulnerability disclosure | CONFIRMED / SETTING_NOT_VERIFIED | `SECURITY.md` has no concrete supported-version/contact/response policy. Private vulnerability-reporting account setting was not authoritatively readable through the current connector. |
| S-10 data at rest / retention | CONFIRMED | No purge/export/retention policy exists; Store directory creation still discards `create_dir_all` errors; no optional at-rest encryption/secure-delete contract exists. |

## Finding reconciliation - reliability and storage

| ID | Status | Current-main evidence / correction |
| --- | --- | --- |
| R-01 truncated JSONL loop | **CONFIRMED / HIGH** | `bounded_jsonl` can reach EOF while `offset < snapshot_end`, break without advancing offset, then re-enter the outer loop. This deserves immediate deterministic regression coverage. |
| R-02 memory bounds | CONFIRMED | 32 MiB JSON records deserialize into `Value`; several JSON files use unbounded `std::fs::read`; tool-result outputs are accumulated before projection; DIRECT_SCAN has no total-byte cap. |
| R-03 evidence writes | CONFIRMED | Runner `write_json` is direct `File::create + to_writer_pretty`, without temp+rename/fsync/lock. |
| R-04 Runner orchestration | CONFIRMED | `runner.rs` remains large and contains multiple production-path `.expect(...)` invariants tied to optional artifacts. |
| R-05 DIRECT_SCAN active-use behavior | CONFIRMED / REFERENCE-ONLY TODAY | Strict whole-manifest revalidation is appropriate for the benchmark reference but would be expensive/brittle for an always-changing active repository. No active retrieval path exists yet. |
| R-06 MCP robustness | CONFIRMED / DETAIL PARTLY SUPERSEDED | Protocol version has advanced since the audit, but unbounded `read_line`, no batch path, pretty JSON, weak unknown-argument enforcement and limited diagnostics remain. |
| D-01 SQLite foreign keys | **CONFIRMED / HIGH** | Normal `Database::open` does not enable `PRAGMA foreign_keys`, `busy_timeout`, WAL or `trusted_schema=OFF`; migration SQL alone cannot enable FK enforcement for later connections. |
| D-02 migrations | CONFIRMED | Multiple hand-managed schema-version keys, non-transactional `execute_batch`, `VACUUM` during open, generic `InvalidQuery` for future versions and no full migration matrix remain. |
| D-03 legacy Store | CONFIRMED | Legacy `runs/sources`, `INSERT OR REPLACE` and `latest_run ORDER BY created_at_unix DESC` still coexist with V2 state. |
| D-04 domain constraints | CONFIRMED | Store tables remain non-STRICT with limited/no CHECK constraints for domain invariants. |
| D-05 historical query scaling | CONFIRMED | Historical readers remain bounded mainly by run-count limits rather than cursor/byte budgets and include N+1-style loading paths. |

## Finding reconciliation - code quality and tests

| ID | Status | Current-main evidence / correction |
| --- | --- | --- |
| Q-01 monoliths / maintainability | **CONFIRMED** | `measurement.rs` is now about 105 KiB / 2555 lines; `cross_run_comparison.rs` ~1030 lines; `STATUS.md` still cites the 2026-09-29 maintainability audit as CLEAN. The old verdict is stale. |
| Q-02 stringly typed domain | CONFIRMED | Core history/observation structs still expose string `kind/category/surface/status` fields with literal comparisons/parsers. |
| Q-03 command classifier | **CONFIRMED** | `classify_command_text` still uses ordered substring checks and its result can feed hook deny behavior and future M5 statistics. |
| Q-04 path normalization | CONFIRMED | Multiple path normalization rules remain and Windows/Linux case semantics are not unified. |
| Q-05 JSON read duplication | CONFIRMED | BOM stripping + `serde_json::from_slice` remains repeated in Runner and Store ingestion. |
| Q-06 error model | CONFIRMED | Library error strategy remains mixed; production `expect` and generic rusqlite errors remain. |
| Q-07 dead/legacy code | CONFIRMED | `scripts/rename_tokn.py` is still tracked; logging remains uneven across binaries. |
| Q-08 lints/release profile | CONFIRMED | `clippy.toml` only sets MSRV; no workspace lint policy or custom release profile is defined. |
| T-01 coverage/mutation/property/fuzz | **CONFIRMED** | No `cargo-llvm-cov`, `cargo-mutants`, `proptest` or fuzz harness is configured. Only `cargo-audit` is installed locally among the reviewed quality/supply-chain Cargo tools. |
| T-02 specific test gaps | PARTIAL | New M4.5/direct-scan tests were added after the audit, so the list is not exact anymore. Important gaps still remain around JSONL truncation, DB FK/concurrency/migrations, hook concurrency, MCP giant/batch/unknown args and classifier false-positive corpus. |
| T-03 test temp hygiene | CONFIRMED | 45 current Rust occurrences of `process::id()` remain; RAII temp patterns exist in newer code but are not generalized. |

## Finding reconciliation - CI, supply chain and repository governance

| ID | Status | Current-main evidence / correction |
| --- | --- | --- |
| C-01 dependency audit in CI | **CONFIRMED** | CI has no `cargo audit` or `cargo deny`; current manual `cargo audit` PASS on 2026-10-04 scanned 96 dependencies. |
| C-02 mutable Actions refs | CONFIRMED | `checkout@v4`, `rust-toolchain@stable`, `rust-cache@v2`; no SHA pinning, `persist-credentials:false` or concurrency group. |
| C-03 Windows-only CI | CONFIRMED | One `windows-latest` job; no Linux matrix, no `cargo doc -D warnings`, no uploaded release artifact. |
| C-04 release trust | CONFIRMED | Hash/provenance exist, but no SBOM, signed attestation, signing, remap-path-prefix or reproducible package guarantee. |
| C-05 package mismatch | **CONFIRMED** | `package.ps1` conditionally reads sibling directories absent in CI and omits `tokn-mcp.exe`; local and CI package shapes can differ. |
| C-06 branch governance | **CONFIRMED** | GitHub reports `main` unprotected and no repository rulesets. Historical PR #3 and #4 were merged although their PR workflows concluded `failure`. Recent work follows a stricter manual exact-SHA discipline, but GitHub does not enforce it. |

Repository cleanup observations:
- no open PR remains after #22;
- `main` is clean and synchronized locally/remotely;
- several historical M4 branches still exist locally and/or remotely after their work was merged/superseded;
- no Git tags/releases exist yet, which is acceptable before a public distribution milestone;
- `delete_branch_on_merge` is disabled, helping explain remote branch accumulation.

No branch deletion is performed by this reconciliation because squash merges make simple ancestry checks insufficient for every historical branch. Cleanup should be a separate deliberate operation.

## Finding reconciliation - documentation, product and portability

| ID | Status | Current-main evidence / correction |
| --- | --- | --- |
| DOC-01 documentation drift | **CONFIRMED** | Baseline `e554c2e` had a ~66 KiB `CHANGELOG.md`, ~22 KiB `STATUS.md`, localized mojibake and repeated historical detail. This documentation-only checkpoint corrects the observed mojibake/status wording but does not solve the structural documentation growth. |
| DOC-02 open-project docs | CONFIRMED | No `CONTRIBUTING.md`, `CODE_OF_CONDUCT.md`, `SUPPORT.md`, threat-model document, `deny.toml` or Dependabot config exists. |
| P-01 user-visible value | **CONFIRMED STRATEGICALLY / WORDING PARTLY SUPERSEDED** | The repository has now reached PR #22 and added a real Shadow retrieval benchmark, so it is no longer true that every new slice is only contract design. However no user-visible token reduction, addressable-waste estimate or reproducible product gain is demonstrated; the first measured unicode61 candidate was rejected on quality. |
| P-02 small sample / runtime alpha | CONFIRMED | Real evidence is still dominated by two accepted historical runs and Codex alpha-era fixtures. Rust code still has no `apply_patch` mutation recognition. |
| P-03 fail-closed usability | CONFIRMED | Evidence gaps are explicit, but there is no aggregate coverage/`doctor` view showing why most observations become UNKNOWN/NOT_CAPTURED. |
| X-01 portability | CONFIRMED | Validated build/CI path remains Windows x64/MSVC + PowerShell. Multi-runtime architecture does not yet equal multi-OS validation. |

## Baseline documentation inconsistencies corrected by this documentation-only checkpoint

1. `docs/plans/NEXT-SESSION-CHECKLIST.md` was dated 2026-10-03 and still said to begin M4.5 although PRs #17-#22 had already delivered the first Shadow design/reference/candidate/pilot slices; it is now moved to the audit decision boundary.
2. `STATUS.md` still said `Maintainability audit : CLEAN / NO MAJOR REFACTOR REQUIRED` from 2026-09-29 although the largest Store module has grown to ~2555 lines; it now points at this current reconciliation.
3. `SECURITY.md` and `STATUS.md` recorded a 2026-10-03 RustSec audit over 95 dependencies; this checkpoint records the fresh 2026-10-04 PASS over 96 dependencies.
4. The docs described durable identities as `privacy-safe` without an adversarial-store qualification; `SECURITY.md` now explicitly says project-scoped pseudonymization is not cryptographic secrecy against dictionary confirmation.
5. MCP documentation used `read-only` without distinguishing tool surface from storage-open behavior; `SECURITY.md` now records that the current DB open path can still migrate/VACUUM until a true read-only DB path exists.

## Product/strategy assessment

The strongest audit recommendation is P-01: stop allowing measurement foundations to become an unbounded success criterion.

Current evidence strengthens that recommendation rather than refuting it:
- the project now has enough observation primitives to start producing a useful one-page diagnostic;
- Experiment 001 already shows descendants at ~52.9% of logical tokens;
- exact repeated operations, source identities, cross-agent overlap and historical ledgers now exist;
- the Shadow pilot proved that a plausible implementation can fail a predeclared quality gate, which is exactly why value/quality evidence should decide the next investment.

Recommended strategic checkpoint before more Shadow backend work:
1. freeze new M4.5 backend/foundation expansion temporarily;
2. fix the smallest high-risk integrity/governance issues first;
3. run a bounded **Value Spike V0** using existing evidence;
4. collect at least 10 native baseline runs across a small frozen task set;
5. report uncached tokens by agent/run, directly observed repeated-source/operation overlap, evidence coverage and a clearly labeled upper bound of addressable duplication;
6. define an explicit stop/reorient criterion before building more retrieval machinery.

The audit suggested 10% addressable upper-bound savings as one stop criterion. That number is a proposal, not current Tokn evidence, and should be accepted or changed explicitly before implementation.

## Recommended remediation order - reconciled

### Wave A - immediate integrity/process, low conceptual risk

1. `R-01`: fix JSONL source-shrink loop + deterministic regression.
2. `D-01`: enable FK enforcement + timeout first; evaluate WAL/trusted-schema together with privacy tests instead of blindly changing all PRAGMAs at once.
3. `C-06`: protect `main` with required PR + required CI, no force-push; enable delete-branch-on-merge only after historical branch review.
4. `C-01/C-02`: add `cargo audit` + `cargo deny`, Dependabot, immutable Action SHAs/concurrency/credential hardening.
5. `S-05`: fail-closed publication scan + multi-OS path patterns + full-history secret scanner; remove the one-off rename script after confirming no operational dependency.

### Wave B - make guarantees match reality

6. `S-02`: true read-only SQLite open path for MCP; migrations explicit outside MCP.
7. `S-04`: quality-gate timeout/output bound/minimal env/redaction.
8. `Q-03/S-03`: replace substring command classification before allowing it to deny production operations; keep hook fail-open.
9. `R-06`: bounded/compact MCP responses, strict argument validation and visible tool errors.
10. `S-01`: reword current privacy guarantee now; design keyed installation identity migration before Store export/sharing/public distribution.

### Wave C - prove value before more foundations

11. `P-01/P-02/P-03`: Value Spike + evidence-coverage report + baseline corpus.
12. Only continue trigram/persistent Shadow work if the spike identifies retrieval/rediscovery as an addressable surface worth the complexity.

### Wave D - maintainability/public distribution

13. split `measurement.rs`/large modules, add domain enums and shared bounded JSON/path/process helpers;
14. coverage/mutation/property/fuzzing;
15. threat model, retention/purge/export, SBOM, signing/attestation, Linux CI/xtask and packaging convergence.

## Tooling changes proposed, not executed

High value for the next implementation wave:
- install PowerShell 7 (`pwsh`) for local CI parity;
- install `cargo-deny`;
- install gitleaks (or equivalent full-history secret scanner);
- later add `cargo-mutants` and `cargo-llvm-cov`;
- keep the repo pinned to Rust 1.97.1 until a dedicated toolchain-bump PR updates local + CI + MSRV together;
- update Visual Studio Build Tools 17.14 servicing release and Git/GitHub CLI as maintenance, one tool at a time with a full Tokn gate after toolchain-critical updates;
- defer `cargo-fuzz`/LLVM setup until the hostile-parser wave, preferably with a Linux CI lane rather than expanding the Windows MSVC environment first.

No tool was installed or upgraded during this reconciliation.

## Decision boundary

No remediation from the external audit is implemented in this document.
No GitHub ruleset/settings mutation, branch deletion, dependency addition, migration change, Rust refactor or tool installation is performed.

The next action requires explicit owner approval of the remediation strategy and first implementation wave.
