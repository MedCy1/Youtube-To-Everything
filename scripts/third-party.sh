#!/bin/sh
# Writes THIRD-PARTY-LICENSES.txt: the license of every Rust dependency that
# ships in the binary, then the licenses of the fonts embedded in the interface.
# The release workflow runs it and puts the result in the archive.
#
#   scripts/third-party.sh [output-file]
set -eu

out="${1:-THIRD-PARTY-LICENSES.txt}"
command -v cargo-about >/dev/null || { echo "cargo-about is needed: cargo install cargo-about --locked --features cli" >&2; exit 1; }

cargo about generate --locked about.hbs > "$out"

{
    printf '\n================================================================================\n'
    printf 'Fonts embedded in the interface\n'
    printf '================================================================================\n'
    for font in third-party/fonts/*.txt; do
        printf '\n--- %s ---\n\n' "$(basename "$font" -LICENSE.txt)"
        cat "$font"
    done
} >> "$out"

echo "Wrote $out ($(wc -l < "$out") lines)."
