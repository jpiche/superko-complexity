#!/bin/sh
# Every number this project states in prose regenerates from a command.
#
# Each file in results/ carries a witness header:
#
#     # witness: cargo run -p superko-cli -- count-games --board 2x2 --rule psk
#     # commit: <sha at which the value was recorded>
#     # date: <ISO date>
#     #
#     <body: the output>
#
# This script re-runs each witness command and diffs its output against the
# body. A measurement nobody can reproduce is an opinion (docs/style.md).
#
# Usage: tools/verify-results.sh [file ...]
set -eu

root=$(cd "$(dirname "$0")/.." && pwd)
cd "$root"

if [ "$#" -gt 0 ]; then
    files="$*"
else
    files=$(find results -type f ! -name 'README.md' ! -name '.*' 2>/dev/null | sort || true)
fi

if [ -z "$files" ]; then
    echo "verify-results: nothing in results/ yet"
    exit 0
fi

fail=0
for f in $files; do
    cmd=$(sed -n 's/^# witness: //p' "$f" | head -1)
    if [ -z "$cmd" ]; then
        echo "verify-results: $f has no '# witness:' header" >&2
        fail=1
        continue
    fi

    body=$(sed '/^#/d' "$f")
    actual=$(sh -c "$cmd" 2>&1) || {
        echo "verify-results: $f: witness command failed: $cmd" >&2
        fail=1
        continue
    }

    if [ "$body" != "$actual" ]; then
        echo "verify-results: $f: output differs from the recorded value" >&2
        tmp=${TMPDIR:-/tmp}/verify-results.$$
        printf '%s\n' "$actual" > "$tmp"
        printf '%s\n' "$body" | diff -u - "$tmp" >&2 || true
        rm -f "$tmp"
        fail=1
        continue
    fi
    echo "verify-results: $f ok"
done

[ "$fail" -eq 0 ] || { echo "verify-results: FAILED" >&2; exit 1; }
