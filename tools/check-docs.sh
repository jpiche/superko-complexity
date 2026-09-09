#!/bin/sh
# Enforce the mechanically checkable parts of docs/style.md.
#
#   1. No hedge-free assertion words: clearly, obviously, evidently,
#      of course, it follows that. Each marks a place a status was skipped.
#
#      Exempt: negated uses ("not obviously equivalent" expresses uncertainty
#      rather than skipping a status), and mentions inside backticks, double
#      quotes or *emphasis*, which is how these files quote the rule itself.
#
#   2. Every claim reference (C-<n>) in prose names a claim the ledger has.
#
#   3. American English. The project settled on it; `Color` is a Lean
#      identifier, so drift here is a rename, not a typo.
#
#   4. Every notebook entry and experiment names its author and the specific
#      models used. See the Attribution section of CLAUDE.md.
#
# A clean run means no *detectable* violation, not a clean document. The status
# rule itself is not mechanically checkable — that is what review is for.
#
# Usage: tools/check-docs.sh
set -eu

root=$(cd "$(dirname "$0")/.." && pwd)
ledger="$root/docs/claim-ledger.md"
fail=0

files=$(find "$root/docs" "$root/notebook" "$root/proofs" "$root/experiments" \
             -name '*.md' 2>/dev/null | sort)
files="$files $root/README.md $root/CLAUDE.md"

# --- 1. banned words --------------------------------------------------------
for f in $files; do
    [ -f "$f" ] || continue
    hits=$(awk '
        {
            line = $0
            gsub(/`[^`]*`/, " ", line)     # code spans
            gsub(/"[^"]*"/, " ", line)     # quoted mentions
            gsub(/\*[^*]*\*/, " ", line)   # emphasis
            gsub(/not[ ]+(obviously|clearly|evidently)/, " ", line)
            if (match(tolower(line), /(^|[^a-z])(clearly|obviously|evidently|of course|it follows that)([^a-z]|$)/))
                printf "%d:%s\n", NR, $0
        }' "$f")
    if [ -n "$hits" ]; then
        echo "check-docs: ${f#"$root"/}: banned assertion word (docs/style.md)" >&2
        echo "$hits" | sed 's/^/    /' >&2
        fail=1
    fi
done

# --- 2. claim references resolve -------------------------------------------
known=$(grep -oE '^\| *C-[0-9]+' "$ledger" | tr -d '| ' | sort -u)
for f in $files; do
    [ -f "$f" ] || continue
    [ "$f" = "$ledger" ] && continue
    for ref in $(grep -oE '\bC-[0-9]+\b' "$f" | sort -u); do
        if ! printf '%s\n' "$known" | grep -qx "$ref"; then
            echo "check-docs: ${f#"$root"/}: references $ref, which the ledger does not have" >&2
            fail=1
        fi
    done
done

# --- 3. American English ----------------------------------------------------
# Source as well as prose: several of these appear in identifiers.
british='behaviour|colour|licence|modelled|labelled|analyse|centre|neighbour|favour|defence|organis|recognis|formalis|specialis|honour|offence|travelled|fulfil'
# `data/` is gitignored scratch, and CLAUDE.md directs agents to write there.
# It may hold vendored third-party sources, which this project does not spell.
spell=$(find "$root" \( -name '*.md' -o -name '*.lean' -o -name '*.rs' -o -name '*.sh' \) \
        -not -path '*/.lake/*' -not -path '*/target/*' -not -path '*/.git/*' \
        -not -path "$root/data/*" 2>/dev/null \
        | xargs grep -niE "$british" 2>/dev/null | grep -v 'check-docs.sh' || true)
if [ -n "$spell" ]; then
    echo "check-docs: British spelling (the project uses American English)" >&2
    echo "$spell" | sed "s|$root/||" | sed 's/^/    /' >&2
    fail=1
fi

# --- 4. attribution ---------------------------------------------------------
attrib=$(find "$root/notebook" -name '*.md' ! -name 'README.md' 2>/dev/null
         find "$root/experiments" -name 'README.md' 2>/dev/null)
for f in $attrib; do
    [ -f "$f" ] || continue
    miss=""
    grep -q '^\*\*Author:\*\*' "$f" || miss="Author"
    grep -q '^\*\*Models:\*\*' "$f" || miss="$miss Models"
    if [ -n "$miss" ]; then
        echo "check-docs: ${f#"$root"/}: missing attribution line(s):$miss" >&2
        echo "check-docs: see the Attribution section of CLAUDE.md" >&2
        fail=1
    fi
done

[ "$fail" -eq 0 ] || { echo "check-docs: FAILED" >&2; exit 1; }
echo "check-docs: no detectable violations"
