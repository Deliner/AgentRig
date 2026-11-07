# State

## Focus

Программа **P071 → P082** (/goal пользователя 2026-11-07): все открытые карточки Backlog выполняются автономно по
одной за ветку. P071–P080 завершены и слиты в master, последняя — `a0a6865`. **P081** (B069 grammar-fork)
доставлена на ветке двумя VAC; этот снимок входит в коммит пина, осталось слияние. Дальше P082 (B064), затем новые
карточки Backlog из `agentrig-research/clippy-triage.md` и `agentrig-research/agent-errors.md` по уже данному
согласию пользователя.

## Workspace

Branch: `feature/grammar-fork`
Revision: `a12958df7d7677599decc18d197cef872e96df9a`

База — master `a0a6865`. VAC 1 `38ddf9c`, VAC 2 `a12958d`; stable.txt указывает на `a12958d`, установлен через
`just bootstrap`. Ветки `feature/embedded-text` и прежние, а также `feature/init-probe`, сохранены. macOS: в PATH
`just write`/`just run`/`just check`/`just feature-merge` нет cargo — коммиты и вызовы с cargo через
`just scratch -- 'export PATH=/Users/admin/.cargo/bin:$PATH && cd /Users/admin/projects/AgentRig && …'`;
`cargo fmt --manifest-path Project/Cargo.toml`; поставка — `just candidate <команда> --root <каталог>`.
cargo-about 0.9.2 установлен в scratchpad сессии `p081/cargo-about/bin`.

## Progress

- VAC 1 `38ddf9c`: копия крейта 0.24.2 без изменений, зависимость-путь, исключения линта и typos, решение
  [D112](Decisions/112.md).
- VAC 2 `a12958d`: правка `grammar.js` с маркерами D112 и `src` от CLI 0.26.13, тесты в
  `languages/tests/unsafe_site.rs` и `panic_surface.rs`, абзац `Project/README.md`. Delivery в
  [Archive/Plan/081.md](Archive/Plan/081.md); P081 и B069 перенесены в архив, ссылки D112 обновлены.

## Verification

- 2026-11-07, гейт pre-commit VAC 1 `38ddf9c`: все проверки PASS, rust-test 149 passed, test 307 passed.
- Гейт pre-commit VAC 2 `a12958d`: все проверки PASS, rust-test 139 passed, test 221 passed; memory предупреждает,
  что маркер в `grammar.js` не проверяется, это поведение проверки для `.js`.
- До коммита VAC 2: оба новых теста на копии с грамматикой 0.24.2 падают на ошибке разбора;
  `just candidate lint --root .` — код 0. После: `just release` из чистого дерева — пакет собран, tree-sitter-rust
  0.24.2 MIT в `THIRD_PARTY_LICENSES.json`; `just bootstrap` на `a12958d` — `agentrig 0.3.0`.
- Корпус r8, AC2: `syntax` 838 с 0.24.2 и 125 с форком, починено 713; у 17 789 разобранных файлов 0 отличий вывода.

## Blockers

Нет.

## Next action

`just feature-merge` ветки `feature/grammar-fork`; затем P082 (B064 ratchet-baseline) на новой ветке. Свободные
ID: D123+, I022+, P083+, B075+ (B066 и D111 зарезервированы).
