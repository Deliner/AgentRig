#!/usr/bin/env bash

# DECISION: D004
# DECISION: D005

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
uv run --locked --project "$project" python "$project/check_repo.py" --root "$tree" --history-root "$root"
uv run --locked --project "$project" python "$project/check_commands.py" "$tree"
uv run --locked --project "$project" ruff format --check --config "$project/pyproject.toml" "${python_files[@]}"
uv run --locked --project "$project" ruff check --config "$project/pyproject.toml" "${python_files[@]}"
uv run --locked --project "$project" mypy --config-file "$project/pyproject.toml" "${python_files[@]}"
uv run --locked --project "$project" pytest -c "$project/pyproject.toml" "$tree"
uv run --locked --project "$project" typos --config "$project/pyproject.toml" "$tree"
uv run --locked --project "$project" vulture --min-confidence 100 "${python_files[@]}"
