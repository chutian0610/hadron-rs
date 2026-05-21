---
phase: "05"
phase_name: "Observability & Clients"
status: "context_captured"
created: "2026-05-13"
---

# Phase 05: Observability & Clients — Context

## Domain

Full observability with metrics, logging, and client interfaces (JDBC, UDF/UDTF). This is the final phase completing the v1 milestone.

## Canonical References

- `.planning/ROADMAP.md` — Phase 5 goal and success criteria
- `.planning/REQUIREMENTS.md` — ADV-04, OBS-02, CLI-02
- `.planning/phases/04-advanced-sql-federated-sources/04-CONTEXT.md`
- `.planning/phases/03-workers-arrow-flight/03-CONTEXT.md`

## Decisions

### Scope — Expanded
**Decision:** Include UDF/UDTF + JDBC driver + observability additions beyond minimal scope.

**Rationale:** User chose expanded scope for the final phase.

### Metrics (OBS-02)
**Decision:** Already implemented in Phase 3 via MetricsCollector in octopus-worker.

**Note:** OBS-02 requirement is already satisfied. Phase 5 focuses on UDF/UDTF and JDBC.

### JDBC Driver
**Decision:** Implement JDBC driver for BI tool connectivity.

**Rationale:** CLI-02 requires JDBC driver to connect from BI tools.

### UDF/UDTF
**Decision:** Implement UDF/UDTF registration and execution.

**Rationale:** ADV-04 requires user-defined functions for custom transformations.

## Deferred Ideas

- HA-01 / HA-02: Multi-coordinator and failover — v2 scope
- CLI-03: HTTP API — v2 scope

## Notes

- Phase 4 established: PostgreSQL and MySQL federated connectors
- Phase 3 established: Worker service, Arrow Flight, metrics collection
- Phase 2 established: Coordinator with HTTP server on :50051
