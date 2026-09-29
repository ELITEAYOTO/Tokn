# Experiment 002 - Instrumentation Validation

Status: PLANNED / BLOCKED BY V0.1 P7-P9
Updated: 2026-09-29

## Purpose

Validate that Tokn V0.1 measures a real Astra run correctly end to end.

Experiment 002 does NOT attempt to prove token savings.

It validates:
- session/diagnostic fallback;
- parent/subagent grouping;
- terminal status;
- tool/policy evidence;
- dynamic workspace tracking;
- correct quality-gate target;
- experiment-validity reducer.

## Task selection

Use a genuine but bounded JEM development or maintenance task.

Requirements:
- useful real work;
- small enough that quota exhaustion is unlikely;
- clear automated acceptance gate;
- preferably one known candidate output workspace;
- no broad multi-phase architecture migration.

Do not ask Astra to be faster, shorter or less thorough.
## Required preflight

Before launch:
- V0.1 tests PASS;
- no active run;
- runtime/model versions recorded;
- source workspace snapshot complete;
- expected output workspace recorded when known;
- policy placement paths recorded;
- session root selection confirmed;
- fallback discovery enabled.

## Success conditions

Capture:
- root session discovered;
- every descendant subagent discovered;
- token totals available;
- diagnostic/session source health visible;
- terminal status known.

Workspace:
- actual changed workspace identified;
- correct before/after diff;
- quality gate runs on the actual output.

Policy/evidence:
- policy level accurately reported when a policy is present;
- zero observations cannot pass;
- no hard-cap enforcement is required for this instrumentation run.

Experiment:
- machine-readable validity verdict generated.
## Failure conditions

Repeat Experiment 002 after fixing Tokn if:
- source fallback needs manual repair;
- descendant sessions are missed;
- output workspace is wrong;
- quality gate runs on the wrong root;
- terminal status is UNKNOWN despite rollout evidence;
- compliance reports PASS with zero/unknown evidence;
- reports silently mix unrelated concurrent sessions.

## After Experiment 002

If instrumentation is valid, prepare Experiment 003 as the first controlled optimization A/B.

Experiment 003 requirements:
- choose one reproducible finding from Historical Analyzer / Context Efficiency analysis;
- frozen identical starting workspace;
- identical task;
- same Codex runtime/model/config;
- control run without the candidate intervention;
- candidate run with exactly one primary optimization variable;
- quality acceptance before token comparison;
- no default assumption that the variable is an output cap.
