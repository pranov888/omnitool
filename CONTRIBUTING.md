# Contributing to omnitool

Thank you for your interest in improving `omnitool`! We welcome contributions of all kinds: bug fixes, new utility subcommands, performance improvements, and documentation updates.

## Development Setup

1. **Prerequisites**:
   - Rust toolchain (`rustc`, `cargo` 1.75+)

2. **Clone & Test**:
   ```bash
   git clone https://github.com/pranov888/omnitool.git
   cd omnitool
   cargo test
   ```

3. **Quality and Verification Checklist**:
   - Ensure all tests pass: `cargo test`
   - Verify code formatting: `cargo fmt --check`
   - Verify static analysis and lints: `cargo clippy -- -D warnings`

4. **Submitting a Pull Request**:
   - Create a feature branch from `dev`: `git checkout -b feat/your-feature-name`
   - Follow Conventional Commits: `feat(...)`, `fix(...)`, `docs(...)`
   - Ensure commits are atomic and descriptive
   - Open a Pull Request targeting `dev`
