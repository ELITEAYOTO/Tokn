# Dynamic Workspace Tracking

## Problem

Experiment 001 watched one fixed path:
B07-B_WORKING\PROJECT.

The task explicitly created:
- B07-B_FROZEN;
- B07-C_WORKING.

Astra then worked in B07-C_WORKING.

Fixed-root observation therefore reported zero changes even though the task produced
13 added and 4 modified files in the actual output workspace.

## V0.1 model

ExperimentWorkspaceSet:
- declared_input_roots;
- declared_expected_output_roots;
- discovered_roots;
- active_roots;
- quality_gate_root;
- snapshot_scope.

A run may have more than one workspace root.
## Discovery sources

Workspace roots may be discovered from:
- experiment config;
- task-declared paths;
- session_meta.cwd;
- runtime_workspace_roots;
- tool call workdir;
- filesystem mutations that create/copy/move directories;
- known project manifests appearing in new directories.

Discovery has provenance and confidence.

Tokn never silently switches quality_gate_root.
It records why the new root was selected.

## Snapshot strategy

Before run:
- snapshot declared input roots;
- inventory parent directory names/hashes at shallow depth;
- record expected output paths when known.

During/after run:
- detect newly created candidate roots;
- fingerprint manifests and project structure;
- snapshot every relevant output root;
- compare source freeze -> candidate output when the task is a fork/copy workflow.

## Quality gate selection

Quality gate target priority:
1. explicit experiment output root;
2. discovered root marked as primary output;
3. original root only if no output root exists.

If ambiguous:
status = QUALITY_TARGET_AMBIGUOUS
and no automatic PASS is allowed.

## Experiment 001 replay acceptance

V0.1 must identify:
- B07-B_FROZEN as immutable reference;
- B07-C_WORKING as candidate output;
- 13 added;
- 4 modified;
- 0 removed;
- verify:local target = B07-C_WORKING\PROJECT.
