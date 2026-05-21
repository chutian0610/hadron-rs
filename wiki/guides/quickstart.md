---
title: Quickstart
type: guide
tags: [build,run,setup]
sources:
  - ./
related:
  - "[[CLI Usage]]"
  - "[[Architecture]]"
---

# Quickstart

Get Octopus running in your local environment.

## Prerequisites

- Rust 1.70+ (install via [rustup](https://rustup.rs/))
- Cargo (comes with Rust)

## Build

Clone the repository and build the workspace:

```bash
git clone https://github.com/your-org/octopus.git
cd octopus
cargo build --workspace
```

Build takes approximately 2-3 minutes on first run (downloading dependencies).

## Run the Coordinator

Start the coordinator server on port 50051:

```bash
cargo run -p octopus-coordinator
```

Expected output:
```
Starting coordinator on port 50051
HTTP server listening on 0.0.0.0:50051
```

## Run Workers (Optional)

For distributed execution, start worker processes:

```bash
# Worker 1
cargo run -p octopus-worker -- --port 50052

# Worker 2
cargo run -p octopus-worker -- --port 50053
```

Workers register with the coordinator and wait for tasks.

## Run the CLI

Start the interactive CLI:

```bash
cargo run -p octopus-cli
```

### Local Mode

Run queries directly via DataFusion without coordinator:

```bash
cargo run -p octopus-cli -- --mode local
```

### Interactive Mode

Connect to coordinator:

```bash
cargo run -p octopus-cli -- --mode interactive --host localhost --port 50051
```

### Batch Mode

Execute SQL from a file:

```bash
cargo run -p octopus-cli -- --mode batch --host localhost --port 50051 --file queries.sql
```

## Submit a Query

Using curl to submit a query:

```bash
curl -X POST http://localhost:50051/query/submit \
  -H "Content-Type: application/json" \
  -d '{"sql": "SELECT 1 + 1 AS result"}'
```

Response:
```json
{
  "query_id": "550e8400-e29b-41d4-a716-446655440000",
  "state": "RUNNING"
}
```

Check query state:

```bash
curl http://localhost:50051/query/state/550e8400-e29b-41d4-a716-446655440000
```

## Next Steps

- See [[CLI Usage]] for detailed CLI options
- See [[Architecture]] for system design
- Explore the codebase starting with `octopus-coordinator/src/`
