# Revue du handoff Claude - 2026-10-02

Statut : VERIFIED AGAINST CURRENT REPOSITORY / ROADMAP INPUT

## Scope

Source reviewed:
`TOKN-HANDOFF-POUR-CHATGPT.md`, produced from Context Pack 008 / commit
`87d3bbcb1f4737aaae6272fc471a073839e14afb`.

The handoff is useful research input, not a source of truth.
The current repository, tests, frozen contracts and direct runtime evidence take precedence.

Current baseline used for this review:
- main: `03339fa` before the current M4 activity-history slice ;
- Measurement Contract V1: FROZEN ;
- Store V2: accepted ;
- local stdio MCP transport: accepted ;
- HistoricalSnapshot / Context Ledger V1: accepted ;
- current M4 branch: privacy-safe tool activity history and Activity Timeline V1.

Status vocabulary in this review:
- CONFIRMED: gap or recommendation is still real ;
- PARTIAL: important parts already exist, remaining gap is narrower ;
- REFUTED: current code/evidence contradicts the claim ;
- SUPERSEDED: once valid, but project state has moved beyond it ;
- NOT_VERIFIABLE: no authoritative local/current evidence yet.

## G-series gap review

| ID | Status | Current evidence / correction | Decision |
| --- | --- | --- | --- |
| G-01 cache profitability | PARTIAL | Cache-write, cached and uncached accounting already exist in Domain, Analyzer, Store and Context Ledger. No write-reuse ratio exists. Frozen V1 deliberately keeps cache-write separate. | Add reuse observations only after semantics are directly evidenced. Do not reopen V1. |
| G-02 Tokn observer overhead | CONFIRMED | ADR-003 mentions instruction/tool-schema overhead generally, but there is no experiment measuring Tokn's own injected context/tool-schema cost. | Add an overhead experiment before default context-injected/plugin or active optimizer behavior. Offline analysis is not required to "pay for itself" in tokens. |
| G-03 public/enterprise threat model | PARTIAL | SECURITY/PRIVACY, publication privacy CI, pseudonymized Store IDs and SQLite no-raw-path tests already exist. No compact threat model, dependency audit, SBOM or retention/purge policy yet. | Pre-public-binary hardening gate; do not block observation-only M4. |
| G-04 run variance | CONFIRMED | No variance/stddev/confidence model exists. | Required before Experiment 003. Pilot repeated runs first, then choose N/effect threshold from measured noise; do not hard-code N=5 as universally sufficient. |
| G-05 useful work | PARTIAL | Runner has a versioned QualityGate and causal validity requires quality evidence. It is currently mostly external command exit status, not rework/diff-retention metrics. | Add multi-dimensional quality evidence before causal optimization. Do not collapse quality into one magic score. |
| G-06 runtime confounders | PARTIAL | ModelRuntimeProfile already records runtime/model/provider/effort/context-management/config/capabilities. Causal validity already requires comparable runtime/model/config, but the PASS can still come from Runner hints rather than an automatic profile comparison. | Add an automatic compatibility reducer in M4 cross-run comparison; missing evidence => UNKNOWN. |
| G-07 anatomy of a turn | PARTIAL / IN PROGRESS | Context Ledger V1 correctly marks per-turn usage NOT_CAPTURED. Current M4 branch adds privacy-safe tool activity and per-agent ordering. | Build evidence-bounded activity anatomy; never backfill invented per-turn token accounting into V1. |
| G-08 unstable Codex format | PARTIAL | JSONL ingest is bounded/streaming, uses serde_json::Value, preserves provenance and fails soft on unknown/malformed evidence. Versioned fixtures/capability matrix exist. | Add schema-drift reporting later; not a blocker for current M4. |
| G-09 retention/purge | CONFIRMED | Local-first privacy exists, but no Tokn Store retention duration, purge/export command or admin policy exists. | Required before public/enterprise binary distribution. |
| G-10 ChatGPT web | OUT OF SCOPE | No change needed. | Keep out of current roadmap. |
| G-11 performance/volume | PARTIAL | JSONL parser is streaming, 256 KiB buffered and caps a record at 32 MiB with oversized/malformed/truncated accounting. No incremental import offset or large-rollout benchmark. | Add benchmark + incremental ingestion when real history volume justifies it. |
| G-12 hooks fail-open | PARTIAL | Historical hook leaves missing max_output_tokens unblocked. Parsing/audit I/O failures can still return process errors. Hooks are not a production optimization path today. | Make explicit fail-open boundary before any production hook is shipped. |
| G-13 multi-OS | CONFIRMED AS FUTURE | Current target and CI are Windows x64 only. | Keep Core/adapters portable; add Linux/macOS CI after stable Codex V1/runtime abstraction, not during M4. |
| G-14 distribution trust | PARTIAL | Release SHA-256/provenance and MIT/Apache licensing already exist. No code signing, SBOM or dependency-audit gate yet. | Add before public binary release. |
| G-15 user-visible value | PARTIAL / SUPERSEDED | `tokn doctor` already exists and reports platform, Codex installs/versions/capabilities and session root. A basic report exists; rich "where tokens went / top evidence" UX is not built. | Improve report after reliable M4/M5 findings. |
| G-16 overdocumentation | PARTIAL | STATUS, ROADMAP, INDEX and exact-commit Context Packs already act as canonical sources. | Prefer updating canonical docs; archive later if needed. Do not auto-generate away the curated rationale in CHANGELOG. |
| G-17 license/governance | SUPERSEDED | `Cargo.toml` already declares MIT OR Apache-2.0 and both license files exist. | CONTRIBUTING can wait for external contributors. |
| G-18 multi-provider token semantics | CONFIRMED / CRITICAL | OpenAI V1 semantics cannot be reused blindly for Anthropic. Anthropic documents input, cache-creation and cache-read as distinct usage components. | Design provider-neutral Token Semantics V2 before runtime #2. Do not mutate frozen V1. |
| G-19 Cowork adapter | NOT_VERIFIABLE LOCALLY / PLANNED | No `%APPDATA%\Claude`, Cowork session root or `%USERPROFILE%\.claude` exists on the development PC. Third-party filesystem claims are therefore unverified here. | Runtime #2 remains import-only/read-only first, but only after obtaining real sanitized Cowork/Claude evidence. |
| G-20 freeze new fields | SUPERSEDED | Measurement Contract V1 is already frozen and released. | New semantics/overhead/provider fields go into additive contracts/V2, never by silently changing V1. |

