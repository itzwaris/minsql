# Contributing to minsql

Thank you for your interest in contributing to minsql.

## Code of Conduct

Be respectful, constructive, and professional in all interactions.

## Development Setup

### Prerequisites

- Rust 1.75 or later
- GCC/Clang with C++20 support
- Linux development environment (Ubuntu/Debian preferred)
- Git

### Building

```bash
git clone https://github.com/notwaris/minsql.git
cd minsql
cargo build
```

### Running Tests

```bash
cargo test
cargo test --package validation
```

## Code Style

### Rust Code

- Follow standard Rust conventions
- Use `cargo fmt` before committing
- Run `cargo clippy` and address all warnings
- Keep functions focused and single-purpose
- Comments should explain WHY, not WHAT

### C/C++ Code

- Use consistent indentation (4 spaces)
- Prefer stack allocation where safe
- Use RAII in C++ code
- Keep C code simple and predictable
- Avoid complex macros

## Commit Messages

Write clear, descriptive commit messages:

```
Add B-tree index split logic

Implements node splitting when a B-tree node exceeds capacity.
Uses a simple median-split strategy for balanced tree growth.
```

## Pull Request Process

1. Fork the repository
2. Create a feature branch: `git checkout -b feature-name`
3. Make your changes
4. Run tests: `cargo test`
5. Commit with clear messages
6. Push to your fork
7. Open a pull request

### PR Guidelines

- Keep PRs focused on a single change
- Include tests for new functionality
- Update documentation as needed
- Ensure all tests pass
- Respond to review feedback promptly

## Areas for Contribution

### High Priority

- Storage engine optimizations
- Query optimizer improvements
- Additional index types
- Replication stability
- Performance benchmarks

### Documentation

- Usage examples
- Architecture diagrams
- Performance tuning guides
- Troubleshooting documentation

### Testing

- Unit test coverage
- Integration tests
- Crash recovery tests
- Performance regression tests

## Architecture Guidelines

### Language Boundaries

- **Rust**: Control plane, safety-critical logic, orchestration
- **C**: Performance-critical deterministic code (WAL, pages)
- **C++**: Data structures benefiting from templates (indexes)

Do not blur these boundaries without discussion.

### Safety Requirements

- All unsafe Rust code must have safety comments
- C/C++ code must not leak memory
- FFI boundaries must validate all inputs
- Resource cleanup must be deterministic

## Performance Expectations

- Profile before optimizing
- Benchmark significant changes
- Consider cache effects
- Minimize allocations in hot paths
- Document performance-critical sections

## Questions?

Open an issue with the "question" label or reach out to the maintainer.

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
