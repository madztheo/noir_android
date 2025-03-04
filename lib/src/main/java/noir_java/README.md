# Noir JNI Library Tests

This directory contains tests for the Noir JNI library.

## Running Rust Tests

Rust tests can be run using cargo:

```bash
cd lib/src/main/java/noir_java
cargo test
```

## Test Structure

### Rust Tests

- `lib.rs`: Contains direct tests for the JNI functions
- `noir_tests.rs`: Contains tests for the core noir_rs functionality
- `test_utils.rs`: Contains utility functions for testing
