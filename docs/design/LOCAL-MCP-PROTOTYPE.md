# Tokn Local MCP Prototype

Status: TRANSPORT PROTOTYPE ACCEPTED / DIRECT HOST TOOL CALL VALIDATED
Date: 2026-10-01

## Purpose

Validate the preferred local Codex integration boundary:

Codex runtime
-> command-launched stdio MCP process
-> shared Tokn Store / domain contracts.

This prototype validates transport and host discovery.
It does not create a production plugin package and does not change the standalone Runner.

## Implementation

Executable:
`apps/tokn-mcp`

Binary:
`tokn-mcp`

Transport:
`stdio`

Lifecycle:
process-bound.
The host launches the process and the process exits when stdin closes.

No localhost port is opened.
No permanent daemon is installed.

## Shared boundaries

The MCP process reuses:
- `tokn-platform::observer_database_path()`;
- `tokn-storage::Database`;
- Store V2 read-only query APIs;
- frozen measurement contract constants.

The MCP layer contains no token accounting reducer, RunGroup reducer,
workspace resolver, validity reducer or Store SQL schema logic.

## Current tools

### tokn_status

Read-only.

Returns:
- MCP server name/version;
- stdio/read-only transport metadata;
- frozen measurement contract ID/version;
- Store schema version;
- project/workspace/run/agent/runtime-profile counts;
- currently exposed Tokn MCP tools.

It intentionally does not return the local database path.

### tokn_recent_runs

Read-only.

Input:
- optional `limit`, 1..100, default 10.

Returns recent Store V2 run summaries:
- privacy-preserving project/workspace/profile IDs;
- contract/evidence versions;
- root thread/terminal;
- source health;
- validity/quality summary;
- agent count;
- usage totals;
- logical total when authoritative.

No raw prompt, raw session JSONL, cwd or personal path is returned.

## MCP protocol surface

Implemented JSON-RPC methods:
- `initialize`;
- `ping`;
- `tools/list`;
- `tools/call`.

Accepted lifecycle notification:
- `notifications/initialized`.

Structured JSON-RPC errors are returned for:
- invalid requests;
- unknown methods;
- invalid tool arguments;
- Store query failures.

The prototype currently advertises compatibility with the MCP protocol versions
observed/expected by the local integration test, while negotiating the client's
supported version.

## Validation layer 1 - Rust tests

`tokn-mcp` has unit tests for:
- initialize + protocol negotiation;
- tools/list;
- status on an empty Store;
- recent-runs on an empty Store;
- structured errors and notifications;
- no database-path leak in tool output.

Workspace Clippy runs with `-D warnings`.

## Validation layer 2 - real process stdio smoke

Script:
`scripts/validation/check-mcp-prototype.ps1`

The release binary is launched as a real child process.
The script sends:
1. initialize;
2. notifications/initialized;
3. tools/list;
4. tokn_status;
5. tokn_recent_runs.

Acceptance:
- process exits 0 after stdin closes;
- stderr is empty;
- protocol negotiation succeeds;
- exactly the expected tools are listed;
- Store schema V2 is reported;
- server reports stdio/read-only;
- local DB path is not present in output.

Observed result on 2026-10-01:
**PASS**.

This smoke is included in GitHub CI after the release build.

## Validation layer 3 - target Codex CLI registration

Observed runtime:
`codex-cli 0.161.0-alpha.2`.

Validation used a disposable `CODEX_HOME`.
The user's normal Codex configuration was not modified.

The real Codex CLI accepted:

`codex mcp add tokn -- <tokn-mcp> --db <temporary-db>`

`codex mcp get tokn --json` reported:
- enabled=true;
- transport=stdio;
- expected executable;
- expected arguments.

The disposable configuration was deleted after the test.

## Validation layer 4 - target Codex host launch and discovery

Script:
`scripts/validation/check-codex-mcp-runtime.ps1`

This is intentionally a local runtime validation, not a GitHub CI requirement,
because GitHub runners do not provide the target Codex installation.

The script:
1. creates an isolated Codex home;
2. registers only Tokn;
3. launches the real `codex app-server --stdio`;
4. sends app-server `initialize`;
5. sends `mcpServerStatus/list` for Tokn;
6. verifies the returned MCP inventory;
7. deletes the isolated home and temporary Store.

Observed result on 2026-10-01:
**PASS**.

Codex returned:
- server name: `tokn`;
- serverInfo.name: `tokn-mcp`;
- serverInfo.version: `0.1.0`;
- MCP tools capability;
- `tokn_status`;
- `tokn_recent_runs`;
- toolsError = null;
- authStatus = unsupported, expected for local stdio without OAuth.

This proves that the target Codex runtime actually starts the Tokn MCP process,
performs MCP initialization and discovers the Tokn tool catalog.

No model turn is created by this validation.

## Direct host tool-call validation

Codex app-server exposes:
`mcpServer/tool/call`.

The local validation now proves this path without a model turn:

1. create a disposable isolated `CODEX_HOME`;
2. register only Tokn;
3. start the native Codex app-server;
4. initialize the app-server experimental API;
5. discover Tokn through `mcpServerStatus/list`;
6. create an ephemeral Codex thread;
7. verify the thread is `idle` with zero turns;
8. call `tokn_status` through `mcpServer/tool/call`;
9. validate the returned Tokn status payload;
10. delete the isolated Codex home and temporary Store.

Observed result on 2026-10-01:
**PASS**.

The direct Codex tool call returned:
- server name `tokn-mcp`;
- transport `stdio`;
- `read_only=true`;
- Store schema version `2`;
- `tokn_status` and `tokn_recent_runs`.

No user authentication data was copied or inspected.
No `turn/start` request was submitted.
The validation thread remained idle with zero model turns.

This proves direct host invocation of a Tokn MCP tool.
It does **not** prove model-driven tool selection or production plugin UX; those remain separate concerns.

## Acceptance decision

The local transport prototype is accepted because:
- startup/shutdown is clean;
- stdio MCP framing works in release;
- errors are structured;
- version/capability reporting is explicit;
- no analysis logic is duplicated;
- Store access uses shared Rust logic;
- target Codex accepts the registration;
- target Codex launches the process and discovers its tools;
- target Codex directly calls `tokn_status` through `mcpServer/tool/call`;
- the validation uses an idle zero-turn ephemeral thread;
- standalone Runner behavior remains independent;
- no daemon is required.

The exact production/plugin packaging surface remains separate and may evolve.

## Non-goals

Not part of this prototype:
- permanent daemon;
- HTTP MCP;
- public/remote plugin distribution;
- GUI;
- model-driven tool invocation validation;
- active optimization;
- Findings mutation;
- Store writes through MCP.

## Next layer

The transport boundary is now sufficiently proven for Tokn to resume the
observation-first product roadmap:

Historical Analyzer + Context Ledger
-> evidence-backed recurring Findings
-> causal Experiment 003
-> Advisor
-> optional active optimizer only after repeated validated wins.
