---
phase: "04"
phase_name: "Advanced SQL & Federated Sources"
status: "context_captured"
created: "2026-05-12"
---

# Phase 04: Advanced SQL & Federated Sources — Context

## Domain

Users can query PostgreSQL/MySQL via federated connectors and use advanced SQL features (window functions, date/time/string functions, type conversions). The system maintains consistent distributed query execution with DataFusion as the query execution engine.

## Canonical References

- `.planning/ROADMAP.md` — Phase 4 goal and success criteria
- `.planning/REQUIREMENTS.md` — DATA-04, DATA-05, ADV-01, ADV-02, ADV-03
- `.planning/phases/03-workers-arrow-flight/03-CONTEXT.md` — prior decisions

## Decisions

### Federated Connector — Connection Pool
**Decision:** Maintain a connection pool per worker for PostgreSQL/MySQL connections.

**Rationale:** Connection pooling provides better performance for multiple queries against the same database. Each worker maintains its own pool, enabling locality-aware query execution.

### Window Functions — Auto Mode
**Decision:** Let DataFusion decide execution mode (streaming vs bounded) based on partition size.

**Rationale:** Small partitions can use streaming for low latency; large partitions use bounded execution to avoid memory issues. DataFusion's query planner makes this decision based on statistics when available.

### Function Coverage — Built-in + Custom
**Decision:** Use DataFusion built-in functions for standard cases; add Octopus-specific custom implementations for performance-critical or commonly-used functions.

**Rationale:** Built-in functions are well-tested and cover 90% of use cases. Custom implementations (e.g., optimized date_trunc for common intervals) provide performance benefits where DataFusion's generic implementations may be suboptimal.

### Type Mapping — Adapter Pattern
**Decision:** Create a unified type abstraction with per-database adapters for PostgreSQL and MySQL.

**Rationale:** Each database has its own type system (PostgreSQL's UUID, JSONB, ARRAY vs MySQL's DATETIME, JSON). An adapter pattern allows shared code while handling database-specific quirks cleanly.

## Deferred Ideas

- HA-01 / HA-02: Multi-coordinator and failover — Phase 5+ scope
- DATA-06: Iceberg connector — v2 scope
- DATA-07: Delta Lake support — v2 scope

## Notes

- Phase 3 established: Worker service, Arrow Flight data plane, Exchange operators with streaming
- Phase 2 established: Coordinator with QueryScheduler, HTTP server
- Phase 1 established: DataFusion 43, ExecutorSession
- Federated connectors extend the data source model from Phase 1 (Parquet/CSV/JSON files → external databases)
