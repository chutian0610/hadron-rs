# Roadmap: Octopus

## Milestones

- ✅ **v1.0 MVP** — Phases 1-5 (shipped 2026-05-13)
- ✅ **v1.1** — 去除 JDBC 实现 (shipped 2026-05-21)
- ⏳ **v1.2** — 测试覆盖与代码注释 (current)

## Progress

| Phase | Milestone | Plans | Status | Completed |
|-------|-----------|-------|--------|-----------|
| 1 | v1.0 | 3/3 | Complete | 2026-05-07 |
| 2 | v1.0 | 3/3 | Complete | 2026-05-11 |
| 3 | v1.0 | 4/4 | Complete | 2026-05-12 |
| 4 | v1.0 | 4/4 | Complete | 2026-05-12 |
| 5 | v1.0 | 2/2 | Complete | 2026-05-13 |
| 2 | v1.2 | 3/3 | Complete | 2026-05-21 | |
| 3 | v1.2 | 5/5 | Planned | — |
| 4 | v1.2 | 0/2 | Not started | — |
| 5 | v1.2 | 0/1 | Not started | — |

---

*v1.0 details archived to `.planning/milestones/v1.0-MVP.md`*

---

*v1.1 details archived to `.planning/milestones/v1.1-去除-JDBC-实现-ROADMAP.md`*

---

## Phase Details

### Phase 2: Test Infrastructure
**Goal:** Set up test utilities and mock infrastructure needed for all subsequent testing
**Depends on:** Phase 1 (v1.1 completed)
**Requirements:** None (foundational - enables all testing requirements)

**Success Criteria** (what must be TRUE):
1. mockall, tempfile, tokio-test configured in Cargo.toml dev-dependencies
2. MockWorkerRegistry available for scheduler tests
3. TestQueryContext with pre-configured SessionContext available
4. TestRecordBatchFactory for Arrow test data available
5. InMemoryConnectionPool for federated connector tests available

**Plans:** 3 plans in 1 wave
- [x] 02-01-PLAN.md - Configure workspace dev-dependencies and test-utils feature gate
- [x] 02-02-PLAN.md - Create test_utils module with MockWorkerRegistry, TestRecordBatchFactory, async helpers
- [x] 02-03-PLAN.md - Create InMemoryConnectionPool and TestQueryContext

---

### Phase 3: Unit Tests
**Goal:** Comprehensive unit test coverage for all core components
**Depends on:** Phase 2
**Requirements:** COORD-01, COORD-02, COORD-03, EXEC-01, EXEC-02, WORKER-01, WORKER-02, COMMON-01, COMMON-02, COMMON-03

**Success Criteria** (what must be TRUE):
1. QueryScheduler unit tests pass (task scheduling, partition scoring, locality-aware distribution)
2. QueryService state machine tests pass (submit, running, complete, failed states)
3. HTTP API endpoint tests pass using tower-test/TestClient
4. ExecutorSession unit tests pass (query execution, error handling)
5. Exchange operator stream tests pass (sender/receiver, backpressure)
6. Worker service task execution tests pass
7. Arrow Flight endpoint tests pass
8. UDF registry tests pass with expanded coverage
9. Federated connector trait tests pass with mockall
10. OctopusError error path tests pass

**Plans:** 5 plans in 3 waves
- [ ] 03-01-PLAN.md — QueryScheduler and QueryService unit tests (COORD-01, COORD-02)
- [ ] 03-02-PLAN.md — HTTP API endpoint tests with tower-test (COORD-03)
- [ ] 03-03-PLAN.md — ExecutorSession and ExchangeOperator tests (EXEC-01, EXEC-02)
- [ ] 03-04-PLAN.md — Common crate tests: UDF, FederatedConnector, OctopusError (COMMON-01, COMMON-02, COMMON-03)
- [ ] 03-05-PLAN.md — Worker crate tests: WorkerService, FlightHandler (WORKER-01, WORKER-02)

---

### Phase 4: Integration Tests
**Goal:** End-to-end integration tests verifying component interactions
**Depends on:** Phase 3
**Requirements:** INTEG-01, INTEG-02

**Success Criteria** (what must be TRUE):
1. End-to-end query execution test passes (coordinator to worker to response)
2. CLI integration tests pass using assert_cmd

**Plans:** TBD

---

### Phase 5: Documentation Pass
**Goal:** Public APIs documented with Rustdoc
**Depends on:** Phase 4
**Requirements:** DOC-01, DOC-02, DOC-03, DOC-04

**Success Criteria** (what must be TRUE):
1. Public API Rustdoc complete for coordinator crate (query_service.rs, scheduler.rs)
2. Public API Rustdoc complete for executor crate
3. Public API Rustdoc complete for worker crate
4. Module-level documentation complete for common crate
5. cargo test --doc passes without warnings

**Plans:** TBD

---

## v1.2 Traceability

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

**v1.2 Coverage:** 16/16 requirements mapped ✓

---

*Last updated: 2026-05-22 after Phase 3 planning*