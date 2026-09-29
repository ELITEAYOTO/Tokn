# Tokn Experiment 001 - Real Astra Runbook

Last updated: 2026-09-27

## Objective

Measure whether conservative runtime-output caps can reduce token/context pressure
without reducing Astra quality, reasoning depth, verification, or completed work.

Candidate policy:
- file_read max_output_tokens <= 5000
- search max_output_tokens <= 3000

If capped evidence is insufficient, Astra must perform a narrower follow-up read/search.
It must not guess or skip required evidence.

Speed is not an optimization target.

## Important limitation

The existing 2026-09-26 JEM baseline was produced by a different development task.

Therefore:
- candidate vs old baseline is descriptive;
- it is not a causal same-task A/B comparison;
- quality and useful work remain mandatory;
- a later frozen same-task A/B is required for a strong causal claim.
## Files

User entry points:

- E:\Tokn\V0-CodexTkn-Consume\EDIT-EXP001-TASK.cmd
- E:\Tokn\V0-CodexTkn-Consume\START-EXP001-CANDIDATE.cmd
- E:\Tokn\V0-CodexTkn-Consume\FINISH-EXP001-CANDIDATE.cmd
- E:\Tokn\V0-CodexTkn-Consume\RECOVER-EXP001.cmd

Experiment root:

E:\Tokn\V0-CodexTkn-Consume\experiments\001-runtime-output-caps

JEM project:

E:\BlockBench-Plugin\JEM_Ultimate_v0.12.0_B07-B_WORKING\PROJECT

## Before the run

1. Decide the real JEM development task you genuinely want completed.
2. Open EDIT-EXP001-TASK.cmd.
3. Replace only the placeholder between ASTRA_PROMPT_START and ASTRA_PROMPT_END.
4. Keep Tokn/cap instructions out of the development task.
5. Include normal functional acceptance criteria in the Astra task when useful.
6. Save TASK.md.
7. Close Codex Desktop completely.
8. Do not manually add AGENTS.md to JEM for this experiment.
## Start

Double-click:

START-EXP001-CANDIDATE.cmd

Tokn automatically:
- refuses a second active experiment;
- refuses to start while Codex is still open;
- validates TASK.md;
- snapshots the JEM project with SHA-256;
- copies the exact task and policy into the run folder;
- temporarily installs the experiment policy as JEM AGENTS.md;
- enables official Codex diagnostic tracing;
- records environment/runtime versions;
- creates ACTIVE-RUN.json;
- puts ONLY the exact Astra development prompt in the clipboard;
- opens Codex Desktop.

Then:
1. Open the JEM Ultimate PROJECT workspace in Codex.
2. Paste the clipboard once as one new Astra prompt.
3. Send it.
4. Let Astra work normally.
5. Do not ask it to be faster or shorter.
6. Avoid extra messages unless genuinely required by the development task.
7. Do not manually edit JEM source files during the run.
## Finish

When Astra has completely finished:

1. Wait for its final response.
2. Close Codex Desktop completely.
3. Double-click FINISH-EXP001-CANDIDATE.cmd.

Tokn automatically:
- restores the pre-run JEM AGENTS.md state first;
- finalizes/imports the diagnostic trace;
- runs the official reducer oracle;
- snapshots JEM after the work;
- produces file added/modified/removed diff;
- runs npm run verify:local;
- produces token attribution;
- checks actual cap-policy compliance;
- produces offline cap simulation;
- compares descriptively with the old baseline;
- records matching local Codex rollout paths/hashes;
- writes RUN_COMPLETE.md;
- removes ACTIVE-RUN.json.

No source trace is modified.
## Human quality review

After FINISH completes, inspect JEM normally.

If host-sensitive behaviour changed, perform the relevant real Blockbench host gate.

Then fill the run's:

QUALITY_AFTER_RUN.md

Important human-only evidence:
- did the requested behaviour actually work?
- did Astra miss required work?
- any regression?
- any interruption or extra prompt?
- any manual edit?
- any visible lack of context?
- repeated re-reads?
- real Blockbench result if required;
- correctness/completeness/code-quality/verification ratings.

Do not score speed.

## If something goes wrong

Power loss, crash, abandoned run, or uncertain state:

1. Close Codex Desktop completely.
2. Do not delete trace/run files manually.
3. Double-click RECOVER-EXP001.cmd.

Recovery restores the best-known pre-run JEM instruction state,
clears CODEX_ROLLOUT_TRACE_ROOT, removes ACTIVE-RUN.json,
and preserves captured evidence for diagnosis.
## What to send back to ChatGPT

You do not need to manually copy logs or token counts.

After the run, say that Experiment 001 is finished and provide the run folder path
shown by FINISH (or simply ask ChatGPT to inspect the latest Experiment 001 run
through Remote Desktop Commander).

Tokn will have captured:
- exact task;
- policy hash;
- environment versions;
- before/after project snapshots;
- project diff;
- diagnostic bundle;
- token usage;
- attribution;
- actual tool caps;
- policy compliance;
- automated verify output;
- session evidence paths/hashes.

The main information only you can supply is the human/host quality judgement.

## Runtime comparability note

The older baseline and the candidate also use different Codex Desktop builds.

Baseline environment observed:
- Codex Desktop: 26.924.1866.0
- embedded codex: codex-cli 0.158.0-alpha.2

Current candidate environment before the run:
- Codex Desktop: 26.924.2738.0
- embedded codex: codex-cli 0.158.0-alpha.2.1

The candidate launcher records the exact versions again at start time.

Therefore old-baseline vs candidate deltas must not be interpreted as the
effect of Tokn alone. A later same-task / frozen-workspace / same-runtime A/B
is required for causal attribution.
