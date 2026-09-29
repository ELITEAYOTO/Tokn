# Policy Model - Hint, Observation, Enforcement

Status: STABLE V0.1 DESIGN
Updated: 2026-09-29

## Purpose

Policy evidence answers three different questions:
1. Was guidance available?
2. Did observed behavior comply?
3. Did a runtime mechanism enforce it?

These questions must never be collapsed into one PASS/FAIL flag.

## policy_hint

A model-facing instruction exists or was placed.

Evidence may include:
- AGENTS.md placement;
- prompt/instruction marker;
- session instruction evidence;
- explicit repository read.

A hint can be ignored.
Presence is not enforcement.

## policy_observed

Actual observable requests satisfy or violate a rule.

States:
- PASS;
- FAIL;
- NO_EVIDENCE;
- INCOMPLETE_EVIDENCE;
- NOT_APPLICABLE.

PASS requires:
- at least one applicable target;
- no violation;
- no unknown required field.

Experiment 001:
58 targets / 17 violations / 0 unknown => FAIL.

## policy_enforced

A runtime mechanism prevents non-compliant requests from reaching the protected surface.

Capability states:
- UNAVAILABLE;
- SUPPORTED_UNVERIFIED;
- SUPPORTED_INSUFFICIENT_INPUT;
- SUPPORTED_NOT_ACTIVE;
- NOT_PROVEN;
- ENFORCED.

ENFORCED requires run-scoped proof, not documentation alone.

## P6 result

Codex Desktop 0.158.0-alpha.2.1:
- hooks are supported;
- trusted/active hook lifecycle was observed;
- nested unified exec maps to PreToolUse/Bash;
- the callback receives command but not max_output_tokens.

Therefore the historical output-cap policy is:
`SUPPORTED_INSUFFICIENT_INPUT`.

This is a runtime contract limitation, not a reason to patch Codex.

## Product scope

Output-cap policy remains useful for:
- diagnostics;
- historical Experiment 001 replay;
- adapter compatibility tests.

It is NOT the primary Tokn optimization strategy.

ADR-003 moves product optimization toward quality-preserving context efficiency.

## Placement

Experimental policy placement must:
- touch only declared roots;
- preserve original content;
- record exact installed hash;
- restore independently;
- fail conservatively on unexpected edits.

A dynamically copied workspace may be cleaned only with exact identity proof.

## Reporting rule

Every relevant experiment report states separately:
- policy_hint;
- policy_observed;
- policy_enforced.

If enforcement is not needed by the experiment,
the report may record capability status without activating a hook.

## Reopen conditions

Hard-enforcement research is reopened only if:
- a future experiment genuinely requires it;
- a supported runtime surface exposes the required policy inputs;
- the intervention does not conflict with quality-preserving strategy.

Until then P6 is closed.
