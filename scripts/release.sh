#!/bin/sh
# Cuts a release: bumps the version, rewrites CHANGELOG.md from the commit
# history, commits and tags. Pushing the tag is left to you, and it is what
# starts the release workflow:  git push --follow-tags
#
#   scripts/release.sh 0.2.0
set -eu

new="${1:?usage: scripts/release.sh <version>  (for example 0.2.0)}"
case "$new" in v*) new="${new#v}" ;; esac
tag="v$new"

[ -z "$(git status --porcelain --untracked-files=no)" ] || { echo "The working tree has uncommitted changes." >&2; exit 1; }
git rev-parse "$tag" >/dev/null 2>&1 && { echo "$tag already exists." >&2; exit 1; }
command -v git-cliff >/dev/null || { echo "git-cliff is needed: cargo install git-cliff" >&2; exit 1; }

sed -i "s/^version = \".*\"$/version = \"$new\"/" Cargo.toml
cargo check --workspace --quiet   # refreshes Cargo.lock

git add Cargo.toml Cargo.lock
git commit -q -m "chore(release): $tag"
git-cliff --tag "$tag" --output CHANGELOG.md
git add CHANGELOG.md
git commit -q --amend --no-edit
git tag -a "$tag" -m "$tag"

echo "Tagged $tag. Review with 'git show', then publish with: git push --follow-tags"
