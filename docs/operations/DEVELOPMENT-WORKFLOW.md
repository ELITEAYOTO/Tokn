# Development Workflow

Status: ACTIVE
Date: 2026-10-03

This document defines the normal development/merge path for Tokn.
It exists to prevent the historical failure mode where a useful PR was merged before its final validation state was green.

## Local toolchain

Canonical toolchain is defined by `rust-toolchain.toml`:
- Rust 1.97.1;
- rustfmt;
- Clippy;
- target `x86_64-pc-windows-msvc`.

Install Rustup once. Entering the repository selects the pinned toolchain automatically.

## Resource guard

Local Rust commands go through `scripts/with-msvc.cmd`, which limits parallel work by default:
- `CARGO_BUILD_JOBS=2`;
- `RUST_TEST_THREADS=2`;
- override with `TOKN_CARGO_JOBS=1` for especially heavy work.

Do not run multiple CPU-intensive validation/build tasks at the same time. Prefer one heavy step at a time, and use the gentler single-job override for exceptional installs or rebuilds.

## Branch workflow

For product/code changes:
1. sync `main` with `origin/main`;
2. create a focused branch;
3. make one bounded slice;
4. run local checks;
5. push and open a PR against `main`;
6. wait for the normal `CI` workflow to be fully green;
7. squash-merge;
8. verify the post-merge `main` CI;
9. delete the merged branch.

## Local gates

Fast/default pre-PR check:

```powershell
.\scripts\dev-check.ps1
```

This runs:
- repository publication/privacy check;
- PowerShell syntax validation;
- rustfmt check;
- strict Clippy (`-D warnings`);
- workspace tests;
- documentation consistency.

Before opening or updating a merge-ready PR:

```powershell
.\scripts\dev-check.ps1 -Full
```

The full mode also runs release build/provenance, MCP smoke, Experiment 001 golden replay and Experiment 002 source-selection regression.
The GitHub workflow remains the final authority.

## Merge rule

Do **not** merge because a branch-level ad-hoc workflow is green.
The normal PR CI against current `main` must be green on the exact PR head SHA.
Old red commits in the PR history are acceptable only when the latest head and required checks are green.

## Recommended GitHub protection for `main`

Repository administration should enforce:
- pull request required before merge;
- required status check: CI / `V0.1 validation`;
- branch must be up to date before merge;
- direct pushes to `main` disabled for normal development;
- force-push/delete protection for `main`;
- squash merge as the normal integration method.

The repository currently has no visible repository ruleset through the connected GitHub integration. The integration cannot administer branch protection, so this is an explicit repository-admin action rather than an automated Tokn code change.

## Temporary workflows

Do not add per-branch validation workflows when the local pinned toolchain can reproduce the normal CI.
A temporary workflow is acceptable only for a specific infrastructure limitation and must be removed before merge.

## Scope discipline

A PR should preserve Tokn's evidence boundary:
- no `PASS` from missing evidence;
- no causal saving claim from descriptive overlap;
- no raw evidence persistence solely for convenience;
- schema/version changes are additive or explicitly migrated and fail closed on unknown future versions.
