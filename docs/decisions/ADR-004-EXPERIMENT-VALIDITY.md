# ADR-004 - Experiment Validity and Causal Claim Gate

Status: ACCEPTED
Date: 2026-09-29

## Context

Tokn can measure token usage even when a run is incomplete,
a source is partial, a workspace was recovered after the fact,
or baseline/candidate controls are not comparable.

Those measurements can still be useful.
They do not automatically support a causal optimization claim.

V0.1 therefore needs a machine-computable validity decision
between evidence collection and any winner/savings conclusion.

## Decision

Tokn introduces a pure Experiment Validity Reducer.

Input is normalized evidence, not prose.
Each required fact is PASS, FAIL, UNKNOWN or NOT_REQUIRED.

The reducer emits exactly one verdict:
- VALID_FOR_CAUSAL_AB;
- VALID_FOR_DESCRIPTIVE_COMPARISON;
- INSTRUMENTATION_ONLY;
- INVALID_CAPTURE;
- INCOMPLETE_TASK.

It also emits:
- causal_claims_allowed;
- descriptive_metrics_allowed;
- structured reason codes.

UNKNOWN never silently becomes PASS.

## Verdict precedence

1. Unusable core capture -> INVALID_CAPTURE.
2. Usable capture + instrumentation intent -> INSTRUMENTATION_ONLY.
3. Required task completion not achieved -> INCOMPLETE_TASK.
4. Descriptive intent with usable capture -> VALID_FOR_DESCRIPTIVE_COMPARISON.
5. Causal intent with every causal requirement satisfied -> VALID_FOR_CAUSAL_AB.
6. Otherwise, usable causal comparison evidence degrades to VALID_FOR_DESCRIPTIVE_COMPARISON.

This precedence intentionally lets an instrumentation run remain useful
when its task did not complete.

## Core capture rule

Core capture is usable only when:
- model usage evidence is PASS;
- RunGroup resolution is PASS;
- source health is HEALTHY,
  or PARTIAL with an explicitly recovered usable fallback.

Malformed, empty, unsupported or unknown source health
cannot support descriptive metrics without recovered evidence.

## Causal requirements

A causal A/B additionally requires:
- tool evidence when required;
- exact task identity;
- task completion when required;
- resolved output workspace;
- before and after state;
- quality gate on the resolved output;
- recorded model/runtime/configuration;
- comparable baseline runtime;
- baseline available;
- same frozen task;
- same starting workspace;
- same runtime/model configuration;
- one primary intervention variable;
- accepted automated/host quality gates;
- sufficient policy evidence when policy is part of the experiment.

A policy violation does not itself invalidate measurement.
Missing or mislabeled policy evidence can invalidate a causal claim.

## Claim gate

Causal language is controlled structurally.

Only VALID_FOR_CAUSAL_AB sets:
causal_claims_allowed = true.

Tokn does not rely on keyword scanning for words such as
"winner", "savings", "gain" or language-specific equivalents.

Descriptive metrics remain visible for:
- VALID_FOR_CAUSAL_AB;
- VALID_FOR_DESCRIPTIVE_COMPARISON;
- INSTRUMENTATION_ONLY;
- INCOMPLETE_TASK.

INVALID_CAPTURE blocks descriptive metrics derived from the invalid capture.

## Experiment 001

Experiment 001 is encoded as instrumentation intent.

Evidence:
- diagnostic source PARTIAL;
- session fallback recovered;
- RunGroup resolved;
- terminal INCOMPLETE_USAGE_LIMIT;
- workspace recovered and verified;
- policy evidence sufficient to describe observation/enforcement;
- no valid causal baseline/intervention.

Expected verdict:
INSTRUMENTATION_ONLY.

Causal claims:
BLOCKED.

## Consequences

P8 can feed the reducer from a self-contained run evidence folder.

P9 can golden-test the reducer offline without Astra quota.

Future analyzers may add evidence dimensions,
but cannot bypass the causal claim gate.

Context Ledger, cache optimization and active interventions
remain outside P7 and outside V0.1 measurement hardening.
