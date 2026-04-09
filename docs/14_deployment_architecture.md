# 14 Deployment Architecture — Two Paths

## Path 1: Local (MCP, no server) — PRIORITY

```
Claude Desktop/Code
    ↓ stdio (MCP)
mcp_server.py
    ↓ PyO3 FFI (in-process, zero network overhead)
Rust Core (agent_browser.so)
    ↓ CDP
Chrome daemon
```

Pros: 7ms distill, shared memory, no port management, no serialization
Cons: single machine only

## Path 2: Cloud (HTTP server) — FUTURE

```
Agent (any machine)
    ↓ HTTP/gRPC
crates/server (axum)
    ↓
Rust Core + Chrome farm
```

Pros: multi-machine, scalable, Browser-as-a-Service
Cons: +10-20ms network overhead, state sync complexity

## Decision

V1.0: Path 1 (MCP + PyO3 direct). No HTTP server in the loop.
V2.0: Path 2 when going to cloud/SaaS.

crates/server stays in repo but is not in the critical path.
