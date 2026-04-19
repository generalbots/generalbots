# Cargo Tools Reference

This chapter documents essential Cargo tools for botserver development, including code coverage, security auditing, performance profiling, and code quality tools.

## Overview

The Rust ecosystem provides powerful tools through Cargo extensions. These tools help maintain code quality, identify security vulnerabilities, measure test coverage, and optimize performance.

## Code Coverage with cargo-tarpaulin

### Installation

```bash
cargo install cargo-tarpaulin
```

### Basic Usage

Run code coverage analysis:

```bash
cargo tarpaulin
```

This generates a coverage report showing which lines of code are exercised by tests.

### Output Formats

Generate HTML report:

```bash
cargo tarpaulin --out Html
```

Generate multiple formats (coverage report, lcov for CI):

```bash
cargo tarpaulin --out Html --out Lcov --out Json
```

### Coverage with Features

Test with specific features enabled:

```bash
cargo tarpaulin --features vectordb,email
```

Test all features:

```bash
cargo tarpaulin --all-features
```

### Excluding Files

Exclude test files and generated code from coverage:

```bash
cargo tarpaulin --ignore-tests --exclude-files "gen/*" "tests/*"
```

### Coverage Thresholds

Fail if coverage drops below a threshold (useful for CI):

```bash
cargo tarpaulin --fail-under 80
```

### Verbose Output

Show detailed coverage per function:

```bash
cargo tarpaulin --verbose
```

### Integration with CI

Example GitHub Actions workflow:

```yaml
- name: Install tarpaulin
  run: cargo install cargo-tarpaulin

- name: Generate coverage
  run: cargo tarpaulin --out Xml --fail-under 70

- name: Upload coverage to Codecov
  uses: codecov/codecov-action@v3
  with:
    files: cobertura.xml
```

### Configuration File

Create `.tarpaulin.toml` for project-wide settings:

```toml
[config]
command = "test"
features = "vectordb"
ignore-tests = true
out = ["Html", "Lcov"]
exclude-files = ["gen/*"]
timeout = "120s"
```

## Security Auditing with cargo-audit

### Installation

```bash
cargo install cargo-audit
```

### Basic Usage

Check for known security vulnerabilities in dependencies:

```bash
cargo audit
```

### Continuous Auditing

Run audit as part of CI pipeline to catch new vulnerabilities:

```bash
cargo audit --deny warnings
```

### Fix Vulnerabilities

Generate a fix for vulnerable dependencies (when possible):

```bash
cargo audit fix
```

### Database Updates

Update the vulnerability database:

```bash
cargo audit fetch
```

### Ignore Known Issues

Create `.cargo/audit.toml` to ignore specific advisories:

```toml
[advisories]
ignore = [
    "RUSTSEC-2020-0071",  # Reason for ignoring
]
```

### JSON Output for CI

```bash
cargo audit --json > audit-report.json
```

## Dependency Analysis with cargo-deny

### Installation

```bash
cargo install cargo-deny
```

### Configuration

Create `deny.toml`:

```toml
[advisories]
vulnerability = "deny"
unmaintained = "warn"

[licenses]
unlicensed = "deny"
allow = [
    "MIT",
    "Apache-2.0",
    "BSD-3-Clause",
]

[bans]
multiple-versions = "warn"
deny = [
    { name = "openssl" },  # Prefer rustls
]

[sources]
unknown-registry = "deny"
unknown-git = "deny"
```

### Usage

Check all configured rules:

```bash
cargo deny check
```

Check specific categories:

```bash
cargo deny check advisories
cargo deny check licenses
cargo deny check bans
cargo deny check sources
```

## Code Formatting with cargo-fmt

### Usage

Format all code:

```bash
cargo fmt
```

Check formatting without changes:

```bash
cargo fmt --check
```

### Configuration

Create `rustfmt.toml`:

```toml
edition = "2021"
max_width = 100
tab_spaces = 4
use_small_heuristics = "Default"
reorder_imports = true
group_imports = "StdExternalCrate"
```

## Linting with cargo-clippy

### Usage

Run clippy with warnings as errors:

```bash
cargo clippy -- -D warnings
```

Run with all lints:

```bash
cargo clippy -- -W clippy::all -W clippy::pedantic
```

### Fix Suggestions Automatically

```bash
cargo clippy --fix --allow-dirty
```

### Configuration

Add to `Cargo.toml`:

```toml
[lints.clippy]
all = "warn"
pedantic = "warn"
unwrap_used = "deny"
expect_used = "deny"
```

Or create `.clippy.toml`:

```toml
avoid-breaking-exported-api = false
msrv = "1.70"
```

## Documentation with cargo-doc

### Generate Documentation

```bash
cargo doc --open
```

With private items:

```bash
cargo doc --document-private-items --open
```

### Check Documentation

