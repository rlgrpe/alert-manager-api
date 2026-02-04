# Agents

Instructions for AI agents working on this codebase.

## Build Commands

```bash
# Build the project
cargo build

# Build with all features
cargo build --all-features

# Build for release
cargo build --release
```

## Test Commands

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --show-output

# Run tests for specific feature
cargo test --no-default-features --features native-tls
cargo test --no-default-features --features rustls-tls
```

## Lint Commands

```bash
# Check formatting
cargo fmt --all -- --check

# Run clippy
cargo clippy --all-targets --all-features -- -D warnings

# Run pre-commit hooks
pre-commit run --all-files
```

## Documentation

```bash
# Generate documentation
cargo doc --all-features --no-deps

# Open documentation
cargo doc --all-features --no-deps --open
```

## Project Structure

```
alert-manager-api/
├── src/
│   ├── lib.rs      # Library entry point, public exports
│   ├── client.rs   # AlertmanagerClient implementation
│   ├── types.rs    # Alert, AlertSeverity types
│   └── errors.rs   # Error types and handling
├── examples/       # Usage examples
│   ├── basic.rs
│   ├── batch_alerts.rs
│   ├── resolve_alert.rs
│   └── custom_middleware.rs
└── Cargo.toml      # Dependencies and features
```

## Features

- `native-tls` (default) - Uses system TLS
- `rustls-tls` - Uses rustls for TLS

## Key Types

- `AlertmanagerClient` - Main client for sending alerts
- `Alert` - Represents an alert with labels and annotations
- `AlertSeverity` - Enum for Critical, Warning, Info
- `AlertmanagerError` - Error type with retry hints
