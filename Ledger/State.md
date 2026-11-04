# State

## Focus

**Программа реализации находок исследовательской дуги (семь раундов) через догфудинг.**
Пользователь поручил (/goal, 2026-11-04) реализовать все находки последовательно и для каждой
убедиться, что она работает. Находки оформлены карточками B052–B064 (B064 — идея, не выбрана),
дополнена B049 (AC2); выбраны в Plan **P055–P068** (порядок — в Plan.md). Текущая карточка —
P055 (B052, новый вид правила `global-state`), фаза VAC-B1: инертный `kind:` + детектор + тесты
+ D096.

## Workspace

Ветка `feature/global-state` (база master `32debc8`), наблюдаемый HEAD `f1c574a` (memory-VAC:
карточки B052–B064, дополнение B049, Plan P055–P068). В рабочем дереве — VAC-B1 P055, не
закоммичен. macOS: коммиты/cargo — через `just scratch` с восстановленным PATH
(`/Users/admin/.cargo/bin`), манифест — `Project/Cargo.toml`; иначе гейт падает на
`build: cargo not found`.

## Progress

- Memory-VAC закоммичен (`f1c574a`).
- P055 VAC-B1 подготовлен: `Kind::GlobalState` (`rules/kind.rs`), дескриптор `GLOBALSTATE`
  (`rules/descriptor.rs`), детектор `languages/global_state.rs` (диспетчеризация из `rust.rs` на
  `static_item` и `macro_invocation`), тесты `languages/tests/global_state.rs`, инвентари
  `architecture.yaml` обеих директорий, карточка D096 + строка индекса, P055 → active.
- Не сделано: проводка (`catalog::ALL`, `registry.rs`, `lint.yaml`), навык `own-the-state`,
  `assets.rs`, догфуд, мид-бранч пин.

## Verification

- `cargo fmt --check` и `cargo clippy --all-targets -- -D warnings` — чисто (рабочее дерево VAC-B1).
- `cargo test --lib global_state` — 2 passed: прямые статики (5 находок, обоснованный, `static mut`,
  приватные ячейки, `OnceLock<String>`, алиас через два уровня, строковый литерал, тестовый модуль)
  и тела `lazy_static!`/`thread_local!` (4 находки, обоснование внутри тела и над макросом,
  `macro_rules!`, тестовая функция).
- Гейт pre-commit для VAC-B1 ещё не запускался. Правило в `lint.yaml` не активно — догфуд не
  проводился.

## Blockers

Нет.

## Next action

Закоммитить VAC-B1 через `.agentrig/hooks/pre-commit` (Ledger: `Decisions.md`,
`Decisions/096.md`, `Plan.md`, `State.md`; Project: `languages/{global_state.rs,mod.rs,rust.rs,architecture.yaml}`,
`languages/tests/{global_state.rs,mod.rs,architecture.yaml}`, `rules/{kind.rs,descriptor.rs}`).
Затем мид-бранч пин D051 (stable.txt = голова VAC-B1 + `just bootstrap`, коммит пина), VAC-B2
(`catalog::ALL`, rust-список `registry.rs`, правило `global-state` в `lint.yaml` с
`# DECISION: D096`, навык `own-the-state` в `Project/shared/skills/references/` + symlink +
`assets.rs`, ссылки в строке D096), догфуд `just candidate lint` → 0, живая проба, ротация
P055/B052 в Archive, финальный пин, `just feature-merge`. Свободные ID: D097+, P069+, B065+.
