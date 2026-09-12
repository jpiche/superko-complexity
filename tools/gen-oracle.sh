#!/bin/sh
# Generate the Lean oracle fixtures under `test_data/lean-oracle/`.
#
# For each board with `m * n <= 6`, run `lean/Oracle.lean` through the Lean
# interpreter and write its tables to `test_data/lean-oracle/<m>x<n>.txt`,
# under a header naming the toolchain and the SHA-256 of the three Lean files
# whose definitions the tables come from.
#
# The grade is `observed` and stays `observed`: the Lean *compiler* evaluated
# these rows, which is the trust surface `native_decide` has and not the
# kernel's. A row here supports no ledger entry. It is a test oracle, and what
# it is for is catching a Rust mirror that has drifted from the Lean it claims
# to mirror.
#
# Usage: tools/gen-oracle.sh [<m>x<n> ...]
#        tools/gen-oracle.sh            # every board with m * n <= 6
set -eu

root=$(cd "$(dirname "$0")/.." && pwd)
lean="$root/lean"
out="$root/test_data/lean-oracle"
oracle="$lean/Oracle.lean"

# Every board with `m * n <= 6`, smallest first, so a failure appears on the
# cheapest board that has it.
boards="1x1 1x2 2x1 1x3 3x1 1x4 4x1 2x2 1x5 5x1 1x6 6x1 2x3 3x2"
[ $# -eq 0 ] || boards="$*"

for f in "$oracle" "$lean/lean-toolchain" "$lean/SuperkoComplexity/Defs.lean" \
         "$lean/SuperkoComplexity/Basic.lean" "$lean/SuperkoComplexity/Decide.lean"; do
    [ -f "$f" ] || { echo "gen-oracle: missing $f" >&2; exit 1; }
done

# SHA-256 of a file, without the file name: `sha256sum` where there is one,
# `shasum -a 256` otherwise. A third spelling is a failure rather than a
# silently different digest.
sha256() {
    if command -v sha256sum >/dev/null 2>&1; then
        sha256sum "$1" | cut -d' ' -f1
    elif command -v shasum >/dev/null 2>&1; then
        shasum -a 256 "$1" | cut -d' ' -f1
    else
        echo "gen-oracle: no sha256sum and no shasum on PATH" >&2
        exit 1
    fi
}

toolchain=$(cat "$lean/lean-toolchain")
defs_sha=$(sha256 "$lean/SuperkoComplexity/Defs.lean")
basic_sha=$(sha256 "$lean/SuperkoComplexity/Basic.lean")
decide_sha=$(sha256 "$lean/SuperkoComplexity/Decide.lean")

mkdir -p "$out"
tmp="$root/data/oracle"
mkdir -p "$tmp"

for board in $boards; do
    m=${board%x*}
    n=${board#*x}
    case "$m$n" in
        *[!0-9]*) echo "gen-oracle: $board is not a board" >&2; exit 1 ;;
    esac

    body="$tmp/$board.body"
    err="$tmp/$board.err"
    started=$(date +%s)
    ( cd "$lean" && lake env lean --run Oracle.lean "$m" "$n" ) >"$body" 2>"$err" || {
        echo "gen-oracle: $board: the oracle exited nonzero" >&2
        cat "$err" >&2
        exit 1
    }
    # Lean prints warnings to stdout, where they would land in the fixture, so
    # anything the elaborator has to say is a failure rather than a comment.
    if grep -q ': \(warning\|error\): ' "$body"; then
        echo "gen-oracle: $board: the elaborator reported something:" >&2
        grep ': \(warning\|error\): ' "$body" >&2
        exit 1
    fi
    [ -s "$err" ] && { echo "gen-oracle: $board: stderr was not empty:" >&2; cat "$err" >&2; exit 1; }
    elapsed=$(( $(date +%s) - started ))

    {
        echo "# superko-oracle 1"
        echo "# board: $m $n"
        echo "# generated-by: tools/gen-oracle.sh"
        echo "# lean-toolchain: $toolchain"
        echo "# defs-sha256: $defs_sha"
        echo "# basic-sha256: $basic_sha"
        echo "# decide-sha256: $decide_sha"
        echo "# grade: observed"
        echo "# note: evaluated by the Lean compiler (#eval), not the kernel; a test oracle, evidence for no ledger row"
        echo "# run-by: lake env lean --run lean/Oracle.lean $m $n"
        cat "$body"
    } > "$out/$board.txt"

    rows=$(grep -c -v '^#' "$out/$board.txt" || true)
    echo "gen-oracle: $board: $rows rows in ${elapsed}s -> test_data/lean-oracle/$board.txt"
done
