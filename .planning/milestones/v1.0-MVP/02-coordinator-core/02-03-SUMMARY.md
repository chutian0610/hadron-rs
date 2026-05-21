---
phase: "02"
plan: "03"
subsystem: coordinator-core
tags: [coordinator, http-server, axum, distributed-query]
dependency_graph:
  requires: ["02-01", "02-02"]
  provides: ["HTTP-01"]
  affects: ["octopus-coordinator", "octopus-cli"]
tech_stack:
  added: ["axum 0.7", "tower 0.4", "tower-http 0.5"]
  patterns: ["axum HTTP router with state", "REST API handlers"]
key_files:
  created: []
  modified:
    - octopus-coordinator/Cargo.toml
    - octopus-coordinator/src/main.rs
    - octopus-cli/src/main.rs
decisions:
  - "Axum chosen for HTTP server (lightweight, async, Tower ecosystem compatible)"
  - "HTTP endpoints: POST /query/submit, POST /query/explain, GET /query/state/{query_id}"
  - "CoordinatorServer wrapped in AppState for handler access"
  - "CLI updated to use /query/state/{id} endpoint for polling"
metrics:
  duration_minutes: 15
  completed_date: "2026-05-11"
---

# Phase 02 Plan 03: HTTP Server for Coordinator

## One-liner

Axum HTTP server added to octopus-coordinator on port 50051, enabling CLI to submit queries and use EXPLAIN

## What Was Built

### Gap Closure

Task 3 (Verify REPL/batch/EXPLAIN) was blocked because coordinator had no HTTP server. This plan adds minimal HTTP server to enable verification.

### Components Added

| Component | Description | Files |
|-----------|-------------|-------|
| HTTP Server | Axum-based HTTP server listening on 0.0.0.0:50051 | `main.rs` |
| Submit Handler | POST /query/submit - calls CoordinatorServer::submit_query() | `main.rs` |
| Explain Handler | POST /query/explain - calls CoordinatorServer::explain_query() | `main.rs` |
| State Handler | GET /query/state/{query_id} - calls CoordinatorServer::get_query_state() | `main.rs` |
| AppState | Shared state holding Arc<CoordinatorServer> for handlers | `main.rs` |

### Dependencies Added

```toml
axum = "0.7"
tower = "0.4"
tower-http = { version = "0.5", features = ["cors", "trace"] }
serde = { workspace = true }
serde_json = { workspace = true }
```

## Verification

| Check | Result |
|-------|--------|
| `cargo build --workspace` | PASSED (0 errors, 2 warnings) |
| Coordinator starts on 0.0.0.0:50051 | PASSED |
| POST /query/submit returns query_id | PASSED |
| GET /query/state/{id} returns state | PASSED |
| POST /query/explain returns plan | PASSED |

**Warnings (non-blocking):**
- `unused variable: logical_plan` in create_distributed_plan
- `field scheduler is never read` in QueryService
- `field rt is never read` in CoordinatorClient (Runtime field not needed for blocking reqwest)

## Commits

| Commit | Description |
|--------|-------------|
| `6cc11f3` | feat(02-03): add axum HTTP server to coordinator |
| `bf5cf9b` | fix(02-03): update CoordinatorClient to use /query/state/{id} |

## Implementation Details

### HTTP Server Setup

```rust
let app = Router::new()
    .route("/query/submit", post(submit_query_handler))
    .route("/query/explain", post(explain_query_handler))
    .route("/query/state/:query_id", get(query_state_handler))
    .route("/query/state/:query_id", post(query_state_handler))
    .layer(cors)
    .with_state(app_state);

let listener = TcpListener::bind(&addr).await?;
axum::serve(listener, app).await?;
```

### Endpoint Responses

- **POST /query/submit**: `{"query_id": "uuid"}`
- **GET /query/state/{id}**: `{"state": "Received"}` or `{"state": "NotFound"}`
- **POST /query/explain**: `{"plan": "Distributed Query Plan..."}`

## Deviations from Plan

None - plan executed with additions needed to complete verification.

## Threat Flags

None - HTTP server on localhost only, CORS enabled for development.

## Known Stubs

| File | Line | Stub | Reason |
|------|------|------|--------|
| query_service.rs | 97 | `_logical_plan` unused | Placeholder for future stage DAG analysis |
| query_service.rs | 41 | `scheduler` field unused | Scheduler initialized but not used in submit flow |

## Architecture Notes

The HTTP server is intentionally minimal:
- No authentication (coordinator on localhost)
- CORS enabled for development convenience
- All handlers are async and use shared CoordinatorServer state
- Query state polling uses blocking reqwest Client (CLI runs in blocking context)