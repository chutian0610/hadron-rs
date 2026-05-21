---
title: Worker Runtime
type: concept
tags: [Tokio,runtime-separation,CPU-IO,retry,metrics]
sources:
  - octopus-worker/src/
  - octopus-executor/src/
related:
  - "[[Architecture]]"
  - "[[Execution]]"
  - "[[Exchange]]"
---

# Worker Runtime

The worker runtime handles task execution with proper resource isolation and retry mechanisms.

## Separate Tokio Runtimes

Workers use separate Tokio runtimes for CPU-intensive work vs I/O-bound work, addressing Pitfall 2 (Tokio Runtime Contention).

`octopus-worker/src/worker_runtime.rs:1`

### Why Separate Runtimes?

DataFusion uses the same Tokio runtime for:
- CPU-intensive compute (hashing, sorting, aggregation)
- I/O-bound work (S3 reads, Arrow Flight network I/O)

When mixed on the same runtime, CPU-intensive tasks can block I/O progress.

### Configuration

```rust
// CPU runtime: bounded thread pool for compute
let cpu_runtime = tokio::runtime::Builder::new_multi_thread()
    .worker_threads(num_cpus)
    .build()?;

// IO runtime: higher concurrency for network I/O
let io_runtime = tokio::runtime::Builder::new_multi_thread()
    .worker_threads(num_cpus * 2)
    .enable_io()
    .build()?;
```

## WorkerService

Coordinates task reception and execution on the worker.

`octopus-worker/src/worker_service.rs:1`

### Responsibilities

- **Task Reception**: Receives ExecuteTask gRPC calls from coordinator
- **Execution Dispatch**: Routes tasks to TaskProcessor
- **Flight Server Coordination**: Manages Arrow Flight data transfer
- **Metrics Collection**: Tracks CPU, memory, rows processed

## MetricsCollector

Observability metrics for worker execution.

`octopus-worker/src/metrics_collector.rs:1`

### Collected Metrics

| Metric | Description |
|--------|-------------|
| `cpu_usage` | Current CPU utilization |
| `memory_usage` | Current memory consumption |
| `rows_processed` | Total rows processed |
| `active_tasks` | Currently executing tasks |
| `completed_tasks` | Total completed tasks |
| `failed_tasks` | Total failed tasks |

### Metrics Endpoint

Workers expose metrics via HTTP:

```
GET /metrics
```

## Retry Handling

Failed tasks are retried using exponential backoff with jitter.

`octopus-executor/src/retry_handler.rs:1`

### Retry Policy

```rust
pub struct RetryPolicy {
    pub base_delay: Duration,
    pub max_delay: Duration,
    pub max_attempts: usize,
    pub jitter_factor: f64,
}

impl RetryPolicy {
    pub fn next_delay(&self, attempt: usize) -> Duration {
        let exp_delay = self.base_delay * 2usize.pow(attempt as u32);
        let delay = exp_delay.min(self.max_delay);
        let jitter = delay * self.jitter_factor * random();
        delay + jitter
    }
}
```

### When to Retry

Retries are appropriate for:
- Transient network failures
- Temporary worker overload
- Resource exhaustion (memory pressure)

Retries are NOT appropriate for:
- Query syntax errors
- Type mismatches
- Data corruption

## TaskProcessor

Executes physical plans on the CPU thread pool with retry integration.

`octopus-executor/src/task_processor.rs:1`

### Execution Flow

```
1. Receive physical plan from coordinator
2. Spawn execution on CPU runtime
3. Report progress via gRPC
4. Stream results to Exchange operators
5. On failure: invoke RetryHandler
6. On success: report completion
```

## FlightServer

Arrow Flight server for worker data plane.

`octopus-worker/src/flight_server.rs:1`

### Endpoints

- `GetFlightData`: Retrieve data for an exchange partition
- `DoGet`: Stream record batches to receivers

### Server Configuration

```rust
FlightServer::bind("[::]:50052")
    .maxConcurrentStreams(100)
    .build()?
```

## Code References

- `octopus-worker/src/worker_runtime.rs:1` - Runtime configuration
- `octopus-worker/src/worker_service.rs:1` - Worker service
- `octopus-worker/src/metrics_collector.rs:1` - Metrics
- `octopus-executor/src/retry_handler.rs:1` - Retry logic
