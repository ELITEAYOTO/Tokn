# Tokn Context Pack Manifest Template

Pack ID: TOKN-CONTEXT-PACK-XXX
Generated at: YYYY-MM-DD
Source branch: <branch>
Source commit: <commit>
Worktree state: <clean|dirty>
Project state: <state>
Next milestone: <milestone>

## Canonical source

The Tokn Git repository is the source of truth.
A Context Pack is a portable documentation snapshot, never the canonical source.

If a pack conflicts with a newer canonical repository, the repository wins
after normal evidence/ADR precedence is applied.

## Versioning

Use simple pack numbering: 001, 002, 003...
The generated filename should include date and source commit when available.

Individual documents do not require SemVer.

## Evidence vocabulary

ACCEPTED - project decision.
VERIFIED - external fact confirmed by an authoritative current source.
OBSERVED - direct runtime/fixture/experiment observation.
RESEARCH - researched input not yet adopted or fully verified.
HYPOTHESIS - testable explanation/proposal.
UNKNOWN - unavailable or unverified.
SUPERSEDED - replaced by newer evidence/decision.

Measurement provenance terms such as DERIVED/INFERRED remain separate
and continue to follow Tokn's measurement design.

## Required content

Include when present:
STATUS.md, ROADMAP.md, CHANGELOG.md,
docs/INDEX.md, docs/MAINTENANCE.md,
docs/decisions, docs/design, docs/plans, docs/strategy,
docs/research, docs/experiments and referenced baselines needed for context.

## Excluded by default

Source code, target/build outputs, dependencies, binaries, secrets,
credentials, private raw sessions, huge traces and temporary workspaces.

Raw evidence should be referenced by provenance unless a sanitized artifact
is explicitly required for understanding a golden result.

## Integrity

A generated canonical pack should record:
source branch, source commit, clean/dirty state, included file inventory,
per-file SHA-256 when practical, and final ZIP SHA-256.

## Re-integration

1. Compare pack source commit with current repository.
2. Never overwrite newer canonical files blindly.
3. Merge document deltas at their canonical relative paths.
4. Reconcile STATUS, ROADMAP, INDEX, plans and ADRs.
5. Run documentation/repository checks.
6. Review git diff.
7. Commit only after consistency is confirmed.
8. Regenerate a new pack from the resulting canonical commit.

## First canonical pack

TOKN-CONTEXT-PACK-001 is generated only from the real Tokn repository
after the documentation delta is integrated and the worktree is controlled.

Any earlier export must carry DRAFT in its name.
