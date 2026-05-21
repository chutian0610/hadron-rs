---
title: Exchange
type: concept
tags: [exchange-operator,Arrow-Flight,data-exchange,worker-pull]
sources:
  - octopus-coordinator/src/exchange_operator.rs
  - octopus-executor/src/
  - octopus-worker/src/
related:
  - "[[Architecture]]"
  - "[[Coordinator]]"
  - "[[Execution]]"
---

# Exchange

Exchange operators define stage boundaries in distributed query plans. They are the only points where data crosses worker boundaries, enabling Trino-style pipeline execution.

## Exchange Operator

`ExchangeOperator` is a DataFusion `ExecutionPlan` that defines stage boundaries with exchange modes.

`octopus-coordinator/src/exchange_operator.rs:1`

### Exchange Modes

| Mode | Description | Use Case |
|------|-------------|----------|
| `Single` | Single sender | Small results, ORDER BY with LIMIT |
| `Hash` | Hash-based partitioning | Shuffling for hash joins or aggregations |
| `RoundRobin` | Round-robin distribution | Load balancing without data affinity |
| `Broadcast` | Send to all workers | Small dimension tables in joins |

### Physical Plan Structure

```rust
ExchangeNode {
    mode: ExchangeMode,
    input: Box<ExecutionPlan>,
    output_partitioning: Partitioning,
}
```

## Worker-Pull Model

Octopus uses a worker-pull model where the receiver requests data from senders.

```
Sender Worker                    Receiver Worker
    │                                  │
    │←────── GetExchangeData ─────────│  (request)
    │─────── Arrow Flight Stream ─────→│  (data)
    │                                  │
```

This contrasts with push models where senders push data to receivers.

### Advantages

- **Backpressure**: Receiver controls data flow rate
- **Simplicity**: No need to coordinate sender schedules
- **Efficiency**: Direct worker-to-worker transfer

## Arrow Flight Data Plane

Workers exchange data using Arrow Flight over gRPC.

`octopus-worker/src/flight_server.rs:1`

### Flight Endpoint

```
GetFlightData: (ticket) → FlightDataStream
```

- `ticket`: Encodes the exchange ID and partition info
- `FlightDataStream`: Stream of Arrow record batches

### FlightHandler

`FlightHandler` processes flight requests and manages data transfer.

`octopus-worker/src/flight_handler.rs:1`

## Exchange Data Flow

```
Stage 1 (Sender)                              Stage 2 (Receiver)
     │                                              ↑
     │  1. Execute local plan                       │
     │  2. Collect output batches                  │
     │  3. Register with FlightServer ─────────────→│  4. Pull batches
     │                                              │  5. Execute local plan
     │                                              ↓
```

## Stage Boundaries

All operators between two Exchanges execute locally on a worker:

```
┌────────────────────────────────────────────────────────┐
│ Worker 1                                              │
│ ┌──────────┐    ┌──────────┐    ┌──────────────────┐ │
│ │   Scan   │───→│  Filter  │───→│ Exchange(Send)   │ │
│ └──────────┘    └──────────┘    └──────────────────┘ │
└────────────────────────────────────────────────────────┘
                          │
                          │ Arrow Flight (Hash shuffle)
                          ↓
┌────────────────────────────────────────────────────────┐
│ Worker 2                                              │
│ ┌──────────────────┐    ┌──────────┐    ┌──────────┐ │
│ │ Exchange(Recv)   │───→│   Agg    │───→│   Scan   │ │
│ └──────────────────┘    └──────────┘    └──────────┘ │
└────────────────────────────────────────────────────────┘
```

## Exchange Deadlock Prevention

StagePlanner validates the Stage DAG before execution to prevent cyclic dependencies through Exchange operators (Pitfall 5).

### Validation Rules

1. No cycle in the DAG
2. Each Exchange has exactly one sender per partition
3. All receivers are registered before execution starts

## Code References

- `octopus-coordinator/src/exchange_operator.rs:1` - ExchangeOperator definition
- `octopus-worker/src/flight_server.rs:1` - FlightServer implementation
- `octopus-worker/src/flight_handler.rs:1` - FlightHandler implementation
