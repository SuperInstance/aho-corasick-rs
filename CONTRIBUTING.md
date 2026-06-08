# Contributing

## Development Setup

```bash
cargo build
cargo test
cargo clippy -- -D warnings
cargo fmt --check
```

## Submitting Changes

1. Fork the repository
2. Create a feature branch
3. Ensure all tests pass and clippy is clean
4. Submit a pull request

## Code Style

- Follow `cargo fmt` conventions
- All public APIs must have doc comments
- No `unsafe` code without justification
