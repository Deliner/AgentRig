#!/usr/bin/env bash
# DECISION: D027
# DECISION: D041
# DECISION: D173
# DECISION: D197
# DECISION: D218
set -euo pipefail
root="$(cd "$(dirname "$0")/../.." && pwd)"
pin="$root/tooling/distribution/stable.txt"
(($# <= 1)) || { echo 'Usage: just bootstrap [REVISION]' >&2; exit 2; }
if (($#)); then
    revision="$(git -C "$root" rev-parse --verify --end-of-options "$1^{commit}")"
else
    read -r revision < "$pin"
fi
[[ "$revision" =~ ^[0-9a-f]{40}$ ]] || { echo 'Invalid stable revision' >&2; exit 2; }
hooks="$(git -C "$root" config --get core.hooksPath || true)"
[[ -z "$hooks" || "$hooks" == .agentrig/hooks ]] || { echo "Existing core.hooksPath=$hooks; reconcile it before bootstrap" >&2; exit 2; }
cache="$root/.cache/development"
mkdir -p "$cache"
exec 9>"$cache/bootstrap.lock"
flock 9
installed="$cache/$revision"
if [[ ! -d "$installed" ]]; then
    staging="$(mktemp -d "$cache/.install-XXXXXX")"
    trap 'rm -rf -- "$staging"' EXIT
    git -C "$root" archive "$revision" Project rust-toolchain.toml > "$staging/source.tar"
    tar -xf "$staging/source.tar" -C "$staging"
    (cd "$staging" && cargo build --locked --release --manifest-path Project/Cargo.toml --target-dir "$cache/build")
    "$cache/build/release/agentrig" init --root "$staging/environment" --language rust
    mv "$staging/environment" "$installed"
fi
"$installed/.agentrig/bin/agentrig" --version
git -C "$root" config core.hooksPath .agentrig/hooks
