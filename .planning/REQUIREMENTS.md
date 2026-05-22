# Requirements: Octopus v1.2

**Defined:** 2026-05-21
**Milestone:** 测试覆盖与代码注释
**Core Value:** Users can run fast interactive SQL queries on large distributed datasets with Rust-level performance and memory safety.

## Testing Requirements

### Coordinator Tests

- [ ] **COORD-01**: QueryScheduler unit tests (task scheduling logic, partition scoring, locality-aware distribution)
- [ ] **COORD-02**: QueryService state machine tests (submit, running, complete, failed states)
- [ ] **COORD-03**: HTTP API endpoint tests using tower-test/TestClient

### Executor Tests

- [ ] **EXEC-01**: ExecutorSession unit tests (query execution, error handling)
- [ ] **EXEC-02**: Exchange operator stream tests (sender/receiver, backpressure)

### Worker Tests

- [ ] **WORKER-01**: Worker service task execution tests
- [ ] **WORKER-02**: Arrow Flight endpoint tests

### Common Tests

- [ ] **COMMON-01**: UDF registry tests (existing, expand coverage)
- [ ] **COMMON-02**: Federated connector trait tests with mockall
- [ ] **COMMON-03**: OctopusError error path tests

### Integration Tests

- [ ] **INTEG-01**: End-to-end query execution test (coordinator to worker to response)
- [ ] **INTEG-02**: CLI integration tests using assert_cmd

## Documentation Requirements

### Code Documentation

- [ ] **DOC-01**: Public API Rustdoc for coordinator crate (query_service.rs, scheduler.rs)
- [ ] **DOC-02**: Public API Rustdoc for executor crate
- [ ] **DOC-03**: Public API Rustdoc for worker crate
- [ ] **DOC-04**: Module-level documentation for common crate

## Out of Scope

| Feature | Reason |
|---------|--------|
| Testcontainers for real DBs | Too heavy, use mocks instead |
| Golden file tests | Query plan output varies by DataFusion version |
| Snapshot testing | Adds maintenance burden without proportional value |
| Performance benchmarking | Defer to separate perf milestone |

## Traceability

| Requirement | Phase | Status |
|-------------|-------|--------|
| COORD-01 | Phase 3 | Pending |
| COORD-02 | Phase 3 | Pending |
| COORD-03 | Phase 3 | Pending |
| EXEC-01 | Phase 3 | Pending |
| EXEC-02 | Phase 3 | Pending |
| WORKER-01 | Phase 3 | Pending |
| WORKER-02 | Phase 3 | Pending |
| COMMON-01 | Phase 3 | Pending |
| COMMON-02 | Phase 3 | Pending |
| COMMON-03 | Phase 3 | Pending |
| INTEG-01 | Phase 4 | Pending |
| INTEG-02 | Phase 4 | Pending |
| DOC-01 | Phase 5 | Pending |
| DOC-02 | Phase 5 | Pending |
| DOC-03 | Phase 5 | Pending |
| DOC-04 | Phase 5 | Pending |

**Coverage:**
- v1.2 requirements: 16 total
- Mapped to phases: 16 ✓
- Unmapped: 0

---
*Requirements defined: 2026-05-21*
*Last updated: 2026-05-21 after v1.2 roadmap created*