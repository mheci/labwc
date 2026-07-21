#!/bin/bash
set -euo pipefail
export PATH="$HOME/.cargo/bin:$PATH"
cd /home/user/labwc-rs
PASS=0; FAIL=0

check() {
    local name="$1"; shift
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "  $name"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    if "$@" > /tmp/ci_out.txt 2>&1; then
        echo "  ✅ PASSED"
        PASS=$((PASS + 1))
    else
        echo "  ❌ FAILED (exit=$?)"
        tail -20 /tmp/ci_out.txt
        FAIL=$((FAIL + 1))
    fi
    echo ""
}

echo "╔══════════════════════════════════════════╗"
echo "║   GitHub Actions CI — Local Simulation  ║"
echo "╚══════════════════════════════════════════╝"
echo ""

echo "── Job: lint ──"
check "fmt --check"           cargo fmt --all -- --check
check "clippy -D warnings"   cargo clippy --workspace --all-targets -- -D warnings
check "check --workspace"    cargo check --workspace --all-targets

echo "── Job: test ──"
check "cargo test" cargo test --workspace

echo "── Job: doc ──"
check "cargo doc" env RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --document-private-items

echo "── Job: build (release) ──"
check "cargo build --release" env RUSTFLAGS="-D warnings" cargo build --release

echo "── Post-checks ──"
echo "  Binary: $(ls -lh target/release/labwc-rs | awk '{print $5}')"
echo ""

echo "╔══════════════════════════════════════════╗"
echo "║  RESULTS: $PASS passed, $FAIL failed              ║"
echo "╚══════════════════════════════════════════╝"
[ "$FAIL" -gt 0 ] && exit 1 || exit 0