## R-series research review

| ID | Status | Current result |
| --- | --- | --- |
| R-01 cache-write inclusion | PARTIAL / UNKNOWN on real corpus | V1 tests enforce cached<=input and cache_write<=input and keep writes separate. A direct private-session probe was not executed; use explicit sanitized evidence before claiming exact rollout semantics. |
| R-02 retained context occupancy | RESOLVED FOR V1 AS UNKNOWN | Context Ledger V1 explicitly reports retained-context occupancy UNKNOWN; total usage/context window are not relabeled as occupancy. |
| R-03 Codex prefix construction | OPEN | Useful source-code/runtime research, but not needed to finish M4 activity history. |
| R-04 MCP/skill schema overhead | OPEN / G-02 | Measure before default injected integration. |
| R-05 deferred tool loading | OPEN | Do not infer from API behavior alone; verify exact Codex host/runtime behavior. |
| R-06 hook output token cost | OPEN | Needs a controlled runtime experiment. |
| R-07 Astra context-management events | OPEN | Needs a controlled run with the feature state recorded. |
| R-08 effort change vs cache | OPEN | Needs controlled run; do not label "cache break" as fact yet. |
| R-09 Cowork real format | BLOCKED BY EVIDENCE | No Cowork/Claude local sessions available on this PC. |
| R-10 Cowork hooks | BLOCKED BY EVIDENCE | Same reason; third-party issue reports are not enough for a Tokn capability verdict. |
| R-11 Codex plugin manifest | VERIFIED CURRENT DOCS | Portable package: root `plugin.json`; Codex compatibility/native layout: `.codex-plugin/plugin.json`. Both are current supported shapes with different roles. |
| R-12 run variance | OPEN / G-04 | Required before causal optimization. |
| R-13 parser robustness | PARTIAL | Bounded streaming parser + malformed/oversized/truncated-tail tests exist. Fuzz/deep hostile corpus remains future hardening. |
| R-14 secret detection | PARTIAL + IMPROVED | Publication privacy and SQLite path/raw-evidence tests already exist. Current M4 slice adds a synthetic fake-secret persistence regression. Broader fake JWT/API-key corpus remains useful. |
| R-15 code-search behavior | IN PROGRESS | Current M4 ToolActivityHistory/ActivityTimeline is the first evidence-bounded foundation: categories, per-agent ordering and exact repeated-operation observations. |
| R-16 rate limits | PARTIAL | `rate_limit_snapshots_v2` schema exists; no active ingestion path was found. Keep distinct window/limit identities when implemented. |

