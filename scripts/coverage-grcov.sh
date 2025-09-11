#!/bin/bash

# Coverage script using grcov (Mozilla's Rust coverage tool)
# This is often easier to install and use than tarpaulin

set -e

echo "Generating code coverage using grcov..."

# Install grcov if not present
if ! command -v grcov &> /dev/null; then
    echo "Installing grcov..."
    cargo install grcov
fi

# Clean previous coverage data
rm -rf target/coverage
mkdir -p target/coverage

# Set up environment for coverage collection
export CARGO_INCREMENTAL=0
export RUSTFLAGS="-Cinstrument-coverage"
export LLVM_PROFILE_FILE="target/coverage/codemarks-%p-%m.profraw"

echo "Running tests with coverage instrumentation..."
cargo test --all-features

echo "Generating coverage reports..."

# Generate HTML report
grcov target/coverage \
    --binary-path ./target/debug/ \
    --source-dir . \
    --output-types html \
    --branch \
    --ignore-not-existing \
    --ignore '*/.cargo/*' \
    --ignore '*/target/*' \
    --ignore '*/.rustup/*' \
    --ignore '*/rustlib/*' \
    --ignore '*/lib/rustlib/*' \
    --ignore '*/toolchains/*' \
    --excl-line GRCOV_EXCL_LINE \
    --excl-start GRCOV_EXCL_START \
    --excl-stop GRCOV_EXCL_STOP \
    --output-path target/coverage/html

# Generate lcov report for potential CI integration
grcov target/coverage \
    --binary-path ./target/debug/ \
    --source-dir . \
    --output-types lcov \
    --branch \
    --ignore-not-existing \
    --ignore '*/.cargo/*' \
    --ignore '*/target/*' \
    --ignore '*/.rustup/*' \
    --ignore '*/rustlib/*' \
    --ignore '*/lib/rustlib/*' \
    --ignore '*/toolchains/*' \
    --excl-line GRCOV_EXCL_LINE \
    --excl-start GRCOV_EXCL_START \
    --excl-stop GRCOV_EXCL_STOP \
    --output-path target/coverage/lcov.info

# Generate text summary
grcov target/coverage \
    --binary-path ./target/debug/ \
    --source-dir . \
    --output-types markdown \
    --branch \
    --ignore-not-existing \
    --ignore '*/.cargo/*' \
    --ignore '*/target/*' \
    --ignore '*/.rustup/*' \
    --ignore '*/rustlib/*' \
    --ignore '*/lib/rustlib/*' \
    --ignore '*/toolchains/*' \
    --excl-line GRCOV_EXCL_LINE \
    --excl-start GRCOV_EXCL_START \
    --excl-stop GRCOV_EXCL_STOP

echo ""
echo "Coverage reports generated:"
echo "- HTML report: target/coverage/html/index.html"
echo "- LCOV report: target/coverage/lcov.info"
echo ""
echo "Open target/coverage/html/index.html in your browser to view the detailed report."
