# Contributing to Axum Quickstart

Thanks for considering contributing!

Before submitting a pull request:

- Ensure all tests pass (`cargo test`)
- Format your code (`cargo fmt`)
- If your change affects behavior, please update `CHANGELOG.md` under the [Unreleased] section
- Keep commits focused and descriptive

We follow [Keep a Changelog](https://keepachangelog.com/en/1.0.0/) and
[Semantic Versioning](https://semver.org/).

## Code Formatting

This project uses `rustfmt` for consistent code formatting. All code should be formatted before committing.

### Quick Commands
```bash
# Format all code
cargo fmt

# Check if code is formatted (used by CI)
cargo fmt --check

# View the complete formatting style guide
cargo run --example formatting_style_guide

# Run the complete test suite (unit + integration)
./scripts/test-all.sh

# Run only unit tests
./scripts/run-unit-tests.sh

# Run only integration tests  
./scripts/run-integration-tests.sh
```

### Visual Separators

Since `rustfmt` removes blank lines at the start of impl blocks, function bodies, and module blocks, we use comment separators for visual clarity:

```rust
// Module blocks
mod helpers {
    // ---
    use super::*;
    
    pub fn some_function() {
        // ---
        // function body
    }
}

// Struct definitions
pub struct Credential {
    // ---
    pub id: Vec<u8>,
    pub user_id: Uuid,
    pub counter: i32,
}

// Impl blocks
impl Metrics for PrometheusMetrics {
    // ---
    fn render(&self) -> String {
        // ---
        super::render_metrics()
    }
}

// Regular functions
pub fn init_metrics() {
    // ---
    let handle = PrometheusBuilder::new()
        .install_recorder()
        .expect("failed to install Prometheus recorder");
    // ...
}

// Struct literals (construction) - NO separator
let credential = Credential {
    id: vec![1, 2, 3],
    user_id: user.id,
    counter: 0,
};

// Test modules
#[cfg(test)]
mod tests {
    // ---
    use super::*;

    #[test]
    fn test_something() {
        // ---
        // test body
    }
}
```

**Style Guidelines:**
1) Use `// ---` for visual separation in at a minimum **module blocks**, **impl blocks**, **struct definitions**, and **function bodies**
2) Place separators after the opening brace and before the first meaningful line
3) Between meaningful steps of logic processing (e.g., separating validation, database operations, and response formatting)
4) For modules: place separator after `mod name {` and before imports/content
5) For impl blocks: place separator after `impl ... {` and before the first method
6) For struct definitions: place separator after `struct Name {` and before field declarations
7) For functions: place separator after function signature and before the main logic
8) Do NOT use separators inside struct literals (during construction)
9) Keep separators consistent across the codebase

**Note:** This project uses rustfmt's default configuration. The `// ---` separator pattern is a formatting convention to work around rustfmt's blank line removal in stable Rust.

### Complete Style Guide

For a comprehensive example showing all formatting conventions, see `examples/formatting_style_guide.rs`. You can run it with:

```bash
cargo run --example formatting_style_guide
```

### Configuration

The project's formatting rules are defined in `rustfmt.toml`. This ensures consistent formatting across all contributors and CI environments.

## Documentation and Doc Comments

This project follows a **production-grade documentation standard** for Rust code.

### Required Doc Comments

Use Rust doc comments (`///`) for:

- Public structs
- Public enums
- Public functions
- Public modules that define architectural boundaries
- Macros that encode non-obvious behavior or policy decisions

Doc comments should describe **intent, guarantees, and failure semantics** —
not restate what the code obviously does.

### Optional (Encouraged) Doc Comments

Doc comments or short block comments are encouraged for:

- Internal functions with security or operational implications
- Startup and initialization logic
- Configuration parsing and validation
- Code that enforces invariants or policy decisions

### Not Required

Doc comments are not required for:

- Trivial helpers
- Simple getters or pass-through functions
- Test code (assert messages should be sufficient)
- Obvious glue code

### General Guidance

- Prefer documenting *why* over *how*
- Be explicit about failure behavior
- Keep comments accurate and up to date
- Avoid over-documenting trivial code

Well-written doc comments are considered part of the code’s correctness.