## Accepted roadmap insertions

### Finish M4 before Findings

M4 should finish:
1. privacy-safe ToolActivityHistory + ActivityTimeline ;
2. exact repeated-operation observations without calling them waste ;
3. automatic ModelRuntimeProfile compatibility reducer ;
4. rate-limit snapshot ingestion with identity/window provenance ;
5. shared/duplicate evidence and rediscovery observations ;
6. compaction/context-management observations only when directly evidenced ;
7. explicit cross-run comparison primitives.

### Gates before Experiment 003

Before the first causal optimization A/B:
1. pilot run-to-run variance ;
2. predeclared multi-dimensional quality acceptance ;
3. automatic runtime/model/config compatibility PASS ;
4. one primary intervention ;
5. repeated finding with direct provenance ;
6. overhead measurement for any Tokn component injected into model context or default active execution.

### Pre-public-binary hardening

Before distributing a general-purpose signed binary/plugin:
- compact threat model ;
- retention/purge/export policy ;
- dependency vulnerability audit ;
- SBOM ;
- Windows signing/update integrity ;
- hostile parser/secret-redaction corpus ;
- production hooks fail-open with tight timeout/no network dependency.

These items do not block the current local observation-only analyzer.

### Runtime #2

Do not implement Cowork/Claude by copying OpenAI accounting.
Order:
1. finish stable Codex V1 observation path ;
2. define provider-neutral token semantics V2 ;
3. obtain real local Cowork/Claude sessions and sanitize fixtures ;
4. build import-only adapter ;
5. map capabilities as OBSERVED / UNSUPPORTED / UNKNOWN per runtime/version ;
6. only then expose runtime-specific MCP/plugin integration.

## Recommendations from the handoff that were modified or rejected

- Do not reopen Measurement Contract V1: it is already frozen.
- Do not require the offline profiler itself to save more tokens than it consumes; overhead gating applies to injected/default-active behavior.
- Do not hard-code N>=5 as a universal A/B sample size. Measure variance first.
- Do not create a single "useful work score". Preserve several quality dimensions.
- Do not invent per-turn token usage from run/agent aggregates.
- Do not implement a Cowork filesystem adapter from unverified third-party paths.
- Do not call exact repeated reads "waste" until frequency/cost/context and quality evidence support a Finding.
- Do not force Linux/macOS production support into the current Windows-first M4 slice.

## Official-source checks performed

OpenAI:
- Prompt caching: https://developers.openai.com/api/docs/guides/prompt-caching
- Deployment checklist: https://developers.openai.com/api/docs/guides/deployment-checklist
- Portable plugin packaging: https://developers.openai.com/plugins/build/plugins
- Codex/plugin API layout: https://developers.openai.com/api/docs/guides/agents-api/tools/plugins

Anthropic:
- Pricing / cache accounting: https://docs.anthropic.com/en/docs/about-claude/pricing
- Claude Code SDK / stream-json sessions: https://docs.anthropic.com/en/docs/claude-code/sdk
- Claude Code data usage / local session retention: https://docs.anthropic.com/en/docs/claude-code/data-usage

## Bottom line

The Claude handoff contains several high-value ideas, especially:
- observer overhead ;
- automatic runtime compatibility ;
- variance before causal A/B ;
- threat-model/public-release hardening ;
- retention/purge ;
- provider-specific token semantics ;
- activity anatomy and exact repetition evidence.

Its main weakness is age/context: it was written from documentation before Store V2,
the accepted MCP path, Experiment 002 closure and the current Historical Analyzer implementation.
Several gaps it reports are already partially solved or fully superseded.

The safe strategy is therefore not to replace the Tokn roadmap with this audit.
Use it as a verified backlog input while preserving:
observation -> evidence -> history -> Findings -> causal experiment -> Advisor -> active optimization.
