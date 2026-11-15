#!/bin/bash
# Builds run in the Solomon-69/murmur repo itself (solomon-build.yml).
# Pushes main, starts one build and prints the newest run.
set -e
cd "$(git rev-parse --show-toplevel)"
git push origin main
gh workflow run solomon-build.yml -R Solomon-69/murmur --ref main
sleep 5
gh run list -R Solomon-69/murmur -w solomon-build.yml -L 1
