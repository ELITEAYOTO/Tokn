# Tokn Workspace Layout

Status: ACCEPTED WORKSPACE CONVENTION
Date: 2026-09-30

The outer workspace separates canonical source from generated or sensitive runtime data.

## Layout

- `tool/` - canonical Git repository and source of truth.
- `experiments/` - mutable experiment workspaces and evidence outside Git.
- `traces/` - raw diagnostic traces; potentially sensitive.
- `artifacts/` - generated packages and analysis outputs.
- `context-packs/` - portable documentation snapshots generated from clean Git commits.
- `launchers/` - convenience .cmd wrappers for historical/manual workflows.
- `archive/` - non-canonical historical inputs such as pre-Tokn scripts and raw research reports.

## Rules

Only `tool/` defines project truth.
Runtime evidence outside `tool/` must be referenced by provenance, not treated as canonical documentation.
Raw traces, private sessions, credentials and temporary workspaces must not be committed.
Context Packs are derived snapshots and never override a newer Git state.
Historical launchers may remain usable, but P8 should replace experiment-specific orchestration with the self-contained Runner.

## Repository root

The Git root should stay minimal:
README, STATUS, ROADMAP, CHANGELOG, SECURITY, Cargo/config files and source directories.

Documentation templates and references belong under `docs/`, not at repository root unless they are true project entry points.
