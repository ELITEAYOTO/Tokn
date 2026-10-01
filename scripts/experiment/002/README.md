# Experiment 002 canonical harness

This directory is the versioned definition of Experiment 002.
Real run artifacts remain outside Git under the outer workspace
`experiments/002-instrumentation-validation/`.

Provision or refresh the external harness only from a clean commit:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/experiment/prepare-exp002-harness.ps1 `
  -ProjectRoot <project> -WorkspaceWatchRoot <watch-root>
```

The provisioner pins Git HEAD, generates machine-local paths, creates the
launchers, and writes `HARNESS-MANIFEST.json` with SHA-256 hashes.
START refuses harness drift or a release binary whose provenance does not
match the pinned commit.
