# Tokn Plugin-to-Engine Integration

Status: LOCAL STDIO TRANSPORT PROTOTYPE ACCEPTED
Date: 2026-10-01

## Purpose

Define the preferred prototype boundary between future Codex integration
and the Tokn Rust Engine without making plugin infrastructure a dependency
of P8/P9.

This document is not an ADR. It records the locally validated transport prototype
derived from official OpenAI documentation and local Codex evidence.

Implementation/validation details:
`LOCAL-MCP-PROTOTYPE.md`.

## Established constraints

- Tokn Core/Engine remains the analytical source of truth.
- Plugin glue must not duplicate P0-P7 reducers.
- P8 must remain usable as a standalone self-contained runner.
- P9 must remain offline/replay capable.
- No permanent daemon is introduced without a measured need.
- Runtime/plugin capabilities remain versioned evidence.

## Current Codex evidence

OBSERVED on codex-cli 0.161.0-alpha.2:

Bundled/installed Codex plugins use compatibility manifests that point
`mcpServers` at root `.mcp.json` files.

Observed local MCP declarations launch processes with:
- `command`;
- `args`;
- optional `cwd`;
- timeout/tool policy fields;
- selected environment-variable names.

Examples include OpenAI's own `codex-app-tools`,
`unified-computer-use`, and `openai-developers` packages.

This demonstrates that the current local Codex runtime can host
command-launched local MCP servers for plugins.

It does not imply that the portable/public plugin format has identical rules.

## Official surface distinction

VERIFIED:

- portable plugins use root `plugin.json` and root `mcp.json`;
- current portable public examples use typed transports such as
  `streamable-http`;
- public plugin submission expects a reachable HTTPS MCP endpoint;
- compatibility plugins can use `.codex-plugin/plugin.json` plus
  `.mcp.json`;
- OpenAI agent environments support stdio MCP when the executable is
  available in the execution environment.

Therefore Tokn must distinguish:

1. local Codex integration;
2. portable/public plugin distribution.

A solution valid for local Codex must not be presented as automatically
portable to ChatGPT/public plugin distribution.

## Preferred local prototype

Prototype implemented after P8/P9, Experiment 002, contract freeze and Store foundation:

1. Keep the P8 Runner/Engine contract standalone.
2. Add a small Tokn MCP adapter executable/process.
3. Let a local Codex plugin launch that adapter through command-based MCP.
4. Expose only thin integration operations such as:
   - status/doctor;
   - analyze existing evidence;
   - request a Runner operation;
   - fetch structured findings.
5. The adapter calls shared Rust Engine/domain code rather than reimplementing it.
6. Persist history through Tokn Store when that layer exists.

The first prototype should be process-lifecycle bound:
Codex starts the MCP process when needed and the process exits with its host.

## Why not a permanent localhost service first

A persistent HTTP service would add:
- lifecycle management;
- port selection/conflicts;
- authentication/origin concerns;
- background process cleanup;
- another failure surface.

Current evidence does not show that Tokn needs those costs yet.

A service becomes justified only if later requirements need:
- multiple simultaneous clients;
- long-lived background observation;
- independent Desktop UI access;
- continuous indexing;
- cross-session shared state that cannot live cleanly in the Store.

Until then, local process/MCP is the lower-complexity candidate.

## Portable/public future

If Tokn later targets universal public plugin distribution,
the integration may require a remote HTTPS MCP surface or another
OpenAI-supported distribution mechanism.

That future distribution concern must not force the local analyzer
to become a network service today.

## Prototype exit criteria

The local MCP prototype is accepted only if it proves:
- clean startup/shutdown;
- no duplicated analysis logic;
- structured errors;
- explicit capability/version reporting;
- no secrets in plugin package/logs;
- successful invocation from the target Codex runtime;
- P8 Runner remains independently usable.

Observed prototype result:
- command-launched `tokn-mcp` over stdio starts/stops cleanly ;
- release smoke validates initialize/list/call directly against the process ;
- Codex 0.161.0-alpha.2 accepts isolated stdio registration ;
- Codex app-server launches Tokn and discovers both tools with `toolsError=null` ;
- an ephemeral idle zero-turn thread is sufficient for direct `mcpServer/tool/call` ;
- Codex directly calls `tokn_status` and validates the returned Store/server status ;
- no user authentication data is copied or inspected ;
- no model turn is started ;
- no permanent daemon or duplicated analysis logic is required.

Therefore command-launched local stdio MCP is accepted as the current local transport
prototype. Production/public plugin packaging remains a separate decision.

## Sources

Official:
- https://developers.openai.com/plugins/build/plugins
- https://developers.openai.com/api/docs/guides/agents-api/tools/plugins
- https://developers.openai.com/api/docs/guides/agents-api/tools/mcp

Local:
Codex 0.161.0-alpha.2 plugin manifests and .mcp.json files inspected
on 2026-09-30.