Find broken links and missing docs:

```bash
cargo rustdoc -- -D warnings
```

## Benchmarking with cargo-criterion

### Installation

```bash
cargo install cargo-criterion
```

### Usage

Run benchmarks:

```bash
cargo criterion
```

### Benchmark Example

Create `benches/my_benchmark.rs`:

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn fibonacci(n: u64) -> u64 {
    match n {
        0 => 1,
        1 => 1,
        n => fibonacci(n-1) + fibonacci(n-2),
    }
}

fn benchmark(c: &mut Criterion) {
    c.bench_function("fib 20", |b| b.iter(|| fibonacci(black_box(20))));
}

criterion_group!(benches, benchmark);
criterion_main!(benches);
```

## Dependency Management with cargo-outdated

### Installation

```bash
cargo install cargo-outdated
```

### Usage

Check for outdated dependencies:

```bash
cargo outdated
```

Show only root dependencies:

```bash
cargo outdated --root-deps-only
```

## Binary Size Analysis with cargo-bloat

### Installation

```bash
cargo install cargo-bloat
```

### Usage

Show largest functions:

```bash
cargo bloat --release -n 20
```

Show largest crates:

```bash
cargo bloat --release --crates
```

### Size Comparison

Compare sizes between releases:

```bash
cargo bloat --release --crates > before.txt
# Make changes
cargo bloat --release --crates > after.txt
diff before.txt after.txt
```

## Dependency Tree with cargo-tree

### Usage

View full dependency tree:

```bash
cargo tree
```

Find duplicate dependencies:

```bash
cargo tree --duplicates
```

Find why a dependency is included:

```bash
cargo tree --invert tokio
```

## Watch Mode with cargo-watch

### Installation

```bash
cargo install cargo-watch
```

### Usage

Auto-rebuild on changes:

```bash
cargo watch -x build
```

Auto-test on changes:

```bash
cargo watch -x test
```

Run multiple commands:

```bash
cargo watch -x check -x test -x clippy
```

## Memory Profiling with cargo-valgrind

### Installation (Linux)

```bash
sudo apt install valgrind
cargo install cargo-valgrind
```

### Usage

```bash
cargo valgrind run
```

## LLVM Coverage with cargo-llvm-cov

### Installation

```bash
cargo install cargo-llvm-cov
```

### Usage

More accurate coverage than tarpaulin for some cases:

```bash
cargo llvm-cov
```

Generate HTML report:

```bash
cargo llvm-cov --html
```

## Recommended CI Pipeline

Example complete CI configuration using these tools:

```yaml
name: CI

on: [push, pull_request]

jobs:
  check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt, clippy
      
      - name: Format check
        run: cargo fmt --check
      
      - name: Clippy
        run: cargo clippy -- -D warnings
      
      - name: Build
        run: cargo build --release
      
      - name: Test
        run: cargo test

  coverage:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      
      - name: Install tarpaulin
        run: cargo install cargo-tarpaulin
      
      - name: Coverage
        run: cargo tarpaulin --out Xml --fail-under 70
      
      - uses: codecov/codecov-action@v3

  security:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      
      - name: Install audit
        run: cargo install cargo-audit
      
      - name: Security audit
        run: cargo audit --deny warnings
```

## Quick Reference

| Tool | Purpose | Command |
|------|---------|---------|
| cargo-tarpaulin | Code coverage | `cargo tarpaulin` |
| cargo-audit | Security vulnerabilities | `cargo audit` |
| cargo-deny | License/dependency rules | `cargo deny check` |
| cargo-fmt | Code formatting | `cargo fmt` |
| cargo-clippy | Linting | `cargo clippy` |
| cargo-doc | Documentation | `cargo doc --open` |
| cargo-criterion | Benchmarking | `cargo criterion` |
| cargo-outdated | Outdated dependencies | `cargo outdated` |
| cargo-bloat | Binary size analysis | `cargo bloat --release` |
| cargo-tree | Dependency tree | `cargo tree` |
| cargo-watch | Auto-rebuild | `cargo watch -x build` |
| cargo-llvm-cov | LLVM coverage | `cargo llvm-cov` |

## Installation Script

Install all recommended tools at once:

```bash
#!/bin/bash
# install-cargo-tools.sh

cargo install cargo-tarpaulin
cargo install cargo-audit
cargo install cargo-deny
cargo install cargo-outdated
cargo install cargo-bloat
cargo install cargo-watch
cargo install cargo-criterion
cargo install cargo-llvm-cov
```

## Next Steps

After setting up these tools:

1. Run `cargo audit` regularly to catch security issues
2. Add `cargo tarpaulin` to your CI pipeline
3. Use `cargo clippy` before every commit
4. Set up pre-commit hooks for automatic formatting

See [Building from Source](./building.md) for build-specific information.
