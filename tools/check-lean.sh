#!/bin/sh
# Gate the Lean development.
#
#   1. `lake build` is clean.
#   2. No `sorry` in a committed file.
#   3. The axiom dump matches results/axioms.txt — a theorem that acquired a
#      dependency on `Lean.ofReduceBool` (native_decide) or anything else
#      beyond propext / Classical.choice / Quot.sound shows up as a diff.
#
# Usage: tools/check-lean.sh
set -eu

root=$(cd "$(dirname "$0")/.." && pwd)
cd "$root/lean"

fail=0

# --- 2. sorry ---------------------------------------------------------------
# Checked first: it needs no toolchain, and it is the failure that matters most.
hits=$(grep -rn --include='*.lean' --exclude-dir=.lake \
    -E '(^|[^A-Za-z_])sorry([^A-Za-z_]|$)' . || true)
if [ -n "$hits" ]; then
    echo "check-lean: sorry in committed files:" >&2
    echo "$hits" >&2
    fail=1
fi

# --- 1. build ---------------------------------------------------------------
if ! command -v lake >/dev/null 2>&1; then
    echo "check-lean: no lake on PATH — cannot check the build or the axioms." >&2
    echo "check-lean: install elan (README.md, 'Building'), or set" >&2
    echo "check-lean: SUPERKO_ALLOW_NO_LEAN=1 to accept the sorry scan alone." >&2
    # Fail closed. A gate that passes because its tool is absent has checked
    # nothing, and reports that as success.
    if [ "${SUPERKO_ALLOW_NO_LEAN:-}" = 1 ]; then
        echo "check-lean: SUPERKO_ALLOW_NO_LEAN=1 — sorry scan only." >&2
        exit "$fail"
    fi
    exit 1
fi

if ! lake build; then
    echo "check-lean: lake build failed" >&2
    exit 1
fi

# --- 3. axioms --------------------------------------------------------------
# Axioms.lean is not part of the library: elaborate it directly, because a
# cached `lake build` prints nothing and would make this check vacuous.
expected="$root/results/axioms.txt"
if [ ! -f "$expected" ]; then
    echo "check-lean: no results/axioms.txt — record the axiom dump." >&2
    exit 1
fi

tmp=${TMPDIR:-/tmp}/check-lean.$$
lake env lean Axioms.lean > "$tmp" 2>&1 || {
    echo "check-lean: Axioms.lean failed to elaborate" >&2
    cat "$tmp" >&2; rm -f "$tmp"; exit 1
}

if ! sed '/^#/d' "$expected" | diff -u - "$tmp" >/dev/null 2>&1; then
    echo "check-lean: axiom dump differs from results/axioms.txt:" >&2
    sed '/^#/d' "$expected" | diff -u - "$tmp" >&2 || true
    echo "check-lean: if intended, update results/axioms.txt and say why in the commit." >&2
    fail=1
fi

if grep -q 'ofReduceBool' "$tmp"; then
    echo "check-lean: a theorem depends on Lean.ofReduceBool (native_decide)." >&2
    echo "check-lean: see the native_decide rule in CLAUDE.md and docs/trusted-base.md." >&2
    fail=1
fi
rm -f "$tmp"

[ "$fail" -eq 0 ] || exit 1
echo "check-lean: build clean, no sorry, axioms as recorded"
