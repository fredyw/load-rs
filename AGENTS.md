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
- **Verification**: Running `./test.sh` automatically performs formatting and linting checks before running tests.
- **Fix**: Use the provided script to automatically fix formatting and common clippy lints.
  ```bash
  ./fix.sh
  ```

## 3. Documentation
- Update `README.md` if you add new CLI arguments or change existing functionality.
- Use docstrings (`///`) for public functions and structs in `src/lib.rs`.

## 4. Commit Messages
- **Structure**: Every commit must have a clear title and a descriptive body.
- **Content**: Explain *what* was changed and *why*. Avoid one-liner commits for non-trivial changes.
- **Atomic Commits**: Keep each commit focused on a single logical change. Do not group unrelated changes (e.g., a bug fix and a refactor) into the same commit.
- **Conciseness**: Be descriptive but avoid being overly verbose. Focus on key technical decisions or rationale.

## 5. Performance & Efficiency
Since `load-rs` is a performance-oriented tool:
- Be mindful of memory allocations in hot loops.
- Use `hdrhistogram` for latency tracking where appropriate.
- Leverage `tokio` and `reqwest` for asynchronous I/O.

## 6. Repository Structure
- `src/lib.rs`: Main library entry point (module declarations and re-exports).
- `src/models.rs`: Core data structures (HttpMethod, Stats, LoadTestResult, etc.).
- `src/runner.rs`: The `LoadTestRunner` implementation and execution logic.
- `src/generator.rs`: `RequestGenerator` trait and dynamic request logic.
- `src/main.rs`: CLI entry point, argument parsing, and UI reporting.
- `tests/`: Integration tests for CLI, statistics, TLS, and core logic.


Thank you for contributing to `load-rs`!
