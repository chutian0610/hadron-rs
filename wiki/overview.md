---
title: Octopus
type: overview
tags: [distributed-sql,MPP,Trino-style,Arrow-Flight]
sources:
  - ./
  - octopus-coordinator/src/
  - octopus-executor/src/
  - octopus-worker/src/
---

# Octopus

Octopus is a distributed MPP (Massively Parallel Processing) query engine following a Trino-style streaming architecture. It enables SQL queries to be executed across multiple worker nodes with Arrow Flight as the data plane and gRPC for control plane communication.

## Architecture

```
Client → Coordinator → Workers → Arrow Flight (data plane)
                     ↑
               gRPC (control plane)
```

The coordinator acts as the central brain, handling SQL parsing, distributed planning, task scheduling, and HTTP API endpoints. Workers execute query fragments in parallel and exchange data via Arrow Flight. Exchange operators define stage boundaries where data crosses worker boundaries.

## Core Concepts

- [[Architecture]] - System design overview and data flow
- [[Coordinator]] - HTTP API server, query service, stage planner, and task scheduler
- [[Execution]] - Local query execution via DataFusion
- [[Exchange]] - Exchange operators and distributed data exchange
- [[Federated Queries]] - PostgreSQL and MySQL connector support
- [[UDF Registry]] - Custom scalar function registration
- [[Worker Runtime]] - CPU/IO runtime separation and retry handling

## Key Principles

**Pipeline Streaming**: Exchange operators are the only points where data crosses worker boundaries. Everything else stays local and streaming, enabling Trino-style pipeline execution where stages run concurrently.

**Data Locality**: The QueryScheduler assigns tasks to workers based on partition locality scoring, with round-robin fallback for load balancing.

## Directory Structure

```
octopus-coordinator/  - Central brain: SQL parsing, distributed planning, HTTP API
octopus-executor/      - Local query execution using DataFusion SessionContext
octopus-worker/        - Task execution + Arrow Flight server
octopus-cli/           - Client CLI (interactive/repl mode)
octopus-common/        - Shared utilities, error types, UDF registry traits
proto/                 - gRPC protocol definitions
```

## Getting Started

1. Build the project: `cargo build --workspace`
2. See [[Quickstart]] for running the coordinator and workers
3. See [[CLI Usage]] for client connection options
