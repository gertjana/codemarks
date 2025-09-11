#!/bin/bash

# Alternative coverage script using LLVM source-based coverage
# This uses Rust's built-in coverage capabilities

set -e

echo "Generating code coverage using LLVM source-based coverage..."

# Set up environment variables for coverage
export RUSTFLAGS="-C instrument-coverage"
export LLVM_PROFILE_FILE="target/coverage/codemarks-%p-%m.profraw"

# Create coverage directory
mkdir -p target/coverage

echo "Building with coverage instrumentation..."
cargo build --all-features

echo "Running tests with coverage..."
cargo test --all-features

echo "Generating coverage report..."

# Check if llvm-profdata and llvm-cov are available
if command -v llvm-profdata &> /dev/null && command -v llvm-cov &> /dev/null; then
    # Merge profile data
    llvm-profdata merge -sparse target/coverage/codemarks-*.profraw -o target/coverage/codemarks.profdata

    # Generate HTML report
    llvm-cov show \
        --format=html \
        --instr-profile=target/coverage/codemarks.profdata \
        --ignore-filename-regex='/.cargo/registry' \
        --ignore-filename-regex='/rustc/' \
        --show-instantiations=false \
        target/debug/deps/codemarks-* \
        --output-dir=target/coverage/html

    echo "HTML coverage report generated in target/coverage/html/index.html"

    # Generate summary
    llvm-cov report \
        --instr-profile=target/coverage/codemarks.profdata \
        --ignore-filename-regex='/.cargo/registry' \
        --ignore-filename-regex='/rustc/' \
        target/debug/deps/codemarks-*

else
    echo "Warning: llvm-profdata and llvm-cov not found."
    echo "Raw profile data saved in target/coverage/"
    echo "Install LLVM tools to generate reports."
fi

echo "Coverage generation complete!"
