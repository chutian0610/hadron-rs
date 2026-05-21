# Research Summary: Octopus v1.2 - Testing & Documentation

**Project:** Octopus Distributed MPP Query Engine
**Milestone:** v1.2 - Test Coverage & Code Comments
**Research Date:** 2026-05-21
**Synthesized From:** STACK.md, FEATURES.md, ARCHITECTURE.md, PITFALLS.md

---

## Executive Summary

Octopus v1.2 adds comprehensive unit tests, integration tests, and Rustdoc documentation to the existing distributed MPP query engine. The existing Tokio 1.52 runtime and DataFusion 43 stack provide a solid foundation requiring no major technology changes. The focus is on test utilities (mockall for trait mocking, tempfile for file operations, tower-test for HTTP/gRPC service testing) that integrate cleanly with the existing async/Tokio/DataFusion/Axum stack.

The project currently has **zero tests** despite #[cfg(test)] infrastructure in place. The testing surface is well-defined: scheduler logic, query service state machine, UDF registry operations, federated connector traits, and exchange sender/receiver pairs. Complexity lies not in test patterns but in mocking async services and Arrow Flight data flows.

Critical risks include blocking the Tokio runtime with sync I/O, shared mutable SessionContext state across parallel tests, deadlocks in multi-stage async pipelines, and over-mocking DataFusion internals. Prevention strategies are well-documented in PITFALLS.md and should guide implementation.

---

## Key Findings

### From STACK.md

**Core Test Framework:**
- tokio 1.52 and tokio-test 0.4 already in workspace
- #[tokio::test] macro is the standard for async Rust testing

**Mocking Infrastructure:**
- mockall 0.13 for trait mocking with native async trait support
- #[automock] attribute generates Mock* structs from traits
- Supports async_trait::async_trait interfaces (UdfRegistry, FederatedConnector)

**Integration Testing:**
- tower-test 0.4 for HTTP/gRPC service integration testing
- portpicker 0.1 for ephemeral port allocation
- testcontainers-rs 15 considered but deferred (Docker dependency)

**Test Utilities:**
- tempfile 3 for RAII temporary files (Parquet/CSV operations)
- assert_cmd 2 + predicates 3 for CLI binary testing

**Code Documentation:**
- rustdoc built into cargo (no additional dependencies)
- cargo-deadlinks for broken doc link checking
- cargo-readme for README synchronization

**What NOT to Add:** rstest, proptest, fake, criterion, cargo-expand, trybuild (overkill or out-of-scope for v1.2)

---

### From FEATURES.md

**Table Stakes (Must Have):**
- Unit tests in #[cfg(test)] modules co-located with source
- #[test] and #[tokio::test] attributes
- Integration tests in tests/ directory
- #[should_panic] for expected panic verification
- assert_eq!/assert_ne! with PartialEq + Debug derives
- #[automock] for trait mocking
- #[ignore] for slow/flaky tests

**Differentiators (Recommended for v1.3+):**
- Property-based testing (proptest) for critical algorithms
- Code coverage tracking (cargo-llvm-cov)
- Criterion benchmarks for hot paths
- Doc tests in Rustdoc comments

**Anti-Features (Explicitly NOT Building):**
- testcontainers (heavy Docker dependency)
- Golden file tests (fragile, plan text changes with DataFusion versions)
- Load/stress testing (requires multi-node cluster)
- Fuzz testing (SQL surface not yet stable)

**Octopus Crate Test Surface:**
- octopus-common: UDF registry, federated connector traits, error types
- octopus-coordinator: scheduler, query_service state machine, stage_planner, HTTP handlers
- octopus-executor: query execution, exchange sender/receiver
- octopus-worker: task execution, flight server
- octopus-cli: argument parsing, REPL mode

---

### From ARCHITECTURE.md

**Integration Points for Tests:**
1. DataFusion SessionContext (query_service.rs:93-96) - needs test harness with known schemas/UDFs
2. WorkerRegistry (scheduler.rs:25) - needs controllable mock with static worker list
3. Tokio async runtime - #[tokio::test] infrastructure already present
4. Arrow Flight data plane - needs mock endpoints returning predefined data
5. Federated connectors - needs in-memory fake connectors

**Current State Assessment:**
- octopus-common: 4 source files, tests exist (udf.rs only)
- octopus-coordinator: 11 source files, no tests
- octopus-executor: 8 source files, no tests
- octopus-worker: 9 source files, no tests

**Test Organization:**
- Unit tests: #[cfg(test)] modules in source files
- Integration tests: tests/ directory per crate
- Priority test utilities: MockWorkerRegistry, TestQueryContext, InMemoryConnectionPool, TestRecordBatchFactory

**Recommended Patterns:**
1. Trait-based mocking via #[automock]
2. Constructor injection for test doubles
3. Async test infrastructure with #[tokio::test]
4. Channel-based component testing with controlled channels

---

### From PITFALLS.md

**Critical Pitfalls:**

1. **Blocking the Tokio Runtime** - Tests hang indefinitely when using std::thread::sleep, std::fs::read, or blocking mutex locks in async context. Prevention: Use tokio::time::sleep, tokio::fs::read, async-aware synchronization.

2. **Shared Mutable State Across Parallel Tests** - Tests pass individually but fail together when reusing SessionContext or global state. Prevention: Fresh state per test; never share mutable context.

3. **Deadlocks in Multi-Stage Async Test Scenarios** - Tests hang forever due to cyclic dependencies or missing .await. Prevention: Wrap all .await in timeouts; test Exchange operators with explicit timeouts.

