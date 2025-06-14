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
- Use `// ---` for visual separation in **module blocks**, **impl blocks**, and **function bodies**
- Place separators after the opening brace and before the first meaningful line
- For modules: place separator after `mod name {` and before imports/content
- For impl blocks: place separator after `impl ... {` and before the first method
- For functions: place separator after function signature and before the main logic
- Keep separators consistent across the codebase

### Complete Style Guide

For a comprehensive example showing all formatting conventions, see `examples/formatting_style_guide.rs`. You can run it with:

```bash
cargo run --example formatting_style_guide
```

### Configuration

The project's formatting rules are defined in `rustfmt.toml`. This ensures consistent formatting across all contributors and CI environments.
