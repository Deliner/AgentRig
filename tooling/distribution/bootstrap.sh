#!/usr/bin/env bash
# DECISION: D027
set -euo pipefail
root="$(cd "$(dirname "$0")/../.." && pwd)"
read -r revision < "$root/tooling/distribution/stable.txt"
[[ "$revision" =~ ^[0-9a-f]{40}$ ]] || { echo 'Invalid stable revision' >&2; exit 2; }
cache="$root/.cache/development"
mkdir -p "$cache"
exec 9>"$cache/bootstrap.lock"
flock 9
installed="$cache/$revision"
if [[ -d "$installed" ]]; then
    "$installed/.agentrig/bin/agentrig" --version
    exit 0
fi
staging="$(mktemp -d "$cache/.install-XXXXXX")"
trap 'rm -rf -- "$staging"' EXIT
git -C "$root" archive "$revision" tooling/worker > "$staging/source.tar"
tar -xf "$staging/source.tar" -C "$staging"
cargo +1.98.1 build --locked --release \
    --manifest-path "$staging/tooling/worker/Cargo.toml" \
    --target-dir "$cache/build"
"$cache/build/release/agentrig" init --root "$staging/environment" --language rust
mv "$staging/environment" "$installed"
"$installed/.agentrig/bin/agentrig" --version
