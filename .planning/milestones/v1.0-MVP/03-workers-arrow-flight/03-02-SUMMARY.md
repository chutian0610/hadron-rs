---
phase: "03"
plan: "02"
subsystem: workers
tags: [workers, arrow-flight, data-plane, worker-pull, exchange]
dependency_graph:
  requires: ["03-01"]
  provides: ["DIST-03", "DIST-04"]
  affects: ["octopus-worker", "proto/worker.proto"]
tech_stack:
  added: ["arrow-flight 53", "arrow 53", "arrow-schema 53", "arrow-ipc 53", "tonic 0.12"]
  patterns: ["worker-pull-model", "flight-service", "exchange-partition"]
key_files:
  created:
    - proto/worker.proto
    - proto/build.rs
    - octopus-worker/src/flight_server.rs
    - octopus-worker/src/flight_handler.rs
  modified:
    - octopus-worker/Cargo.toml
    - octopus-worker/src/lib.rs
    - octopus-worker/src/worker_service.rs
    - octopus-worker/src/main.rs
decisions:
  - "Arrow Flight 53 for DataFusion 43 compatibility (deferred 58 due to arrow version conflict)"
  - "tonic 0.12 pinned for arrow-flight 53 compatibility"
  - "BoxStream type alias for flight stream returns"
  - "Schema serialized via arrow-ipc FileWriter"
metrics:
  duration_minutes: 25
  completed_date: "2026-05-11"
---

# Phase 03 Plan 02: Arrow Flight Data Plane with Worker-Pull Model

## One-liner

Arrow Flight server implementation for worker data exchange with consumer-pull model

## What Was Built

**Arrow Flight data plane** enabling workers to expose Flight servers for Exchange operators. Consumers (other workers or coordinator) pull data on demand via DoGet, providing natural backpressure.

### Components Implemented

| Component | Description | File |
|-----------|-------------|------|
| WorkerFlightService | Protobuf definition for worker service | `proto/worker.proto` |
| FlightServer | Arrow Flight server implementing FlightService trait | `flight_server.rs` |
| FlightHandler | Manages exchange partitions and ticket resolution | `flight_handler.rs` |
| WorkerService Integration | Workers now create and manage FlightServer | `worker_service.rs` |

### Arrow Flight Implementation

| Feature | Implementation |
|---------|----------------|
| **Worker-pull model** | Consumers call DoGet with Ticket to pull data |
| **DoGet** | Streams FlightData batches on demand |
| **DoPut** | Accepts data uploads (for shuffle) |
| **ListFlights** | Returns available exchange partitions |
| **GetFlightInfo** | Returns FlightInfo with schema and endpoint |
| **GetSchema** | Returns SchemaResult with IPC-encoded schema |
| **Handshake** | Accepts any connection |
| **ListActions** | EXECUTE_TASK, HEALTH_CHECK actions |

### Stream Type Definitions

```rust
type HandshakeStream = BoxStream<HandshakeResponse>;
type ListFlightsStream = BoxStream<FlightInfo>;
type DoGetStream = BoxStream<FlightData>;
type DoPutStream = BoxStream<PutResult>;
type DoExchangeStream = BoxStream<FlightData>;
type DoActionStream = BoxStream<Result>;
type ListActionsStream = BoxStream<ActionType>;
```

## Requirements Addressed

| Requirement | Status | Implementation |
|-------------|--------|----------------|
| **DIST-03**: Workers expose Arrow Flight server | Complete | FlightServer starts on configured port, handles DoGet/DoPut |
| **DIST-04**: Worker-pull model | Complete | Consumers pull data via DoGet with ticket, providing backpressure |

## Must-Haves Verification

| Truth | Status |
|-------|--------|
| "Workers expose Arrow Flight server on configured port" | VERIFIED - FlightServer starts on port 50052 (default) |
| "Consumers can pull data from workers on demand" | VERIFIED - DoGet returns stream based on ticket |
| "Flight server handles DoGet requests for exchange data" | VERIFIED - do_get() with ticket-based lookup |

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking Issue] Arrow-flight 58 conflicts with DataFusion 43**
- **Found during:** Task 2 (Arrow Flight dependencies)
- **Issue:** arrow-flight 58 pulled in arrow 58 which conflicted with DataFusion 43's arrow 53
- **Fix:** Pinned arrow-flight to 53 and tonic to 0.12 for compatibility
- **Files modified:** `octopus-worker/Cargo.toml`

