#!/bin/bash
# Builds run in the public Solomon-69/murmur-build repo (free Actions minutes).
# Pushes a one-commit snapshot of the current tree, without the private notes
# (QA/, HANDOFF files), and prints the newest run.
set -e
cd "$(git rev-parse --show-toplevel)"
git remote get-url build >/dev/null 2>&1 || git remote add build https://github.com/Solomon-69/murmur-build.git
git remote set-url build https://github.com/Solomon-69/murmur-build.git
IDX=$(mktemp); trap 'rm -f "$IDX"' EXIT
GIT_INDEX_FILE=$IDX git read-tree HEAD
GIT_INDEX_FILE=$IDX git rm -r -q --cached --ignore-unmatch QA HANDOFF.md docs/HANDOFF.md docs/proof docs/wispr-feature-audit.md .base-ast
T=$(GIT_INDEX_FILE=$IDX git write-tree)
C=$(git commit-tree "$T" -m "build snapshot $(git rev-parse --short HEAD)")
git push -f build "${C}:refs/heads/overnight-8-nov"
sleep 8
gh run list -R Solomon-69/murmur-build -L 1
