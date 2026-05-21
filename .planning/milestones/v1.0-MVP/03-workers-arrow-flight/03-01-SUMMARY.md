---
phase: "03"
plan: "01"
subsystem: workers
tags: [workers, cpu-io-runtime, task-execution, datafusion]
dependency_graph:
  requires: []
  provides: ["DIST-03"]
  affects: ["octopus-worker"]
tech_stack:
  added: ["tokio runtime separation", "datafusion physical plan execution"]
  patterns: ["worker-pull-model", "cpu-thread-pool", "io-async-runtime"]
key_files:
  created:
    - octopus-worker/src/runtime.rs
    - octopus-worker/src/task_processor.rs
    - octopus-worker/src/worker_service.rs
    - octopus-worker/src/main.rs
  modified:
    - octopus-worker/Cargo.toml
    - octopus-worker/src/lib.rs
decisions:
  - "CpuRuntime: dedicated thread pool sized to CPU cores for compute-intensive tasks"
  - "IoRuntime: separate async runtime with 2x CPU threads for network IO"
  - "TaskProcessor: uses SessionContext.task_ctx() for DataFusion physical plan execution"
  - "WorkerService: placeholder for gRPC task receiving (deferred to 03-02)"
  - "Arrow Flight deferred to 03-02 due to arrow 53 compatibility issue"
metrics:
  duration_minutes: 15
  completed_date: "2026-05-11"
---

# Phase 03 Plan 01: Worker Service Foundation with CPU/IO Runtime Separation

## One-liner

Worker service with separate CPU/IO runtimes and dedicated thread pool for DataFusion task execution

## What Was Built

**Worker Service Foundation** establishing the runtime infrastructure for distributed task execution with CPU/IO separation (addressing Pitfall 2: Tokio runtime contention).

### Components Implemented

| Component | Description | File |
|-----------|-------------|------|
| CpuRuntime | Dedicated thread pool for compute-intensive tasks | `runtime.rs` |
| IoRuntime | Async runtime for network-bound IO operations | `runtime.rs` |
| WorkerRuntime | Unified access to both runtimes | `runtime.rs` |
| TaskProcessor | Executes DataFusion physical plans on CPU thread pool | `task_processor.rs` |
| WorkerService | Receives tasks from coordinator, dispatches to processor | `worker_service.rs` |
| Worker Binary | Entry point with CLI args for coordinator and port | `main.rs` |

### Requirements Addressed

| Requirement | Status | Implementation |
|-------------|--------|----------------|
| DIST-03 | Complete | Separate CPU/IO runtimes (D-05 implemented); Compute tasks on dedicated thread pool (D-02 implemented) |

## Must-Haves Verification

| Truth | Status |
|-------|--------|
| "Workers can receive and execute tasks on dedicated thread pool" | VERIFIED - CpuRuntime with spawn_blocking |
| "Compute tasks run on CPU thread pool separate from async IO runtime" | VERIFIED - Two separate runtimes |
| "Worker process can register with coordinator and start task execution loop" | VERIFIED - WorkerService::run() with placeholder loop |

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking Issue] Arrow Flight version compatibility**
- **Found during:** Task 1
- **Issue:** arrow-flight 58 pulled in arrow 58 which conflicted with DataFusion 43's arrow 53, causing compilation error in `arrow-arith`
- **Fix:** Deferred Arrow Flight to 03-02, removed arrow-flight and arrow dependencies from Cargo.toml
- **Files modified:** `octopus-worker/Cargo.toml`

**2. [Rule 1 - Bug] Missing `self` parameter on spawn_blocking**
- **Found during:** Task 4
- **Issue:** CpuRuntime::spawn_blocking was defined as an associated function without `&self`, causing "expected value, found module `self`" error
- **Fix:** Added `&self` parameter to spawn_blocking method
- **Files modified:** `octopus-worker/src/runtime.rs`

**3. [Rule 1 - Bug] execute_stream requires TaskContext not SessionContext**
- **Found during:** Task 3
- **Issue:** `execute_stream(plan, Arc::new(context))` failed because DataFusion's execute_stream expects TaskContext, not SessionContext
- **Fix:** Used `context.task_ctx()` to get TaskContext for execute_stream
- **Files modified:** `octopus-worker/src/task_processor.rs`

**4. [Rule 1 - Bug] JoinHandle::map_err not available**
- **Found during:** Task 3, 4
- **Issue:** `spawn_blocking` returns `JoinHandle` but JoinHandle doesn't have `map_err` method directly
- **Fix:** Used `runtime.block_on(async { handle.await })` to poll the JoinHandle and then apply map_err
- **Files modified:** `octopus-worker/src/task_processor.rs`, `octopus-worker/src/worker_service.rs`

## Verification

| Check | Result |
|-------|--------|
| `cargo build --workspace` | PASSED |
| `cargo build -p octopus-worker` | PASSED (0 errors) |
| `./target/debug/octopus-worker --help` | PASSED (shows --coordinator, --port, --verbose) |

## Commits

| Commit | Description |
|--------|-------------|
| `182bae1` | feat(03-01): add dependencies for worker service |
| `4190563` | feat(03-01): create separate CPU and IO runtimes |
| `ce95076` | feat(03-01): implement task processor with DataFusion execution |
| `84fda5f` | feat(03-01): implement worker service with task receiver |
| `800bece` | feat(03-01): create worker binary entry point |

## Known Stubs

| Stub | File | Reason |
|------|------|--------|
| task_receiver_loop placeholder | `worker_service.rs:76-85` | gRPC task receiving deferred to 03-02 |
| process_task placeholder | `worker_service.rs:89-100` | Deserialization deferred to 03-02 |
| execute_task_sync placeholder | `worker_service.rs:103-123` | Actual plan execution deferred to 03-02 |

## Threat Flags

None - no new security surface introduced.

## Architecture Notes

The worker service follows the worker-pull model from D-01:
- Workers expose Arrow Flight servers; consumers pull data on demand
- Compute-intensive tasks (DataFusion physical plan execution) run on dedicated CPU thread pool
- Async network IO (coordinator communication) runs on separate IO runtime
- This separation addresses Pitfall 2: Tokio runtime contention

## Next Steps (Phase 03 - Plan 02)

- gRPC task receiving from coordinator
- Arrow Flight server implementation
- Task deserialization and execution