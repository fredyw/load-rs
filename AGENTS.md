# Rules for AI Agents

Welcome! As an AI agent working on the `load-rs` repository, please adhere to the following rules and guidelines to ensure high-quality contributions.

## 1. Testing Requirements
**Every change must be accompanied by tests.** Whether it's a new feature, a bug fix, or a refactoring effort, you must include:
- **Unit Tests**: For internal logic, utility functions, and individual components.
- **Integration Tests**: For CLI behavior, end-to-end flows, and multi-component interactions.

### Running Tests
Before submitting any change, ensure all tests pass:
```bash
./test.sh
```

## 2. Coding Standards
- **Idiomatic Rust**: Follow standard Rust idioms and best practices.
- **Check**: Ensure your code passes formatting and linting checks.
  ```bash
  ./check.sh
  ```
- **Fix**: Use the provided script to automatically fix formatting and common clippy lints.
  ```bash
  ./fix.sh
  ```

## 3. Documentation
- Update `README.md` if you add new CLI arguments or change existing functionality.
- Use docstrings (`///`) for public functions and structs in `src/lib.rs`.

## 4. Performance & Efficiency
Since `load-rs` is a performance-oriented tool:
- Be mindful of memory allocations in hot loops.
- Use `hdrhistogram` for latency tracking where appropriate.
- Leverage `tokio` and `reqwest` for asynchronous I/O.

## 5. Repository Structure
- `src/lib.rs`: Main library entry point (module declarations and re-exports).
- `src/models.rs`: Core data structures (HttpMethod, Stats, LoadTestResult, etc.).
- `src/runner.rs`: The `LoadTestRunner` implementation and execution logic.
- `src/generator.rs`: `RequestGenerator` trait and dynamic request logic.
- `src/main.rs`: CLI entry point, argument parsing, and UI reporting.
- `tests/`: Integration tests for CLI, statistics, TLS, and core logic.


Thank you for contributing to `load-rs`!
