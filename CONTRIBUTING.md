# Contributing to SLIME

## Getting Started

1. Fork the repository
2. Clone your fork
3. Run `cargo build` to verify setup
4. Run `cargo test` to check existing functionality

## Development Workflow

1. Create a feature branch: `git checkout -b feature/description`
2. Make your changes
3. Add tests for new functionality
4. Ensure all tests pass: `cargo test`
5. Format code: `cargo fmt`
6. Submit a pull request

## Code Style

- Follow Rust standard formatting (`cargo fmt`)
- Document public APIs with doc comments
- Keep functions focused and small
- Error handling: use `Result` types, not panics

## Testing

- Unit tests in `src/` files
- Integration tests in `tests/`
- Example programs in `examples/`

## Language Design Proposals

Open an issue with the `design` label. Include:

- Motivation
- Proposed syntax
- Examples
- Potential impact on existing code

## Questions?

Open a discussion or reach out to maintainers.
