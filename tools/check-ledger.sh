#!/bin/sh
# Enforce the claim ledger's invariants.
#
#   1. Every status and formalization value is one the ledger defines.
#   2. Every claim named in a depends-on column exists.
#   3. No `proved` claim depends on a claim that is not `proved` or `cited`.
#      A proof resting on folklore, a computation, or a conjecture is not a
#      proof, and this gate treats it as a build break.
#   4. Every `formalized:Name` claim names a theorem that exists in the Lean
#      sources.
#
# Usage: tools/check-ledger.sh
set -eu

root=$(cd "$(dirname "$0")/.." && pwd)
ledger="$root/docs/claim-ledger.md"

[ -f "$ledger" ] || { echo "check-ledger: no ledger at $ledger" >&2; exit 1; }

# Lean sources first so theorem names are known before the ledger is read.
# Exclude .lake: the dependency tree holds all of Mathlib, which both blows the
# argument list and would let a Mathlib theorem satisfy a `formalized:` claim.
leanfiles=$(find "$root/lean" -name '*.lean' -not -path '*/.lake/*' 2>/dev/null | sort || true)

# shellcheck disable=SC2086
awk '
function trim(s) { sub(/^[ \t]+/, "", s); sub(/[ \t]+$/, "", s); return s }
function note(msg) { print "check-ledger: " msg > "/dev/stderr"; fail = 1 }

FILENAME ~ /\.lean$/ {
    if ($0 ~ /^[ \t]*(theorem|lemma)[ \t]+[A-Za-z_]/) {
        line = $0
        sub(/^[ \t]*(theorem|lemma)[ \t]+/, "", line)
        sub(/[^A-Za-z0-9_'"'"'.].*$/, "", line)
        if (line != "") thm[line] = 1
    }
    next
}

# A claim row: six pipe-delimited columns, id matching C-<digits>.
/^\| *C-[0-9]+ *\|/ {
    n = split($0, col, "|")
    if (n < 8) { note("malformed row: " $0); next }
    id = trim(col[2]); statement = trim(col[3]); status = trim(col[4])
    formal = trim(col[5]); deps = trim(col[6]); witness = trim(col[7])

    if (id in seen)        note(id ": duplicate row")
    if (statement == "")   note(id ": empty statement")
    if (witness == "")     note(id ": empty witness column (use - if none)")

    seen[id] = 1; st[id] = status; fm[id] = formal; dp[id] = deps
    order[++count] = id
    next
}

END {
    if (count == 0) { note("no claim rows found"); exit 1 }

    for (i = 1; i <= count; i++) {
        id = order[i]

        # 1. vocabularies
        s = st[id]
        if (s != "proved" && s != "cited" && s != "folklore" && s != "computed" &&
            s != "conjecture" && s != "open" && s != "refuted")
            note(id ": unknown status \"" s "\"")
        tally[s]++

        f = fm[id]
        if (f != "formalizable" && f != "infra-gap" && f != "prose-only" &&
            f != "n/a" && f !~ /^formalized:/)
            note(id ": unknown formalization \"" f "\"")

        # 4. a formalized claim names a real theorem
        if (f ~ /^formalized:/) {
            name = substr(f, length("formalized:") + 1)
            if (!(name in thm))
                note(id ": claims formalized:" name ", but no such theorem in lean/")
        }

        # 2 and 3. dependencies
        if (dp[id] != "-" && dp[id] != "") {
            m = split(dp[id], d, ",")
            for (j = 1; j <= m; j++) {
                dep = trim(d[j])
                if (dep == "") continue
                if (!(dep in seen)) { note(id ": depends on " dep ", which has no row"); continue }
                if (st[id] == "proved" && st[dep] != "proved" && st[dep] != "cited")
                    note(id " is proved but depends on " dep ", which is " st[dep])
            }
        }
    }

    if (fail) { print "check-ledger: FAILED (" count " claims)" > "/dev/stderr"; exit 1 }

    printf "check-ledger: %d claims, invariants hold\n", count
    split("proved cited folklore computed conjecture open refuted", ord, " ")
    for (k = 1; k <= 7; k++)
        if (tally[ord[k]] > 0) printf "  %-10s %d\n", ord[k], tally[ord[k]]
}
' $leanfiles "$ledger"
