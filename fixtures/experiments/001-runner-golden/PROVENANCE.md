# Experiment 001 Runner Golden Fixture

Status: ACCEPTED TEST FIXTURE

This fixture makes P9 release validation deterministic and repo-contained.

## Evidence origin

- Session identities, token usage records, tool categories, caps, workdirs and terminal states are reconstructed from normalized real Experiment 001 session evidence captured by Tokn.
- Historical aggregate facts are cross-checked against the accepted Experiment 001 postmortem and V0.1 test matrix.
- Raw prompts, base instructions, tool outputs and user content are intentionally excluded.
- Commands are synthetic neutral strings selected only to preserve Tokn command categories and cap-policy behavior.

## Workspace diff provenance

The accepted historical replay recorded 13 added / 4 modified / 0 removed for historical frozen-workspace -> candidate-workspace roles.
The corrected raw snapshot files from that replay were not retained, and the live B07-C directory has since drifted; on 2026-09-30 it re-snapshotted as 11 / 8 / 0.

Therefore the snapshot file names, hashes and byte sizes in this fixture are synthetic test data chosen only to reproduce the accepted 13 / 4 / 0 contract. They are not historical file-level evidence.

## Use

Run from the repository root. session_candidates prevents dependency on the current Codex session index.
The quality gate is explicitly NOT_REQUIRED in this fixture; executable quality-gate behavior is covered separately by P8 integration tests.

## Privacy

Thread IDs, agent names/paths and filesystem roots in the executable fixture are synthetic. Historical raw identifiers and personal filesystem roots are intentionally not retained.
