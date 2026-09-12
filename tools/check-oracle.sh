#!/bin/sh
# Refuse a Lean oracle fixture whose recorded hashes do not match the working
# tree.
#
# Each file under `test_data/lean-oracle/` records the SHA-256 of `Defs.lean`,
# `Basic.lean` and `Decide.lean` as they stood when it was generated. A change
# to any of those three changes what the rows mean, and a fixture that still
# carried the old digest would be a test passing against a definition that no
# longer exists. This gate holds the two together; `tools/gen-oracle.sh`
# regenerates.
#
# It also checks the shape of the header — the format marker, the board
# against the file name, and that the grade is still `observed`. The grade is
# not a detail: these rows were produced by the Lean compiler, not its kernel,
# and a fixture claiming otherwise would be claiming more than was done.
#
# What this gate does NOT do is check a single row. That is
# `crates/superko-rules/tests/lean_oracle.rs`, which replays every row against
# the Rust mirror.
#
# Usage: tools/check-oracle.sh
set -eu

root=$(cd "$(dirname "$0")/.." && pwd)
lean="$root/lean"
dir="$root/test_data/lean-oracle"

[ -d "$dir" ] || { echo "check-oracle: missing $dir" >&2; exit 1; }

sha256() {
    if command -v sha256sum >/dev/null 2>&1; then
        sha256sum "$1" | cut -d' ' -f1
    elif command -v shasum >/dev/null 2>&1; then
        shasum -a 256 "$1" | cut -d' ' -f1
    else
        echo "check-oracle: no sha256sum and no shasum on PATH" >&2
        exit 1
    fi
}

defs_sha=$(sha256 "$lean/SuperkoComplexity/Defs.lean")
basic_sha=$(sha256 "$lean/SuperkoComplexity/Basic.lean")
decide_sha=$(sha256 "$lean/SuperkoComplexity/Decide.lean")

fail=0
count=0

note() { echo "check-oracle: $1" >&2; fail=1; }

field() { sed -n "s/^# $2: //p" "$1" | head -1; }

for f in "$dir"/*.txt; do
    [ -e "$f" ] || { echo "check-oracle: no fixtures in $dir" >&2; exit 1; }
    name=$(basename "$f" .txt)
    count=$((count + 1))

    [ "$(head -1 "$f")" = "# superko-oracle 1" ] ||
        note "$name: the first line is not '# superko-oracle 1'"

    board=$(field "$f" board)
    m=${board% *}
    n=${board#* }
    [ "${m}x${n}" = "$name" ] ||
        note "$name: the header says board '$board', which is not the file name"

    [ "$(field "$f" grade)" = "observed" ] ||
        note "$name: the grade is not 'observed'"

    for triple in "defs:Defs:$defs_sha" "basic:Basic:$basic_sha" "decide:Decide:$decide_sha"; do
        which=${triple%%:*}
        rest=${triple#*:}
        file=${rest%%:*}
        want=${rest#*:}
        got=$(field "$f" "$which-sha256")
        [ -n "$got" ] || { note "$name: no '# $which-sha256' line"; continue; }
        [ "$got" = "$want" ] ||
            note "$name: $which-sha256 is $got; lean/SuperkoComplexity/$file.lean now hashes to $want. Regenerate with tools/gen-oracle.sh $name"
    done

    grep -q '^resolve ' "$f" || note "$name: no resolve rows"
    grep -q '^playable ' "$f" || note "$name: no playable rows"
    grep -q '^area ' "$f" || note "$name: no area rows"
    grep -q '^play ' "$f" || note "$name: no play rows"

    # The `count-table` header is a promise about the rows. A fixture that had
    # lost its `count` rows would otherwise pass every gate: the replay test
    # requires a count row somewhere in the set, not in this file.
    counts=$(field "$f" count-table)
    case "$counts" in
        yes*) grep -q '^count ' "$f" ||
            note "$name: the header says 'count-table: yes' and there are no count rows" ;;
        no*) ! grep -q '^count ' "$f" ||
            note "$name: the header says 'count-table: no' and there are count rows" ;;
        *) note "$name: no '# count-table' line, or one that is neither yes nor no" ;;
    esac
done

if [ "$fail" -ne 0 ]; then
    echo "check-oracle: FAILED" >&2
    exit 1
fi

echo "check-oracle: $count fixtures agree with the working tree's Defs.lean, Basic.lean and Decide.lean"
