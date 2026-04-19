# Testing and Safety Tooling

This guide covers advanced testing and safety tools for General Bots development, including formal verification, undefined behavior detection, and memory safety analysis.

## Overview

General Bots prioritizes safety and correctness. While Rust's type system catches many errors at compile time, additional tooling can detect subtle bugs and verify critical code paths.

## Standard Testing

### Unit Tests

```bash
cargo test
```

Run tests for a specific package:

```bash
cargo test -p botserver
cargo test -p botlib
```

### Integration Tests

```bash
cargo test --test integration
```

## Miri - Undefined Behavior Detection

[Miri](https://github.com/rust-lang/miri) is an interpreter for Rust's mid-level intermediate representation (MIR) that detects undefined behavior.

### When to Use Miri

- Testing unsafe code blocks
- Detecting memory leaks in complex data structures
- Finding data races in concurrent code
- Validating pointer arithmetic

### Running Miri

```bash
# Install Miri
rustup +nightly component add miri

# Run tests under Miri
cargo +nightly miri test

# Run specific test
cargo +nightly miri test test_name

# Run with isolation disabled (for FFI)
cargo +nightly miri test -- -Zmiri-disable-isolation
```

### Miri Limitations

- **Slow execution** - 10-100x slower than native
- **No FFI** - Cannot call C libraries
- **No I/O** - Cannot perform actual I/O operations
- **Nightly only** - Requires nightly Rust

### Recommended Usage for General Bots

Miri is valuable for testing:
- `botlib` data structures and parsing logic
- BASIC interpreter core in `botserver`
- Custom serialization/deserialization code

Not suitable for:
- HTTP handlers (requires I/O)
- Database operations (requires FFI)
- Full integration tests

## Kani - Formal Verification

[Kani](https://github.com/model-checking/kani) is a model checker that mathematically proves code properties.

### When to Use Kani

- Verifying critical algorithms
- Proving absence of panics
- Checking invariants in state machines
- Validating security-critical code

### Running Kani

```bash
# Install Kani
cargo install --locked kani-verifier
kani setup

# Verify a function
cargo kani --function critical_function

# Verify with specific harness
cargo kani --harness verify_limits
```

### Writing Kani Proofs

```rust
#[cfg(kani)]
mod verification {
    use super::*;

    #[kani::proof]
    fn verify_loop_limit_check() {
        let iterations: u32 = kani::any();
        let max: u32 = kani::any();
        
        // Assume valid inputs
        kani::assume(max > 0);
        
        let result = check_loop_limit(iterations, max);
        
        // Prove: if iterations < max, result is Ok
        if iterations < max {
            assert!(result.is_ok());
        }
    }

    #[kani::proof]
    fn verify_rate_limiter_never_panics() {
        let count: u64 = kani::any();
        let limit: u64 = kani::any();
        
        kani::assume(limit > 0);
        
        // This should never panic regardless of inputs
        let _ = count.checked_div(limit);
    }
}
```

### Kani Limitations

- **Bounded verification** - Cannot verify unbounded loops
- **Slow** - Model checking is computationally expensive
- **Limited async support** - Async code requires special handling

### Recommended Usage for General Bots

Kani is valuable for:
- Verifying `botlib::limits` enforcement
- Proving rate limiter correctness
- Validating authentication logic invariants
- Checking BASIC parser edge cases

## AddressSanitizer (ASan)

AddressSanitizer detects memory errors at runtime.

### Running with ASan

```bash
# Build and test with AddressSanitizer
RUSTFLAGS="-Z sanitizer=address" cargo +nightly test

# For a specific package
RUSTFLAGS="-Z sanitizer=address" cargo +nightly test -p botserver
```

### What ASan Detects

- Use-after-free
- Buffer overflows
- Stack buffer overflow
- Global buffer overflow
- Use after return
- Memory leaks

### ASan Configuration

```bash
# Set ASan options
export ASAN_OPTIONS="detect_leaks=1:abort_on_error=1:symbolize=1"

# Run tests
RUSTFLAGS="-Z sanitizer=address" cargo +nightly test
```

## ThreadSanitizer (TSan)

Detects data races in multi-threaded code.

```bash
RUSTFLAGS="-Z sanitizer=thread" cargo +nightly test
```

### TSan Considerations

- Requires `--target x86_64-unknown-linux-gnu`
- Incompatible with ASan (run separately)
- Higher memory overhead

## MemorySanitizer (MSan)

Detects uninitialized memory reads.

```bash
RUSTFLAGS="-Z sanitizer=memory" cargo +nightly test
```

### MSan Requirements

- Must compile all dependencies with MSan
- Most practical for pure Rust code
- Requires nightly toolchain

## Ferrocene - Safety-Critical Rust

[Ferrocene](https://ferrocene.dev/) is a qualified Rust compiler for safety-critical systems.

### When to Consider Ferrocene

Ferrocene is relevant if General Bots is deployed in:
- Medical devices
- Automotive systems
- Aerospace applications
- Industrial control systems
- Any context requiring ISO 26262, IEC 61508, or DO-178C compliance

### Ferrocene vs Standard Rust

| Aspect | Standard Rust | Ferrocene |
|--------|---------------|-----------|
| **Qualification** | None | ISO 26262, IEC 61508 |
| **Documentation** | Community | Formal qualification docs |
| **Support** | Community | Commercial support |
| **Cost** | Free | Commercial license |
| **Traceability** | Limited | Full requirement traceability |

### Should General Bots Use Ferrocene?

**For typical SaaS deployment: No**

Ferrocene is overkill for:
- Web applications
- Business automation
- General chatbot deployments

**Consider Ferrocene if:**
- Deploying GB in safety-critical environments
- Customer requires formal certification
- Regulatory compliance mandates qualified tooling

### Alternative: Standard Rust + Testing

For most deployments, comprehensive testing provides sufficient confidence:

```bash
# Full test suite
cargo test --all

# With coverage
cargo tarpaulin --out Html

# Fuzzing critical parsers
cargo fuzz run parse_basic_script
```

## Recommended Testing Strategy

### Development (Every Commit)

```bash
cargo test
cargo clippy -- -D warnings
```

### Pre-Release

```bash
# Full test suite
cargo test --all

# Miri for unsafe code
cargo +nightly miri test -p botlib

# AddressSanitizer
RUSTFLAGS="-Z sanitizer=address" cargo +nightly test

# ThreadSanitizer (for concurrent code)
RUSTFLAGS="-Z sanitizer=thread" cargo +nightly test
```

### Critical Code Changes

```bash
# Kani for formal verification
cargo kani --function critical_function

# Extended fuzzing
cargo fuzz run target_name -- -max_total_time=3600
```

## CI/CD Integration

Example GitHub Actions workflow:

```yaml
name: Safety Tests

on:
  push:
    branches: [main]
  pull_request:

jobs:
  miri:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@nightly
        with:
          components: miri
      - run: cargo miri test -p botlib

  sanitizers:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@nightly
      - run: RUSTFLAGS="-Z sanitizer=address" cargo test

  kani:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: model-checking/kani-github-action@v1
      - run: cargo kani --tests
```

## Summary

| Tool | Purpose | When to Use |
|------|---------|-------------|
| `cargo test` | Unit/integration tests | Always |
| `miri` | Undefined behavior | Unsafe code changes |
| `kani` | Formal verification | Critical algorithms |
| `ASan` | Memory errors | Pre-release, CI |
| `TSan` | Data races | Concurrent code changes |
| `Ferrocene` | Safety certification | Regulated industries only |

## See Also

- [System Limits](../09-security/system-limits.md) - Rate limiting and resource constraints
- [Security Features](../09-security/security-features.md) - Security architecture
- [Building from Source](./building.md) - Compilation guide