#!/bin/sh
# Hold `crates/superko-rules` to being a mirror of the audit target.
#
#   1. Every `def`, `abbrev`, `structure` and `inductive` declared in
#      `lean/SuperkoComplexity/Defs.lean` is named by a `**Mirrors**` doc line
#      somewhere under `crates/superko-rules/src`, or is on the
#      MIRROR-ELSEWHERE list below with a reason. An item that quietly stops
#      being mirrored is the drift this gate exists to prevent.
#
#   2. Every `**Mirrors**` doc line names an item `Defs.lean` actually
#      declares, so a rename in Lean breaks the build rather than leaving a
#      Rust comment pointing at nothing. The `**Mirrors (Decide.lean)**` form
#      is checked the same way against `Decide.lean`, the computable layer.
#
#   3. The set of `// DIVERGENCE: <slug>` markers in the crate equals the set
#      of slugs `divergence.rs` declares. A departure from a literal reading of
#      `Defs.lean` that is not registered is either a bug or an unrecorded
#      change of meaning, and this gate cannot tell which, so it fails on both.
#
# A clean run means the two files still line up item for item. It says nothing
# about whether a mirrored item mirrors its Lean counterpart *correctly* —
# that is what the crate's tests and, above them, the acceptance suite are for.
#
# Usage: tools/check-mirror.sh
set -eu

root=$(cd "$(dirname "$0")/.." && pwd)
defs="$root/lean/SuperkoComplexity/Defs.lean"
decide="$root/lean/SuperkoComplexity/Decide.lean"
src="$root/crates/superko-rules/src"
divergence="$src/divergence.rs"

for f in "$defs" "$decide" "$divergence"; do
    [ -f "$f" ] || { echo "check-mirror: missing $f" >&2; exit 1; }
done

# --- MIRROR-ELSEWHERE -------------------------------------------------------
# `Defs.lean` items that `superko-rules` deliberately does not hold, one per
# line as `<name>|<reason>`. Each name must still be a declaration of
# `Defs.lean`; a stale entry fails the gate.
mirror_elsewhere='
Repetition|the Lean item is the *type* of a repetition rule, State -> Move -> Prop; config::Repetition names one of the two instances, and the type-level item is a parameter of WinsFor
WinsFor|the winning-strategy recursion is the solver'\''s, and superko-rules holds no search
BlackWins|the decision problem is stated over WinsFor and travels with it
BlackWinsPSK|as BlackWins, under the comparison rule
'

rsfiles=$(find "$src" -name '*.rs' | sort)

# shellcheck disable=SC2086
MIRROR_ELSEWHERE="$mirror_elsewhere" awk '
function note(msg) { print "check-mirror: " msg > "/dev/stderr"; fail = 1 }

# A top-level Lean declaration: the keyword at column zero, then the name.
function declname(line,   s) {
    s = line
    sub(/^(noncomputable[ \t]+)?(private[ \t]+)?(protected[ \t]+)?/, "", s)
    sub(/^(def|abbrev|structure|inductive)[ \t]+/, "", s)
    sub(/[^A-Za-z0-9_'"'"'.].*$/, "", s)
    return s
}

function isdecl(line) {
    return line ~ /^(noncomputable[ \t]+)?(private[ \t]+)?(protected[ \t]+)?(def|abbrev|structure|inductive)[ \t]+[A-Za-z_]/
}

FILENAME ~ /Defs\.lean$/ {
    if (isdecl($0)) { n = declname($0); if (n != "") { defs[n] = 1; order[++ndefs] = n } }
    next
}

FILENAME ~ /Decide\.lean$/ {
    if (isdecl($0)) { n = declname($0); if (n != "") decide[n] = 1 }
    next
}

FILENAME ~ /divergence\.rs$/ && inslug {
    if ($0 ~ /^    }$/) { inslug = 0; next }
    if (match($0, /"[a-z][a-z0-9-]*"/)) slugs[substr($0, RSTART + 1, RLENGTH - 2)] = 1
}

FILENAME ~ /divergence\.rs$/ && $0 ~ /fn slug\(/ { inslug = 1 }

FILENAME ~ /\.rs$/ {
    short = FILENAME; sub(/^.*\/crates\//, "crates/", short)

    if (match($0, /\*\*Mirrors\*\* `Superko\.[^`]+`/)) {
        s = substr($0, RSTART, RLENGTH)
        sub(/^\*\*Mirrors\*\* `Superko\./, "", s)
        sub(/`$/, "", s)
        mirrored[s] = 1
        where[s] = short ":" FNR
    }

    if (match($0, /\*\*Mirrors \(Decide\.lean\)\*\* `Superko\.[^`]+`/)) {
        s = substr($0, RSTART, RLENGTH)
        sub(/^\*\*Mirrors \(Decide\.lean\)\*\* `Superko\./, "", s)
        sub(/`$/, "", s)
        mirroredDecide[s] = 1
        whereDecide[s] = short ":" FNR
    }

    if (match($0, /^[ \t]*\/\/ DIVERGENCE: [a-z][a-z0-9-]*[ \t]*$/)) {
        s = $0
        sub(/^[ \t]*\/\/ DIVERGENCE: /, "", s)
        sub(/[ \t]*$/, "", s)
        markers[s] = 1
        markerCount[s]++
    }
}

END {
    if (ndefs == 0) { note("no declarations found in Defs.lean"); exit 1 }

    # The MIRROR-ELSEWHERE list.
    n = split(ENVIRON["MIRROR_ELSEWHERE"], lines, "\n")
    for (i = 1; i <= n; i++) {
        if (lines[i] == "") continue
        p = index(lines[i], "|")
        if (p == 0) { note("MIRROR-ELSEWHERE entry without a reason: " lines[i]); continue }
        name = substr(lines[i], 1, p - 1)
        reason = substr(lines[i], p + 1)
        if (reason == "") { note("MIRROR-ELSEWHERE " name ": empty reason") }
        if (!(name in defs)) { note("MIRROR-ELSEWHERE " name ": Defs.lean has no such declaration") }
        exempt[name] = 1
        nexempt++
    }

    # 1. Every Defs.lean item is mirrored or exempt.
    for (i = 1; i <= ndefs; i++) {
        name = order[i]
        if (!(name in mirrored) && !(name in exempt))
            note("Defs.lean declares " name ", and no **Mirrors** line under crates/superko-rules/src names it")
    }

    # 2. Every **Mirrors** line names something that exists.
    for (name in mirrored)
        if (!(name in defs))
            note(where[name] ": **Mirrors** `Superko." name "`, which Defs.lean does not declare")
    for (name in mirroredDecide)
        if (!(name in decide))
            note(whereDecide[name] ": **Mirrors (Decide.lean)** `Superko." name "`, which Decide.lean does not declare")

    # 3. Markers and slugs are the same set.
    for (s in slugs)
        if (!(s in markers))
            note("divergence.rs declares the slug " s ", and no // DIVERGENCE: " s " marker appears in the crate")
    for (s in markers)
        if (!(s in slugs))
            note("// DIVERGENCE: " s " is marked in the crate, and divergence.rs does not declare that slug")

    nslugs = 0; for (s in slugs) nslugs++
    if (nslugs == 0) { note("divergence.rs declares no slugs; is the slug() match still there?") }

    if (fail) { print "check-mirror: FAILED" > "/dev/stderr"; exit 1 }
    printf "check-mirror: %d Defs.lean items, %d mirrored, %d elsewhere; %d divergence slugs\n", \
        ndefs, ndefs - nexempt, nexempt, nslugs
}
' "$defs" "$decide" $rsfiles
