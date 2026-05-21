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

## Interactive Mode Commands

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
