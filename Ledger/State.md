# State

## Focus

Программа **P071 → P082** (/goal пользователя 2026-11-07): все открытые карточки Backlog выполняются автономно по
одной за ветку. P071–P080 завершены и слиты в master, последняя — `a0a6865`. **P081** (B069 grammar-fork) активна:
VAC 1 кладёт в `Project/shared/syntax/tree-sitter-rust` неизменённую копию крейта 0.24.2 и решение
[D112](Decisions/112.md). Дальше VAC 2 с правкой `grammar.js`, затем P082 (B064), затем новые карточки Backlog из
`agentrig-research/clippy-triage.md` и `agentrig-research/agent-errors.md` по уже данному согласию пользователя.

## Workspace

Branch: `feature/grammar-fork`
Revision: `a0a6865d69002df247e0e3fd69c1af4e4a6fcf71`

База — master `a0a6865`. Ветки `feature/embedded-text` и прежние, а также `feature/init-probe`, сохранены. macOS: в
PATH `just write`/`just run`/`just check`/`just feature-merge` нет cargo — коммиты и вызовы с cargo через
`just scratch -- 'export PATH=/Users/admin/.cargo/bin:$PATH && cd /Users/admin/projects/AgentRig && …'`;
`cargo fmt --manifest-path Project/Cargo.toml`; поставка — `just candidate <команда> --root <каталог>`.
Рабочие файлы форка в scratchpad сессии `p081/`: `regen-0.26.13/` — правленная и перегенерированная грамматика,
`ac2.sh` и `ac2_compare.py` — сравнение на корпусе, генератор CLI 0.26.13 в `cli-0.26.13/`.

## Progress

- VAC 1, в этом коммите: копия крейта 0.24.2 без изменений, зависимость-путь в `Project/Cargo.toml`, исключения
  линта и typos, маркеры D112, решение D112 и строка индекса.
- Проверено на корпусе r8 до VAC 2 с правленной грамматикой: из 18 627 файлов `syntax` у 838 с 0.24.2 и у 125 с
  форком, починено 713, все из списка потерянных; у разобранных файлов вывод линта не изменился ни в одном пути.

## Verification

- 2026-11-07, `cargo update --offline -p tree-sitter-rust` убрал источник и контрольную сумму крейта из
  `Cargo.lock`; проба typos подтвердила исключение каталога форка.
- AC2 на корпусе: сборки `target-base` из рабочего дерева и `target-fork` с правленной грамматикой, 18 репозиториев,
  0 отличий вывода у разобранных файлов.

## Blockers

Нет.

## Next action

Коммит VAC 1; затем VAC 2: маркеры D112 у правил PR #256 в `regen-0.26.13/grammar.js`, перегенерация, копирование
`grammar.js` и `src` в каталог форка, тесты в `languages/tests/unsafe_site.rs` и `panic_surface.rs`, абзац README,
проверка AC3. Свободные ID: D123+, I022+, P083+, B075+ (B066 и D111 зарезервированы).
