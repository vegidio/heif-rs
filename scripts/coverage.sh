#!/usr/bin/env bash
#
# Generate a code-coverage report for heif-rs using cargo-llvm-cov.
#
# Usage:
#   scripts/coverage.sh           # build an HTML report and open it in a browser
#   scripts/coverage.sh --lcov    # emit target/coverage/lcov.info instead (for tooling/CI)
#
# Coverage runs the full test suite (unit + integration). The integration tests exercise
# assets/image.jpg and assets/image.heic. The libheif static binaries are downloaded
# automatically by build.rs.
#
# The machine-generated FFI bindings are excluded: heif-rs generates them at build time with
# `bindgen` into `OUT_DIR/bindings.rs` (included by `src/sys.rs`) — hundreds of `extern`
# declarations, type aliases and constants that can't be "executed", so counting them would
# understate the real, hand-written coverage.
set -euo pipefail

cd "$(dirname "$0")/.."

# Generated FFI bindings are not meaningfully coverable; keep them out of every report.
IGNORE_REGEX='bindings\.rs'

# Ensure the LLVM coverage tooling is available.
if ! cargo llvm-cov --version >/dev/null 2>&1; then
    echo "cargo-llvm-cov not found; installing it (one-time setup)..." >&2
    rustup component add llvm-tools-preview
    cargo install cargo-llvm-cov
fi

if [[ "${1:-}" == "--lcov" ]]; then
    mkdir -p target/coverage
    cargo llvm-cov --all-features --ignore-filename-regex "$IGNORE_REGEX" \
        --lcov --output-path target/coverage/lcov.info
    echo "Wrote target/coverage/lcov.info"
else
    cargo llvm-cov --all-features --ignore-filename-regex "$IGNORE_REGEX" --html --open
    echo "HTML report written to target/llvm-cov/html/index.html"
fi
