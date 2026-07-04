#!/bin/bash
# Quality check script for hello-euclid
# Runs Clippy, formatting check, dependency audit, and denial checks

set -e

echo "🔍 Running Rust quality checks..."
echo ""

echo "📋 Checking code with Clippy (all targets, warnings as errors)..."
cargo clippy --all-targets -- -D warnings -W clippy::pedantic
echo "✅ Clippy passed"
echo ""

echo "🎨 Checking code formatting..."
cargo fmt --check
echo "✅ Formatting check passed"
echo ""

echo "🚨 Checking for security advisories..."
cargo audit
echo "✅ No known vulnerabilities"
echo ""

echo "📦 Checking dependencies (licenses, banned crates, duplicates)..."
cargo deny check
echo "✅ Dependency check passed"
echo ""

echo "🧹 Checking for unused dependencies..."
if command -v cargo-machete &> /dev/null; then
    cargo machete
    echo "✅ No unused dependencies"
else
    echo "⚠️  cargo-machete not installed (optional, install with: cargo install cargo-machete)"
fi
echo ""

echo "🎉 All quality checks passed!"
