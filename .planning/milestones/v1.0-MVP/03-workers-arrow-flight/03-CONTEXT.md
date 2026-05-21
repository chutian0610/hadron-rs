---
phase: "03"
phase_name: "Workers + Arrow Flight"
status: "context_captured"
created: "2026-05-11"
---

# Phase 03: Workers + Arrow Flight — Context

## Domain

Workers execute distributed tasks in parallel and exchange data via Arrow Flight. The coordinator owns all query planning; workers are stateless executors that receive physical plans, execute them, and exchange data through Exchange operators.

## Canonical References

- `.planning/ROADMAP.md` — Phase 3 goal and success criteria
- `.planning/PROJECT.md` — Core value and architecture
- `.planning/PROJECT.md` — PITFALLS.md referenced (Exchange deadlock, Tokio runtime contention)

## Decisions

### Arrow Flight Integration — Worker-pull
**Decision:** Workers expose Arrow Flight servers; downstream consumers pull data on demand.

**Rationale:** Follows Trino-style pull model where data flows based on consumer demand, enabling natural backpressure. The coordinator orchestrates but is not in the data path.

### Task Execution Model — Dedicated Thread Pool
**Decision:** Use a dedicated thread pool for DataFusion task execution on workers.

**Rationale:** Compute-intensive query execution benefits from thread pool isolation — it prevents blocking tasks from starving the async runtime. Thread pool size should match the number of CPU cores for optimal parallelism.

**Connection to runtime separation:** Since CPU and IO runtimes are separate, compute tasks (physical plan execution) run on the dedicated thread pool while IO tasks (Arrow Flight network, S3 reads) run on the IO runtime. This avoids Tokio runtime contention — the CPU pool handles compute without being blocked by async network I/O.

### Exchange Operator Pattern — Streaming with Backpressure
**Decision:** Exchange operators use streaming with backpressure — data is produced and consumed concurrently without stage materialization.

**Rationale:** Maintains the pipeline streaming behavior that enables sub-second OLAP. Batch materialization would be a pipeline breaker.

### Retry Strategy — Same Worker Retry
**Decision:** Failed tasks retry on the same worker before rescheduling elsewhere.

**Rationale:** Avoids unnecessary data transfer when the failure was transient. If the same worker keeps failing, the task is marked failed and the coordinator is notified.

### Runtime Separation — Separate CPU/IO Runtimes
**Decision:** Implement separate CPU and IO runtimes for DataFusion now (Phase 3).

**Rationale:** Addresses the Tokio runtime contention pitfall identified in PITFALLS.md. DataFusion performs both CPU-intensive compute (query execution) and IO (S3 reads, Arrow Flight network I/O) — these should not compete for the same runtime threads.

## Deferred Ideas

- Multi-stage with shared buffer (e.g., S3/HDFS) — Phase 4+ scope
- Peer-to-peer direct Exchange — Phase 4+ scope

## Notes

- Phase 2 established: WorkerRegistry, QueryScheduler with locality scoring, HTTP server on port 50051
- Phase 1 established: DataFusion 43, ExecutorSession wrapping SessionContext
- Worker process (`octopus-worker`) is currently a stub — needs implementation in Phase 3
