# Contributing to minsql

Thanks for considering contributing to minsql! We appreciate your interest in making this project better.

## Code of Conduct

Keep things respectful and professional. We're all here to build something great together.

## Getting Started

### What You'll Need

- Rust 1.82+
- GCC/Clang with C++20 support
- Linux environment (Ubuntu/Debian works best)
- Git

### Building from Source

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

### Rust
- Follow standard Rust conventions
- Run `cargo fmt` before committing
- Fix all `cargo clippy` warnings
- Keep functions small and focused
- Comment the "why", not the "what"

### C/C++
- 4 spaces for indentation
- Prefer stack allocation when possible
- Use RAII in C++ code
- Keep C code simple
- Avoid complex macros

## Making Changes

### Commit Messages
Keep them clear and descriptive:

```
Add B-tree index split logic

Implements node splitting when capacity is exceeded.
Uses median-split strategy for balanced growth.
```

### Pull Requests

1. Fork the repo
2. Create a branch: `git checkout -b your-feature`
3. Make your changes
4. Run tests: `cargo test`
5. Commit your work
6. Push to your fork
7. Open a PR

Keep PRs focused on one thing. Include tests for new features. Make sure everything passes before submitting.

## What to Work On

### High Priority
- Storage engine optimizations
- Query optimizer improvements
- New index types
- Replication stability
- Performance benchmarks

### Documentation
- Usage examples
- Architecture docs
- Performance guides
- Troubleshooting tips

### Testing
- Unit tests
- Integration tests
- Crash recovery tests
- Performance tests

## Architecture Notes

### Language Usage
- **Rust**: Safety-critical logic, orchestration
- **C**: Performance-critical deterministic code (WAL, pages)
- **C++**: Complex data structures (indexes, trees)

Don't mix these without good reason.

### Safety First
- Document all unsafe Rust code
- No memory leaks in C/C++
- Validate all FFI inputs
- Clean up resources properly

## Performance

- Profile before you optimize
- Benchmark significant changes
- Think about cache effects
- Minimize allocations in hot paths
- Document performance-critical code

## Questions?

Open an issue or reach out to the maintainer. We're happy to help!

## License

By contributing, you agree your code will be MIT licensed.
