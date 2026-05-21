# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build Commands

```bash
cargo build --workspace              # Build all crates
cargo test -p octopus-coordinator   # Test specific crate
cargo test -p octopus-executor      # Test executor crate
cargo run -p octopus-cli -- --help # Run CLI
cargo run -p octopus-coordinator   # Run coordinator (starts HTTP server on :50051)
```

## Architecture

Octopus is a distributed MPP query engine following **Trino-style streaming architecture**:

```
Client → Coordinator → Workers → Arrow Flight (data plane)
                     ↑
               gRPC (control plane)
```

**Key principle:** Exchange operators are the **only points where data crosses worker boundaries**. Everything else stays local and streaming.

### Crates

- `octopus-coordinator` — Central brain: SQL parsing, distributed planning, task scheduling, HTTP API
- `octopus-executor` — Local query execution using DataFusion SessionContext
- `octopus-worker` — Task execution + Arrow Flight server (Phase 3)
- `octopus-cli` — Client CLI with local/interactive/batch modes
- `octopus-common` — Shared utilities

### Data Flow

1. Client submits SQL via HTTP to coordinator
2. Coordinator parses SQL via DataFusion `SessionContext::sql()` → LogicalPlan
3. Coordinator creates Stage DAG with Exchange boundaries
4. Coordinator schedules tasks to workers based on data locality
5. Workers execute in parallel, exchange data via Arrow Flight
6. Results streamed back to coordinator

## Critical Patterns

### Pipeline Streaming
The coordinator uses `QueryService` with DataFusion `SessionContext` for SQL parsing. `QueryScheduler` uses locality-aware task assignment (partition scoring). The CLI connects via HTTP using `CoordinatorClient`.

### HTTP Server (Coordinator)
Coordinator runs an axum HTTP server on port 50051 with endpoints:
- `POST /query/submit` — submit query, returns query_id
- `POST /query/explain` — parse SQL and return formatted plan
- `GET /query/state/{id}` — get query state

### Exchange Operator Pattern
Exchange operators define stage boundaries in distributed query plans. All operators between two Exchanges execute locally on a worker. This enables Trino-style pipeline execution where stages run concurrently rather than sequentially.

## Key Pitfalls (from PITFALLS.md)

**Pitfall 1: Pipeline Breakers** — Operators like full sort or hash agg with many groups must materialize all input, breaking streaming. Mark operators with `unbounded_output()` correctly.

**Pitfall 2: Tokio Runtime Contention** — DataFusion uses the same Tokio runtime for CPU-intensive compute AND network I/O (S3 reads, Arrow Flight). Separate runtimes for CPU vs IO-bound work. Phase 3 will address this.

**Pitfall 5: Exchange Deadlock** — Multi-stage pipelines can deadlock when cyclic dependencies form through Exchange operators. Implement DAG validation before execution.

**Pitfall 7: Load Imbalance** — Round-robin task distribution doesn't account for partition size variance or worker heterogeneity. `QueryScheduler` now uses partition locality scoring with round-robin fallback.

## DataFusion Version

Currently using **DataFusion 43** (API-stable). All SQL execution uses `SessionContext::sql()` which parses SQL into LogicalPlan via DataFusion's built-in SQL planner.

## Wiki

The project maintains an auto-generated wiki in `wiki/`. The index is at `wiki/llms.txt`. When making significant changes to the codebase, update the wiki by running the `/autowiki:update` skill.
