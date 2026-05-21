---
title: CLI Usage
type: guide
tags: [CLI,modes,commands,interactive,batch]
sources:
  - octopus-cli/src/
related:
  - "[[Quickstart]]"
  - "[[Coordinator]]"
---

# CLI Usage

The Octopus CLI (`octopus-cli`) provides multiple modes for interacting with the distributed query engine.

## CLI Modes

### Local Mode

Executes queries directly via DataFusion without the coordinator. Useful for testing and single-node execution.

```bash
cargo run -p octopus-cli -- --mode local
```

### Interactive Mode

REPL-style interface for running queries against a coordinator. Supports command history and completion.

```bash
cargo run -p octopus-cli -- --mode interactive --host localhost --port 50051
```

### Batch Mode

Executes SQL from a file or stdin without interactive prompts.

```bash
# From file
cargo run -p octopus-cli -- --mode batch --file queries.sql

# From stdin
echo "SELECT 1;" | cargo run -p octopus-cli -- --mode batch
```

## CLI Options

```
USAGE:
    octopus-cli [OPTIONS]

OPTIONS:
    -m, --mode <MODE>          Mode: local, interactive, batch [default: interactive]
    -h, --host <HOST>          Coordinator host [default: localhost]
    -p, --port <PORT>          Coordinator port [default: 50051]
    -f, --file <FILE>          SQL file for batch mode
    -o, --output <FORMAT>      Output format: table, csv, json [default: table]
    --help                     Print help
```

## Interactive Mode Commands

| Command | Description |
|---------|-------------|
| `SELECT ...` | Execute a SELECT query |
| `EXPLAIN ...` | Parse and show query plan |
| `\q` | Quit |
| `\h` | Show help |
| `\d <table>` | Describe table schema |
| `\t` | Toggle timing |

## Output Formats

### Table Format (Default)

```
+------+
| id   |
+------+
| 1    |
| 2    |
+------+
```

### CSV Format

```bash
cargo run -p octopus-cli -- --mode batch --output csv --file query.sql > output.csv
```

### JSON Format

```bash
cargo run -p octopus-cli -- --mode batch --output json --file query.sql
```

## Connection via JDBC

For Java applications, use the JDBC driver:

```java
// Connection URL format
String url = "jdbc:octopus://localhost:50051";

// Connect
Connection conn = DriverManager.getConnection(url);

// Execute query
Statement stmt = conn.createStatement();
ResultSet rs = stmt.executeQuery("SELECT * FROM orders");
```

See [[Reference/Crates]] for JDBC driver details.

## Error Handling

CLI reports errors with context:

```
Error: Sql error
  at QueryService::execute (src/query_service.rs:42)
  caused by: Parse error: syntax error at position 15
```

## Code References

- `octopus-cli/src/main.rs` - CLI entry point
- `octopus-cli/src/coordinator_client.rs` - HTTP client for coordinator
- `octopus-cli/src/repl.rs` - Interactive REPL implementation
