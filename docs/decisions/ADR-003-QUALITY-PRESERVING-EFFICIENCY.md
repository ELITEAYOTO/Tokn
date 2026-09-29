# ADR-003 - Optimize context efficiency, not model capability

Status: ACCEPTED
Date: 2026-09-29

## Context

Tokn originally explored output-cap policies because oversized tool reads/searches looked like a possible source of waste.
Experiment 001 and P6 made that behavior measurable, but the product goal is broader:
maximize useful work and quality per consumed token.

The controlled P6 hook investigation also proved that Codex Desktop 0.158.0-alpha.2.1 does not expose
the nested Code Mode `max_output_tokens` field to the PreToolUse Bash callback.

More importantly, reducing Astra output or reasoning capacity is not an acceptable default optimization:
a smaller token count is not a win if it removes useful reasoning, evidence, implementation or verification.

## Decision

Tokn will not use hard output-token caps as its primary optimization strategy.

Output-cap analysis remains available as:
- historical evidence from Experiment 001;
- a diagnostic signal for unusually large tool requests;
- compatibility research for supported runtime surfaces.

The primary optimization direction becomes quality-preserving context efficiency:
- cache-aware context measurement;
- repeated read/search detection;
- duplicate context across parent/subagents;
- instruction and tool-schema overhead;
- tool-output footprint and reuse;
- stable project knowledge candidates;
- cross-run repeated discovery;
- compaction/context-management observation;
- project memory candidates only after evidence.

## Guardrails

Tokn does not delete, truncate, summarize or rewrite context merely to reduce tokens.
Any active context transformation requires a controlled experiment with a quality gate.
Cached tokens, logical tokens, API cost and ChatGPT/Codex product quota are separate metrics.
A high cache ratio does not prove that context is useful or that quota use is efficient.
A low token count does not prove a better run.

## Consequences

P6 can close once policy hint/observation/enforcement are correctly separated and the runtime limitation is documented.
The live zero-row hook audit bug becomes non-blocking diagnostic debt.
P7-P9 remain measurement-hardening work and are not replaced by premature optimization features.
Experiment 002 remains an instrumentation validation run.
The first causal optimization experiment will target a measured context-efficiency finding, not an arbitrary output cap.
