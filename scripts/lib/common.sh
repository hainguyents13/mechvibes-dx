# Shared helpers for the bash build/CI scripts. Source it, do not execute it:
#
#   source "$(dirname "${BASH_SOURCE[0]}")/lib/common.sh"
#
# build-windows-installer.ps1 carries its own copy of the toolchain check,
# because PowerShell cannot source this file.

_COMMON_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
_REPO_ROOT="$(cd "${_COMMON_DIR}/../.." && pwd)"

# True if version $1 is >= version $2. Compares up to three numeric,
# dot-separated parts, treating a missing part as 0 and ignoring any suffix, so
# "1.88" equals "1.88.0" and "1.89.0-nightly" counts as 1.89.0. Plain bash, no
# `sort -V`, so it also runs under macOS's stock bash 3.2.
_version_ge() {
  local a="$1" b="$2" i x y
  local -a av bv
  local IFS=.
  read -r -a av <<<"$a"
  read -r -a bv <<<"$b"
  for i in 0 1 2; do
    x="${av[$i]:-0}"
    y="${bv[$i]:-0}"
    x="${x%%[!0-9]*}"
    y="${y%%[!0-9]*}"
    x="${x:-0}"
    y="${y:-0}"
    [ "$x" -gt "$y" ] && return 0
    [ "$x" -lt "$y" ] && return 1
  done
  return 0
}

# Fails unless the Rust in use is at least the `rust-version` declared in
# Cargo.toml - the crate's minimum supported Rust, which is the floor.
#
# It is a floor, not an exact match: any newer compiler is fine. What this
# catches is a compiler too old to build the crate, which otherwise surfaces late
# as an unrelated-looking error. Typical causes are an exported RUSTUP_TOOLCHAIN,
# an outdated default toolchain, or a Rust that does not come from rustup at all.
verify_rust_toolchain() {
  local cargo_toml="${_REPO_ROOT}/Cargo.toml"
  local floor got

  floor=$(sed -n 's/^[[:space:]]*rust-version[[:space:]]*=[[:space:]]*"\([^"]*\)".*/\1/p' "$cargo_toml" | head -n1)
  if [ -z "$floor" ]; then
    echo "::error::Cargo.toml has no 'rust-version', so there is no minimum Rust to check against"
    return 1
  fi

  got=$(rustc --version 2>/dev/null | awk '{print $2}' || true)

  if [ -z "$got" ] || ! _version_ge "$got" "$floor"; then
    echo "::error::Rust ${got:-<none>} is active but Cargo.toml requires rust-version ${floor} or newer."
    echo "  Update it ('rustup update stable'), or check for a RUSTUP_TOOLCHAIN override."
    return 1
  fi

  echo "Rust toolchain OK: ${got} (at least rust-version ${floor} from Cargo.toml)"
}

# The Windows auto-updater picks the first release asset whose name contains
# "x64" AND ends in ".exe". No Linux or macOS asset may ever match, or a rename
# would break Windows updates silently. Asserted rather than trusted; the same
# pairing is covered by a unit test in auto_updater.rs.
#
# Usage: assert_no_updater_collision <dir>
assert_no_updater_collision() {
  local dir="$1" f name status=0
  for f in "$dir"/*; do
    [ -e "$f" ] || continue
    name=$(basename "$f" | tr '[:upper:]' '[:lower:]')
    case "$name" in
      *x64*.exe)
        echo "::error::asset '${name}' matches the Windows auto-updater filter (contains 'x64' and ends in '.exe')"
        status=1
        ;;
    esac
  done
  return $status
}
