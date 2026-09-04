#!/usr/bin/env bash

# DECISION: D004
# DECISION: D005
# DECISION: D015
# DECISION: D016

set -euo pipefail
root="$(git rev-parse --show-toplevel)"
tree="$root"
if [[ "${1:-}" == "--staged" ]]; then
    tree="$(mktemp -d)"
    trap 'rm -rf "$tree"' EXIT
    git checkout-index --all --prefix="$tree/"
fi
project="$tree/tooling"
export UV_PROJECT_ENVIRONMENT="$root/.cache/venv"
if [[ "$tree" == "$root" ]]; then
    mapfile -d '' relative_python_files < <(git ls-files --cached --others --exclude-standard -z -- '*.py')
    python_files=()
    for path in "${relative_python_files[@]}"; do
        python_files+=("$tree/$path")
    done
else
    mapfile -d '' python_files < <(find "$tree" -type f -name '*.py' -print0)
fi
# Build from the tree being verified, including the staged Rust sources.
export WORKER_TARGET_DIR="$root/.cache/worker"
"$tree/tooling/worker/run" lint --root "$tree"
export WORKER_BINARY="$WORKER_TARGET_DIR/release/discipline-worker"
gate() {
    "$WORKER_BINARY" gate --root "$tree" "$@"
}
gate rustfmt -- cargo +1.98.1 fmt --manifest-path "$project/worker/Cargo.toml" --check
gate clippy -- cargo +1.98.1 clippy --manifest-path "$project/worker/Cargo.toml" --target-dir "$WORKER_TARGET_DIR" --locked -- -D warnings
gate repo-policy -- uv run --locked --project "$project" python "$project/check_repo.py" --root "$tree" --history-root "$root"
gate command-policy -- uv run --locked --project "$project" python "$project/check_commands.py" "$tree"
gate ruff-format -- uv run --locked --project "$project" ruff format --check --config "$project/pyproject.toml" "${python_files[@]}"
gate ruff -- uv run --locked --project "$project" ruff check --config "$project/pyproject.toml" "${python_files[@]}"
gate mypy -- uv run --locked --project "$project" mypy --config-file "$project/pyproject.toml" "${python_files[@]}"
gate pytest -- uv run --locked --project "$project" pytest -c "$project/pyproject.toml" "$tree"
gate typos -- uv run --locked --project "$project" typos --config "$project/pyproject.toml" "$tree"
gate vulture -- uv run --locked --project "$project" vulture --min-confidence 100 "${python_files[@]}"
