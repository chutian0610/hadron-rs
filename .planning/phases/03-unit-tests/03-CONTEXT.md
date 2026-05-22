---
phase: 03
subsystem: unit-tests
tags: [test-infrastructure, mocking, fixtures]
dependency_graph:
  requires: ["02-Test-Infrastructure"]
  provides: ["query-scheduler-tests", "query-service-tests", "exchange-operator-tests", "executor-session-tests"]
  affects: [octopus-coordinator, octopus-executor]
tech_stack:
  testing: [mockall, tower-test, TestRecordBatchFactory]
  patterns: [mock-worker-registry, synthetic-fixtures]
canonical_refs:
  - .planning/ROADMAP.md
  - .planning/REQUIREMENTS.md
  - .planning/phases/02-Test-Infrastructure/02-CONTEXT.md
decisions:
  - id: TEST-FIXTURE-01
    area: Synthetic fixtures
    decision: Minimal constants for relational test data
    rationale: |
      Unit tests prioritize readability and traceability. Fixed constants let anyone
      reading the test immediately understand what data is being verified. Generative
      factories (TestRecordBatchFactory from Phase 2) are used for boundary/variant
      data generation only — not as the primary fixture source.
    applies_to:
      - octopus-coordinator query tests
      - octopus-executor query tests
    established: "Phase 3 discussion"
  - id: TEST-MOCK-01
    area: Realistic mocks
    decision: Mock workers/Flight only; real DataFusion context
    rationale: |
      Phase 3 focuses on scheduler logic (partition locality scoring, round-robin
      fallback). MockWorkerRegistry from Phase 2 gives fine-grained control over
      worker state for algorithm verification. Arrow Flight is a transport layer —
      mocking it isolates the exchange operator logic. Real SessionContext provides
      complete SQL parsing and execution without external dependencies.
    applies_to:
      - octopus-coordinator/scheduler.rs tests
      - octopus-coordinator/exchange_operator.rs tests
      - octopus-coordinator/query_service.rs tests
    established: "Phase 3 discussion"
  - id: TEST-REAL-01
    area: Maximal real
    decision: Real DataFusion + in-memory; no real files or external DBs
    rationale: |
      In-memory DataFusion with synthetic fixtures provides complete query execution
      capability (parsing, planning, execution, result materialization) without
      I/O overhead or external dependencies. This gives the highest confidence in
      core algorithm correctness while keeping tests fast and reliable.
    applies_to:
      - octopus-executor/session.rs tests
      - octopus-executor/query.rs tests
      - octopus-coordinator QueryService integration
    established: "Phase 3 discussion"
deferred:
  - idea: "External fixture files for complex multi-table schemas"
    phase: "Future phase if schema complexity grows"
    reason: "Current schemas are simple enough for in-code constants"
code_context:
  reusable_assets:
    - name: MockWorkerRegistry
      path: octopus-common/src/test_utils/mock.rs
      purpose: Controllable worker list for scheduler tests
    - name: TestRecordBatchFactory
      path: octopus-common/src/test_utils/fixture.rs
      purpose: Arrow RecordBatch generation for executor tests
    - name: async_with_timeout / async_with_timeout_or_panic
      path: octopus-common/src/test_utils/timeout.rs
      purpose: Prevent hanging tests
  patterns:
    - scheduler tests use MockWorkerRegistry with predictable UUIDs (counter-based)
    - QueryService tests use tower-test/TestClient for HTTP endpoint testing
    - Exchange operator tests mock Flight client, use real RecordBatch
    - TestQueryContext from octopus-coordinator/src/test_utils.rs for SessionContext setup
---

# Phase 3: Unit Tests Context

## Domain

Comprehensive unit test coverage for all core Octopus components. Phase 3 enables
all subsequent testing (Phase 4 integration, Phase 5 documentation).

## Decisions

### Synthetic fixtures — TEST-FIXTURE-01

**Decision:** Minimal constants for relational test data

**Rationale:** Unit tests prioritize readability and traceability. Fixed constants let
anyone reading the test immediately understand what data is being verified. Generative
factories (TestRecordBatchFactory from Phase 2) are used for boundary/variant data
generation only — not as the primary fixture source.

**Applies to:** octopus-coordinator query tests, octopus-executor query tests

### Realistic mocks — TEST-MOCK-01

**Decision:** Mock workers/Flight only; real DataFusion context

**Rationale:** Phase 3 focuses on scheduler logic (partition locality scoring, round-robin
fallback). MockWorkerRegistry from Phase 2 gives fine-grained control over worker state
for algorithm verification. Arrow Flight is a transport layer — mocking it isolates the
exchange operator logic. Real SessionContext provides complete SQL parsing and execution
without external dependencies.

**Applies to:** octopus-coordinator/scheduler.rs, octopus-coordinator/exchange_operator.rs,
octopus-coordinator/query_service.rs

### Maximal real — TEST-REAL-01

**Decision:** Real DataFusion + in-memory; no real files or external DBs

**Rationale:** In-memory DataFusion with synthetic fixtures provides complete query execution
capability (parsing, planning, execution, result materialization) without I/O overhead or
external dependencies. This gives the highest confidence in core algorithm correctness while
keeping tests fast and reliable.

**Applies to:** octopus-executor/session.rs, octopus-executor/query.rs,
octopus-coordinator QueryService integration

## Scope

**In scope (from ROADMAP.md):**
1. QueryScheduler unit tests (task scheduling, partition scoring, locality-aware distribution)
2. QueryService state machine tests (submit, running, complete, failed states)
3. HTTP API endpoint tests using tower-test/TestClient
4. ExecutorSession unit tests (query execution, error handling)
5. Exchange operator stream tests (sender/receiver, backpressure)
6. Worker service task execution tests
7. UDF registry tests with expanded coverage
8. Federated connector trait tests with mockall
9. OctopusError error path tests

**Not in scope:**
- End-to-end query execution (Phase 4)
- Arrow Flight endpoint tests against real worker process
- Database connector integration (Phase 4)

## Test Data

**Relational fixtures (constants):**
- `orders(id, amount, date)` — small fixed dataset
- `users(id, name, value)` — small fixed dataset
- `partitions(partition_id, table_name, file_path)` — locality test data

**Generative variants (TestRecordBatchFactory):**
- Mismatched-length columns (error case)
- Empty batches (edge case)
- Large batches (stress case)

## Noted for Later

- External fixture files (YAML/JSON) if schema complexity grows beyond constants
- Integration tests for MySQL/Postgres connectors (Phase 4)
- Real Parquet file tests if partition pruning becomes a focus