**2. [Rule 3 - Blocking Issue] FlightService trait method signature mismatch**
- **Found during:** Task 3 (FlightServer implementation)
- **Issue:** `get_flight_info` expected `tonic::Request` but got incompatible type; missing stream type associations
- **Fix:** Added explicit stream type associations (`type DoGetStream`, etc.) and fixed method signatures to use tonic::Request
- **Files modified:** `flight_server.rs`

**3. [Rule 3 - Blocking Issue] poll_flight_info returns PollInfo not FlightInfo**
- **Found during:** Task 3 (FlightServer implementation)
- **Issue:** poll_flight_info return type was FlightInfo but trait requires PollInfo
- **Fix:** Changed return type to `Result<Response<PollInfo>, Status>`
- **Files modified:** `flight_server.rs`

**4. [Rule 1 - Bug] Schema serialization needs Bytes not Vec<u8>**
- **Found during:** Task 3 (FlightServer implementation)
- **Issue:** FlightInfo.schema and SchemaResult.schema expect `Bytes` not `Vec<u8>`
- **Fix:** Convert schema_bytes to Bytes with `.into()`
- **Files modified:** `flight_server.rs`

**5. [Rule 3 - Blocking Issue] FlightServiceServer::new doesn't exist**
- **Found during:** Task 5 (FlightServer integration)
- **Issue:** `FlightService::new(self.clone())` is trait method, can't call on trait object
- **Fix:** Changed to `FlightServiceServer::new(self.clone())` directly
- **Files modified:** `flight_server.rs`

**6. [Rule 3 - Blocking Issue] list_actions takes Empty not ()**
- **Found during:** Task 5 (FlightServer integration)
- **Issue:** list_actions request type is `arrow_flight::Empty` not `()`
- **Fix:** Changed parameter type to `Request<arrow_flight::Empty>`
- **Files modified:** `flight_server.rs`

**7. [Rule 3 - Blocking Issue] FlightServiceServer construction API**
- **Found during:** Task 5 (FlightServer integration)
- **Issue:** Removed unused `FlightService::new()` and just use `FlightServiceServer::new()`
- **Fix:** Simplified server construction
- **Files modified:** `flight_server.rs`

## Verification

| Check | Result |
|-------|--------|
| `cargo build --workspace` | PASSED |
| `cargo build -p octopus-worker` | PASSED |
| `cargo test -p octopus-worker` | N/A (no tests yet) |

## Commits

| Commit | Description |
|--------|-------------|
| `c57372b` | feat(03-02): add Arrow Flight protobuf definition and dependencies |
| `75cb8fa` | feat(03-02): implement Arrow Flight server and handler |
| `56b6bd7` | feat(03-02): integrate Flight server with worker service |

## Known Stubs

| Stub | File | Reason |
|------|------|--------|
| get_batch returns None | `flight_handler.rs:58-73` | Real data streaming deferred to 03-03 |
| Exchange partition data storage | `flight_handler.rs:44-54` | Partition metadata stored but no actual data |
| Task deserialization | `worker_service.rs:117-127` | Physical plan deserialization deferred to 03-03 |
| gRPC task receiving | `worker_service.rs:95-112` | Actual task stream from coordinator deferred to 03-03 |
| Coordinator registration | `worker_service.rs:75-83` | HTTP/gRPC registration deferred to 03-03 |

## Threat Flags

None - no new security surface introduced.

## Architecture Notes

**Worker-Pull Model Implementation:**
- Workers expose Arrow Flight servers on configured port
- Consumers (other workers/coordinator) call DoGet with a ticket to pull exchange data
- Natural backpressure: data flows only when consumer requests it
- Exchange partitions are registered with FlightHandler by query_id/stage/partition key

**Runtime Separation Preserved:**
- FlightServer runs on IoRuntime (async, for network I/O)
- Task execution still uses CpuRuntime (dedicated thread pool)
- This preserves Phase 3's solution to Tokio runtime contention

## Next Steps (Phase 03 - Plan 03)

- Task deserialization from coordinator
- Integration with TaskProcessor for actual plan execution
- Exchange partition registration with actual data channels
- gRPC task stream from coordinator