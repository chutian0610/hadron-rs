---
title: Coordinator
type: concept
tags: [HTTP-API,QueryService,StagePlanner,TaskScheduler,WorkerRegistry]
sources:
  - octopus-coordinator/src/
related:
  - "[[Architecture]]"
  - "[[Execution]]"
  - "[[Exchange]]"
---

# Coordinator

The coordinator (`octopus-coordinator`) is the central brain of Octopus, responsible for SQL parsing, distributed planning, task scheduling, and serving the HTTP API.

## HTTP API Server

The coordinator runs an axum HTTP server on port 50051 with the following endpoints:

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/query/submit` | POST | Submit SQL query, returns query_id |
| `/query/explain` | POST | Parse SQL, return formatted logical plan |
| `/query/state/:query_id` | GET | Get query state and progress |

### Submit Query Request

```json
{
  "sql": "SELECT * FROM orders WHERE amount > 100"
}
```

### Submit Query Response

```json
{
  "query_id": "550e8400-e29b-41d4-a716-446655440000",
  "state": "RUNNING"
}
```

## CoordinatorServer

`CoordinatorServer` is the main facade that coordinates:
- `WorkerRegistry` - worker registration and heartbeat
- `QueryScheduler` - locality-aware task assignment
- `QueryService` - SQL parsing and query state management

`src/coordinator_server.rs:1`

## WorkerRegistry

In-memory worker registration with heartbeat tracking and partition locality information.

### Key Responsibilities

- **Registration**: Workers register via gRPC with their host:port and partition info
- **Heartbeat**: Workers send periodic heartbeats; stale workers are removed
- **Partition Locality**: Tracks which partitions are local to which workers for locality-aware scheduling

`src/worker_registry.rs:1`

## QueryService

SQL parsing via DataFusion `SessionContext::sql()` and query state management.

### Key Responsibilities

- **SQL Parsing**: Uses DataFusion's built-in SQL planner via `SessionContext::sql()`
- **Query State**: Tracks query lifecycle (PENDING, RUNNING, COMPLETED, FAILED)
- **UDF Registry Integration**: Registers custom scalar functions for use in SQL

`src/query_service.rs:1`

## StagePlanner

Analyzes DataFusion `ExecutionPlan` to split at exchange boundaries and create a Stage DAG.

### Key Responsibilities

- **Exchange Detection**: Identifies exchange boundaries in the plan
- **DAG Validation**: Ensures no cyclic dependencies between stages (Pitfall 5 mitigation)
- **Pipeline Breaker Detection**: Marks operators that must materialize all input (e.g., full sort)

`src/stage_planner.rs:1`

### Exchange Modes

StagePlanner recognizes these exchange modes defined in ExchangeOperator:
- `Single` - Single sender (for small results)
- `Hash` - Hash-based partitioning
- `RoundRobin` - Round-robin distribution
- `Broadcast` - Send to all workers (for small lookup tables)

## QueryScheduler

Locality-aware task assignment with round-robin fallback.

### Key Responsibilities

- **Partition Scoring**: Scores workers based on partition locality (prefer local data)
- **Task Assignment**: Assigns tasks to workers considering both locality and load
- **Round-Robin Fallback**: When locality is equal, distributes tasks round-robin

`src/query_scheduler.rs:1`

### Partition Locality Scoring

```
score = local_partitions * LOCALITY_WEIGHT + available_capacity * CAPACITY_WEIGHT
```

This implements Pitfall 7 mitigation for load imbalance.

## TaskTracker

Tracks task state, manages retries, and makes rescheduling decisions.

### Key Responsibilities

- **Task State**: Tracks each task (PENDING, RUNNING, COMPLETED, FAILED)
- **Retry Management**: Determines when to retry failed tasks
- **Rescheduling**: Decides when to reschedule tasks from failed workers

`src/task_tracker.rs:1`

## UDF Registry Integration

Custom scalar functions are registered via `UdfRegistry` trait in `octopus-common`. QueryService integrates with the UDF registry to make custom functions available in SQL queries.

See [[UDF Registry]] for details.
