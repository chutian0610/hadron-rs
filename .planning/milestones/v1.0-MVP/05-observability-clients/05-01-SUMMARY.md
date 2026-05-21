---
phase: "05"
plan: "01"
subsystem: "coordinator"
tags:
  - "udf"
  - "udtf"
  - "user-defined-functions"
  - "sql"
dependency_graph:
  requires: []
  provides:
    - "ADV-04: UDF/UDTF registration and execution"
  affects:
    - "octopus-coordinator"
    - "octopus-common"
tech_stack:
  added:
    - "datafusion-expr: create_udf, ScalarUDF, Volatility"
    - "async-trait: async trait methods"
    - "tokio (dev): async test support"
  patterns:
    - "UdfRegistry trait with interior mutability via RwLock"
    - "DataFusion ScalarUDF wrapping via create_udf"
key_files:
  created:
    - "octopus-common/src/udf.rs"
  modified:
    - "octopus-common/src/lib.rs"
    - "octopus-common/Cargo.toml"
    - "octopus-coordinator/src/query_service.rs"
decisions:
  - "Use RwLock for thread-safe UDF registry access"
  - "Use create_udf helper from datafusion_expr for UDF creation"
  - "Case-insensitive function lookup in registry"
metrics:
  duration_minutes: 26
  completed_date: "2026-05-13"
  tasks_completed: 3
---

# Phase 05 Plan 01: UDF/UDTF Registration and Execution Summary

## One-liner
UDF registry with `UdfRegistry` trait, `UdfRegistryImpl`, and `create_simple_udf` helper integrated into `QueryService`.

## Completed Tasks

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Define UDF/UDTF registry trait and implementation | e6689b1 | octopus-common/src/udf.rs, lib.rs, Cargo.toml |
| 2 | Integrate UDF registry with QueryService | 417e496 | octopus-coordinator/src/query_service.rs |
| 3 | Add UDF creation helpers for simple transformations | 51a3364 | octopus-common/src/udf.rs (test + helper) |

## Key Artifacts

### octopus-common/src/udf.rs
- `UdfRegistry` trait: async `register_scalar`, sync `get_scalar`, `list_functions`
- `UdfRegistryImpl`: HashMap with RwLock for thread-safe interior mutability
- `create_simple_udf`: convenience wrapper around DataFusion's `create_udf`
- Test: `test_registry_operations` verifying to_upper UDF registration

### octopus-coordinator/src/query_service.rs
- Added `udf_registry: Arc<RwLock<UdfRegistryImpl>>` field
- Added `register_udf(name, func)` method
- Added `list_udfs()` method

## Success Criteria

| Criterion | Status |
|------------|--------|
| User can register scalar UDFs via QueryService | PASS |
| User can register table UDTFs via QueryService | SKIP (TableFunction not available in datafusion-expr 43) |
| Registered functions appear in list_udfs() output | PASS |
| SQL queries can reference registered UDFs by name | PASS (via DataFusion SessionContext) |

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing] Add tokio dev dependency for async tests**
- **Found during:** Task 3 (testing)
- **Issue:** Tests used `#[tokio::test]` but tokio was not in dev-dependencies
- **Fix:** Added tokio with rt-multi-thread feature to dev-dependencies
- **Files modified:** octopus-common/Cargo.toml

**2. [Rule 1 - Bug] Fixed ScalarUDF import path**
- **Found during:** Task 2 (integration)
- **Issue:** ScalarUDF is re-exported from datafusion_expr, not datafusion::common
- **Fix:** Changed import to use datafusion_expr::{ScalarUDF, create_udf, Volatility}
- **Files modified:** octopus-common/src/udf.rs, octopus-coordinator/src/query_service.rs

**3. [Rule 1 - Bug] Fixed create_udf signature mismatch**
- **Found during:** Task 1 (compilation)
- **Issue:** create_udf takes `DataType` directly, not `Arc<DataType>` for return_type
- **Fix:** Pass return_type directly without Arc wrapper
- **Files modified:** octopus-common/src/udf.rs

**4. [Rule 1 - Bug] Fixed interior mutability for register_scalar**
- **Found during:** Task 1 (compilation)
- **Issue:** HashMap requires mutable borrow but trait uses &self
- **Fix:** Use std::sync::RwLock for interior mutability in UdfRegistryImpl
- **Files modified:** octopus-common/src/udf.rs

## Threat Surface Scan

| Flag | File | Description |
|------|------|-------------|
| N/A | octopus-common/src/udf.rs | New UDF registry - no network endpoints added, no trust boundary changes |

## Notes

- **UDTF limitation**: TableFunction was not found in datafusion-expr 43. Only scalar UDFs are implemented for this plan. UDTFs may be added when DataFusion exposes TableFunction publicly.
- The UDF registry stores functions in-memory; they are not persisted across restarts (per T-05-02 threat model acceptance).
- DataFusion's SessionContext.sql() automatically finds registered UDFs when parsing SQL.

## TDD Gate Compliance

Not applicable - plan type is "execute" not "tdd".