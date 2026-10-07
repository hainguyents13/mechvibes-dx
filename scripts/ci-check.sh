#!/usr/bin/env bash
#
# The compile-and-test gate that CI runs on every OS, and that you can run
# locally to get the same answer.
#
# Usage: scripts/ci-check.sh [--release]
#   --release   check and test in the release profile. The release workflow uses
#               this so the gate shares build artifacts with the package build.
#
# Does NOT install system libraries (webkit, GTK, ALSA, ...): on Linux those
# come from the package list in README.md / .github/actions/linux-system-deps.

set -euo pipefail

source "$(dirname "${BASH_SOURCE[0]}")/lib/common.sh"
cd "$_REPO_ROOT"

# The array is expanded with the ${arr[@]+...} form so an empty one is not an
# "unbound variable" error under `set -u` on macOS's stock bash 3.2.
PROFILE_FLAG=()
for arg in "$@"; do
  case "$arg" in
    --release) PROFILE_FLAG=(--release) ;;
    *) echo "unknown option: $arg" >&2; echo "usage: ci-check.sh [--release]" >&2; exit 2 ;;
  esac
done

verify_rust_toolchain

# `--locked`: a stale Cargo.lock fails here instead of being quietly rewritten.
echo "=== cargo check ==="
cargo check --locked --all-targets ${PROFILE_FLAG[@]+"${PROFILE_FLAG[@]}"}

# Safe on a headless machine: the suite is pure logic and none of it opens an
# audio or input device. Platform-specific tests sit behind file-level cfg
# gates, so each OS runs its own subset.
echo "=== cargo test ==="
cargo test --locked ${PROFILE_FLAG[@]+"${PROFILE_FLAG[@]}"}
