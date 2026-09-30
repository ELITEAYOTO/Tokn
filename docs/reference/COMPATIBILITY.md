# Compatibility

Initial target:
- Windows 10 / 11 x64;
- Rust 1.97.1;
- target `x86_64-pc-windows-msvc`;
- Codex Desktop, VS Code Codex runtime, or explicit custom Codex binary.

Codex capability detection is probe-based rather than hard-coded to version strings.
The current test machine exposes Codex Desktop and the hidden `debug trace-reduce` command.

If normal shell linking fails, use `scripts\with-msvc.cmd` to initialize a complete Visual Studio C++ environment.

## V0.1 compatibility note - 2026-09-27

Experiment 001 observed:
- Codex Desktop 26.924.2738.0 ;
- codex-cli 0.158.0-alpha.2.1.

The earlier healthy baseline used:
- Codex Desktop 26.924.1866.0 ;
- codex-cli 0.158.0-alpha.2.

Therefore runtime version is part of experiment identity.

V0.1 adapters must:
- detect schema/capabilities from evidence ;
- preserve unknown fields ;
- fail soft on unsupported records ;
- avoid exact-version coupling ;
- keep sanitized fixtures for every observed shape.

Diagnostic trace availability/completeness is capability-like evidence,
not an assumption.

Standard session rollouts are required as fallback when diagnostic health is PARTIAL.

## P6 runtime note - 2026-09-29

On codex-cli 0.158.0-alpha.2.1:
- hooks are supported and real Desktop lifecycle execution was observed;
- unified exec is exposed to PreToolUse as Bash;
- max_output_tokens is not forwarded to that callback.

Tokn records this as SUPPORTED_INSUFFICIENT_INPUT for the historical cap policy.
This is capability evidence, not a reason to couple the adapter permanently to version 0.158.

Future context-management/cache capabilities must be detected and recorded per runtime/config
before Tokn compares runs that depend on them.

## Current development runtime - 2026-09-30

Observed on the Tokn development PC:
- OpenAI.Codex app 26.928.1915.0 ;
- codex-cli 0.161.0-alpha.2.

This does not supersede the historical P6 evidence captured on 0.158.0-alpha.2.1.
Capability decisions remain versioned and evidence-based rather than inferred from version strings alone.
