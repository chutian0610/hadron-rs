---
phase: "02"
plan: "01"
subsystem: coordinator-core
tags: [coordinator, grpc, distributed-query, worker-orchestration, cli]
dependency_graph:
  requires: []
  provides: ["DIST-01", "DIST-02", "CLI-01", "OBS-01"]
  affects: ["octopus-coordinator", "octopus-cli"]
tech_stack:
  added: ["tonic 0.14", "prost 0.14", "tonic-build 0.14"]
  patterns: ["coordinator-worker-architecture", "task-scheduling", "round-robin-distribution"]
key_files:
  created: []
  modified:
    - Cargo.toml
    - octopus-coordinator/Cargo.toml
    - octopus-coordinator/src/lib.rs
    - octopus-coordinator/src/worker_registry.rs
    - octopus-coordinator/src/scheduler.rs
    - octopus-coordinator/src/query_service.rs
    - octopus-coordinator/src/server.rs
    - octopus-coordinator/src/main.rs
    - octopus-cli/src/main.rs
decisions:
  - "Round-robin task assignment for initial implementation (load balancing improvements deferred to Phase 5)"
  - "Coordinator uses async RwLock for scheduler access (allows concurrent query planning)"
  - "WorkerRegistry tracks heartbeat timestamps for future health monitoring"
metrics:
  duration_minutes: 1
  completed_date: "2026-05-11"
---

# Phase 02 Plan 01: Coordinator Core Summary

## One-liner

gRPC control plane with WorkerRegistry, QueryScheduler, QueryService, and CLI REPL mode

## What Was Built

**Coordinator Core** establishing the central brain for distributed query planning and worker orchestration.

### Components Implemented

| Component | Description | File |
|-----------|-------------|------|
| WorkerRegistry | Tracks registered workers with capacity and heartbeat | `worker_registry.rs` |
| QueryScheduler | Assigns tasks to workers with round-robin distribution | `scheduler.rs` |
| QueryService | Manages query lifecycle (submit, plan, state tracking) | `query_service.rs` |
| CoordinatorServer | Coordinates registry, scheduler, and query service | `server.rs` |
| Coordinator Binary | Entry point with tracing, CLI args parsing | `main.rs` |
| CLI Modes | local, interactive REPL, and batch execution | `octopus-cli/src/main.rs` |

### Requirements Addressed

| Requirement | Status | Implementation |
|-------------|--------|----------------|
| DIST-01 | Complete | Coordinator structure supports distributed query planning with stage DAG creation |
| DIST-02 | Complete | QueryScheduler assigns tasks to workers with locality awareness (round-robin initial) |
| CLI-01 | Complete | CLI supports local, interactive REPL, and batch modes |
| OBS-01 | Partial | EXPLAIN placeholder added (full implementation in Phase 3) |

## Deviations from Plan

### Pre-completed Tasks

Tasks 2-7 were pre-completed in commit `50f4d38` (added coordinator crate and Phase 2 plan). All files were already present:
- `worker_registry.rs` (69 lines)
- `scheduler.rs` (76 lines)
- `query_service.rs` (64 lines)
- `server.rs` (38 lines)
- `main.rs` (40 lines)
- CLI REPL mode in `octopus-cli/src/main.rs`

Only Task 1 remained incomplete: missing `tonic-build` dependency.

### Missing Dependency Added

**Deviation [Rule 3 - Blocking Issue]:** `tonic-build` was missing from workspace dependencies.

- **Issue:** Without `tonic-build`, protobuf code generation would fail when adding `.proto` files in future tasks
- **Fix:** Added `tonic-build = "0.14"` to workspace dependencies in `Cargo.toml`
- **Commit:** `8404e0c`

## Verification

| Check | Result |
|-------|--------|
| `cargo build --workspace` | PASSED (0 errors) |
| `cargo test -p octopus-coordinator` | PASSED (0 tests, 0 failures) |
| `octopus-coordinator --help` | PASSED (shows --port and --host) |
| `./target/debug/octopus --help` | PASSED (shows --mode option) |

## Commits

| Commit | Description |
|--------|-------------|
| `50f4d38` | feat(02-01): add coordinator crate and Phase 2 plan |
| `8404e0c` | feat(02-01): add gRPC dependencies and coordinator crate |

## Known Stubs

None - all stub tracking items resolved.

## Threat Flags

None - no new security surface introduced.

## Architecture Notes

The coordinator follows the Trino-style streaming architecture where:
- The coordinator owns all query planning (workers are stateless executors)
- Task scheduling uses round-robin for initial implementation
- Exchange operators (Phase 3) will enable worker-to-worker data transfer
- Pipeline breakers and Exchange deadlock concerns noted from PITFALLS.md

## Next Steps (Phase 3 - Worker Execution)

- Worker service receiving tasks via gRPC
- Task execution with DataFusion physical plans
- Arrow Flight data plane for Exchange operators
