#!/bin/bash

# Script to generate code coverage reports locally
# Usage: ./scripts/coverage.sh [html|xml|lcov]

set -e

FORMAT=${1:-html}

echo "Generating code coverage report in $FORMAT format..."

# Install tarpaulin if not already installed
if ! command -v cargo-tarpaulin &> /dev/null; then
    echo "Installing cargo-tarpaulin..."
    cargo install cargo-tarpaulin
fi

case $FORMAT in
    "html")
        echo "Generating HTML coverage report..."
        cargo tarpaulin --verbose --all-features --workspace --timeout 120 --out html
        echo "Coverage report generated in target/tarpaulin/tarpaulin-report.html"
        ;;
    "xml")
        echo "Generating XML coverage report..."
        cargo tarpaulin --verbose --all-features --workspace --timeout 120 --out xml
        echo "Coverage report generated in cobertura.xml"
        ;;
    "lcov")
        echo "Generating LCOV coverage report..."
        cargo tarpaulin --verbose --all-features --workspace --timeout 120 --out lcov
        echo "Coverage report generated in lcov.info"
        ;;
    *)
        echo "Unknown format: $FORMAT"
        echo "Usage: $0 [html|xml|lcov]"
        exit 1
        ;;
esac

echo "Coverage report generation complete!"
