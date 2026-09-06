#!/usr/bin/env bash
set -euo pipefail
root="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$root"
[[ -z "$(git status --porcelain)" ]] || { echo 'Commit the release inputs before packaging' >&2; exit 2; }
[[ "$(uname -s)-$(uname -m)" == Linux-x86_64 ]] || { echo 'Release packaging supports Linux x86_64' >&2; exit 2; }
version_output="$(tooling/worker/build --version)"
read -r _ version <<< "$version_output"
[[ "$version" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]] || { echo 'Invalid release version' >&2; exit 2; }
output="$root/.cache/distribution"
mkdir -p "$output"
staging="$(mktemp -d "$output/.package-XXXXXX")"
trap 'rm -rf -- "$staging"' EXIT
name="agentrig-$version-linux-x86_64"
package="$staging/$name"
mkdir "$package"
target="${WORKER_TARGET_DIR:-$root/.cache/worker}"
cp "$target/release/agentrig" "$target/release/agentrig-lint" "$package/"
cp LICENSE README.md tooling/distribution/RELEASE.md "$package/"
cargo about generate --locked --fail --workspace --target x86_64-unknown-linux-gnu \
    --manifest-path tooling/worker/Cargo.toml --config tooling/distribution/about.toml \
    --format json --output-file "$package/THIRD_PARTY_LICENSES.json"
git rev-parse HEAD > "$package/REVISION"
tar -czf "$staging/$name.tar.gz" -C "$staging" "$name"
(cd "$staging" && sha256sum "$name.tar.gz" > "$name.tar.gz.sha256")
mv "$staging/$name.tar.gz" "$staging/$name.tar.gz.sha256" "$output/"
cp tooling/distribution/RELEASE.md "$output/RELEASE.md"
echo "$output/$name.tar.gz"