4. **Integration Tests Without Proper Resource Cleanup** - Port conflicts, file handle leaks on CI. Prevention: Use tempfile for automatic cleanup, portpicker for unique ports, proper shutdown signals.

5. **Over-Mocking Internal Dependencies** - Tests pass but code integration fails when mock behavior diverges from real implementation. Prevention: Test at highest level possible; use real RecordBatch::try_from_iter; mock only at system boundaries.

**Moderate Pitfalls:**

6. Missing error path coverage - every ? needs corresponding error test
7. Doc tests that fail after API changes - run cargo test --doc in CI
8. Slow test suite accumulation - keep unit tests fast, mark slow tests with #[ignore]
9. Environment-dependent tests - provide defaults that work in CI
10. Send + Sync trait bounds not verified - add compile-fail tests for thread-safe types

---

## Implications for Roadmap

### Recommended Phase Structure

**Phase 1: Test Infrastructure Foundations**
- Rationale: All subsequent testing depends on having mock utilities and test data factories
- Deliverables:
  - Add mockall, tempfile, tokio-test to Cargo.toml dev-dependencies
  - Create MockWorkerRegistry for scheduler tests
  - Create TestQueryContext with pre-configured SessionContext
  - Create TestRecordBatchFactory for Arrow test data
  - Create InMemoryConnectionPool for connector tests
- Pitfalls to avoid: PITFALL 4 (resource cleanup), PITFALL 5 (over-mocking)

**Phase 2: Unit Tests for Core Components**
- Rationale: Critical paths need coverage before integration testing; pure logic tests are fastest to write
- Deliverables:
  - octopus-common: UDF registry tests, error type tests
  - octopus-coordinator: scheduler tests (locality scoring, round-robin fallback), query_service state machine tests
  - octopus-executor: query execution basics with temp Parquet files
- Pitfalls to avoid: PITFALL 1 (blocking), PITFALL 2 (shared state), PITFALL 3 (deadlocks)

**Phase 3: Integration Tests**
- Rationale: Components must work together before documentation; HTTP API must be stable
- Deliverables:
  - HTTP API integration tests for coordinator endpoints (submit, explain, state)
  - Arrow Flight endpoint tests for executor
  - Federated connector mock tests with in-memory fake DB
  - Exchange sender/receiver channel tests
- Pitfalls to avoid: PITFALL 4 (resource leaks), PITFALL 6 (missing error paths)

**Phase 4: Documentation**
- Rationale: Final phase; public APIs must be stable before documenting
- Deliverables:
  - Rustdoc for all pub struct, pub enum, pub fn in coordinator and executor
  - Module-level //! docs for each source file
  - Usage examples in test utilities
  - Run cargo test --doc validation
- Pitfalls to avoid: PITFALL 7 (stale doc tests)

---

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Stack | HIGH | All recommended tools are established Rust ecosystem libraries |
| Features | MEDIUM-HIGH | Standard Rust patterns well-documented; Octopus modules confirmed via code inspection |
| Architecture | MEDIUM | Integration points identified via code inspection; specific mocking patterns need spike |
| Pitfalls | MEDIUM-HIGH | Async testing patterns well-documented; specific Octopus failure modes inferred from architecture |

### Research Gaps to Address During Implementation

1. **gRPC/Flight service mocking**: Specific patterns for testing tonic services and Arrow Flight servers need spike during Phase 3
2. **DataFusion test helpers**: How to set up SessionContext for testing without hitting production object stores
3. **Test HTTP server setup**: axum::test::TestClient vs tower MockSvc vs real server on random port
4. **Mock coordinator client**: Need #[automock] trait for CoordinatorClient to test worker in isolation

---

## Sources

### STACK.md Sources
- tokio test macro - https://docs.rs/tokio/latest/tokio/attr.test.html
- mockall documentation - https://docs.rs/mockall/latest/mockall/
- tower-test - https://docs.rs/tower-test/latest/tower_test/
- tempfile - https://docs.rs/tempfile/latest/tempfile/
- assert_cmd - https://docs.rs/assert_cmd/latest/assert_cmd/
- portpicker - https://docs.rs/portpicker/latest/portpicker/

### FEATURES.md Sources
- The Rust Programming Language - Testing Chapter - HIGH confidence
- Tokio Testing Documentation - HIGH confidence
- Mockall Documentation - HIGH confidence
- tokio-test crate - MEDIUM confidence
- proptest crate - MEDIUM confidence
- axum testing guide - MEDIUM confidence
- DataFusion/Ballista test patterns - LOW confidence (inferential)

### ARCHITECTURE.md Sources
- Tokio testing: tokio-test documentation - HIGH confidence
- DataFusion testing: Integration with SessionContext - HIGH confidence
- Rust async testing patterns: Standard practice - HIGH confidence

### PITFALLS.md Sources
- Tokio: Testing async code - MEDIUM confidence
- The Rust Programming Language: Testing - HIGH confidence
- DataFusion testing utilities - MEDIUM confidence
- tokio::test documentation - HIGH confidence

---

## Verification Checklist

- [ ] All async tests use #[tokio::test]
- [ ] No blocking calls (std::thread, std::fs) in async test context
- [ ] Each test creates fresh state (no shared SessionContext)
- [ ] All .await points have timeout protection
- [ ] Error paths tested for every ? operator
- [ ] cargo test --doc passes
- [ ] Integration tests in tests/ directory
- [ ] Slow tests marked with #[ignore] and documented
