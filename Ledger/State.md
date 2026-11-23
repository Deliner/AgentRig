# State

## Focus

P198 (B205): риг без режима ratchet. VAC 3 пинует ревизию `0c74f22`, в которой ratchet удалён (D235).

## Workspace

Branch: feature/lint-ratchet-removal
Revision: 0c74f22004e14b4058838a5778a677a278ed8f9a

## Progress

VAC 1 `0ef0631`: P198 выбран. VAC 2 `0c74f22`: удалены команда, рецепт и пропуск хуком `lint-baseline`, модуль
baseline и `Repository::renames`; lint, гейт и `agentrig-lint` не читают baseline, он вне защиты; `init` и мастер
называют число ошибок; D235. `just propose-pin HEAD` подготовил пин `0c74f22`.

## Verification

Гейт коммита `0c74f22`: build, candidate-lint, lint, memory, rustfmt, clippy, 635 Rust-тестов, ruff, mypy, typos,
vulture — PASS.

Пользователь выполнил `just approve`: `0c74f22` установлен, пин записан в `tooling/distribution/stable.txt`.

## Blockers

Нет.

## Next action

Закоммитить пин с State, затем архивировать P198 и B205 и выполнить `just feature-merge`.
