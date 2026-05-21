---
title: Execution
type: concept
tags: [DataFusion,QueryExecutor,ExecutorSession,TaskProcessor]
sources:
  - octopus-executor/src/
related:
  - "[[Architecture]]"
  - "[[Coordinator]]"
  - "[[Worker Runtime]]"
---

# Execution

The executor (`octopus-executor`) handles local query execution using Apache DataFusion's `SessionContext`.

## QueryExecutor

Main entry point for local query execution. Wraps DataFusion's `SessionContext` with Octopus-specific configuration.

`src/query_executor.rs:1`

### Key Responsibilities

- **SQL Execution**: Parses and executes SQL via DataFusion
- **Physical Plan Execution**: Executes DataFusion physical plans locally
- **Session Management**: Manages query sessions with configuration

## ExecutorSession

Wrapper around DataFusion `SessionContext` with CPU partition configuration.

`src/executor_session.rs:1`

### Configuration

- **CPU Partitions**: Number of threads for CPU-intensive operators
- **Memory Configuration**: Memory limits for operators and spilling

## TaskProcessor

Executes DataFusion physical plans on the CPU thread pool with retry support.

`src/task_processor.rs:1`

### Execution Flow

1. Receive physical plan fragment from coordinator
2. Execute on CPU thread pool
3. Handle operator errors with retry logic
4. Stream results via Arrow Flight

### CPU Thread Pool

TaskProcessor uses a dedicated CPU thread pool separate from the Tokio runtime used for I/O. This addresses Pitfall 2 (Tokio Runtime Contention) by separating CPU-intensive compute from I/O-bound work.

## DataSourceRegistrar

Registers Parquet, CSV, and JSON data sources with DataFusion.

`src/datasource_registrar.rs:1`

### Supported Formats

- **Parquet**: Columnar format with predicate pushdown
- **CSV**: Text format with header support
- **JSON**: JSON Lines format

### Registration

```rust
ctx.register_parquet("orders", "s3://bucket/orders.parquet")?;
ctx.register_csv("users", "s3://bucket/users.csv")?;
```

## QueryTrace

Structured query logging with UUID-based correlation for distributed tracing.

`src/query_trace.rs:1`

### Fields

- `query_id`: UUID for correlating all events
- `stage_id`: Current stage identifier
- `task_id`: Current task identifier
- `timestamp`: Event timestamp
- `event`: Event type (START, COMPLETE, FAIL, etc.)

## RetryHandler

Exponential backoff with jitter for task retry.

`src/retry_handler.rs:1`

### Retry Policy

```
delay = min(base * 2^attempt + jitter, max_delay)
```

- `base`: Initial delay (e.g., 100ms)
- `max_delay`: Maximum delay cap (e.g., 30s)
- `jitter`: Random variance to prevent thundering herd

See [[Worker Runtime]] for retry handling in the worker context.
