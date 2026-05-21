---
title: Architecture
type: concept
tags: [system-design,data-flow,streaming]
sources:
  - octopus-coordinator/src/
  - octopus-executor/src/
  - octopus-worker/src/
related:
  - "[[Coordinator]]"
  - "[[Execution]]"
  - "[[Exchange]]"
---

# Architecture

Octopus follows a distributed MPP query engine architecture inspired by Trino's streaming design.

## System Overview

```
┌─────────┐     HTTP      ┌──────────────┐    gRPC    ┌─────────────┐
│  CLI    │──────────────→│ Coordinator  │───────────→│   Workers   │
│ Client  │←──────────────│  (port 50051)│            │ (Arrow Flight)
└─────────┘   Query State └──────────────┘            └─────────────┘
                            │                                    │
                            │         ┌────────────────────────┘
                            │         ↓
                            │   ┌──────────────┐
                            └──→│ QueryService │──→ DataFusion (SQL parsing)
                                └──────────────┘
```

## Data Flow

1. **Client submits SQL** via HTTP POST to `/query/submit` on the coordinator
2. **Coordinator parses SQL** via DataFusion `SessionContext::sql()` → LogicalPlan
3. **StagePlanner creates Stage DAG** with Exchange boundaries at shuffle points
4. **QueryScheduler assigns tasks** to workers based on data locality
5. **Workers execute in parallel**, exchanging data via Arrow Flight
6. **Results streamed back** to coordinator, then to client

## Key Components

### Coordinator (octopus-coordinator)

The coordinator is the central brain:
- HTTP API Server (axum) on port 50051
- QueryService for SQL parsing and state management
- StagePlanner for distributed plan analysis
- QueryScheduler for locality-aware task assignment
- WorkerRegistry for worker registration and heartbeat

See [[Coordinator]] for details.

### Executor (octopus-executor)

Local query execution:
- DataFusion SessionContext for SQL execution
- ExecutorSession wrapping SessionContext with CPU partition config
- TaskProcessor for physical plan execution
- RetryHandler with exponential backoff

See [[Execution]] for details.

### Worker (octopus-worker)

Task execution and data plane:
- FlightServer/FlightHandler for Arrow Flight
- WorkerService coordinating task reception and execution
- MetricsCollector for observability

### Exchange Operators

Exchange operators define stage boundaries. All operators between two Exchanges execute locally on a worker. This is the critical insight enabling Trino-style pipeline execution.

See [[Exchange]] for details.

## Exchange Operator Pattern

```
┌─────────────────────────────────────────────────────────┐
│ Stage 1: Scan → Filter → Project                       │
│                      │                                  │
│                      ↓ (Exchange - Hash)                │
│ ┌────────────────────────────────────────────────────┐ │
│ │ Stage 2: Hash Join → Aggregate                    │ │
│ │                      │                             │ │
│ │                      ↓ (Exchange - Broadcast)       │ │
│ │ ┌──────────────────────────────────────────────┐  │ │
│ │ │ Stage 3: Output                              │  │ │
│ │ └──────────────────────────────────────────────┘  │ │
│ └────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────┘
```

Stages execute concurrently rather than sequentially, maximizing pipeline parallelism.

## gRPC Protocol

Worker coordination uses gRPC defined in `proto/worker.proto`:
- `ExecuteTask` - dispatch task to worker
- `GetExchangeData` - pull data between workers
- `Health` - worker health check

## Arrow Flight Data Plane

Worker-to-worker data transfer uses Arrow Flight over gRPC:
- Worker-pull model (receiver requests data from sender)
- Efficient columnar format via Arrow
- Batched transfer for streaming
