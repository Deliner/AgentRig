# State

## Focus

**Программа реализации находок исследовательской дуги (семь раундов) через догфудинг.**
Пользователь поручил (/goal, 2026-11-04) реализовать все находки последовательно и для каждой
убедиться, что она работает. Карточки B052–B064 (B064 — идея, не выбрана), дополнение B049 (AC2);
выбраны в Plan **P055–P068** (порядок — в Plan.md). P055 (B052, `global-state`) выполнен, проверен
и перенесён в Archive; ветка завершается `just feature-merge`. Следующая карточка — P056 (B049 AC2,
`silent-error` для Rust), новая ветка.

## Workspace

Ветка `feature/global-state` (база master `32debc8`), наблюдаемый HEAD `5202340` (финальный пин
D051: stable.txt → `db0a868`, bootstrap выполнен, гейт пина PASS). Этот State коммитится отдельным
memory-коммитом перед слиянием. macOS: коммиты/cargo — через `just scratch` с восстановленным PATH
(`/Users/admin/.cargo/bin`), манифест — `Project/Cargo.toml`; `just candidate …` напрямую падает на
`cargo: command not found`, рабочая форма — `tooling/worker/run run candidate -- lint` внутри
`just scratch`. Навык для нового правила создаётся раньше ссылки в `lint.yaml`
(`.agents/skills` → `Project/shared/skills`), иначе конфигурация уходит в режим починки.

## Progress

- `f1c574a` memory-VAC (B052–B064, B049 AC2, P055–P068); `c065b42` VAC-B1 (`Kind::GlobalState`,
  детектор `languages/global_state.rs`, тесты, D096); `a8de4ec` пин → `c065b42`; `db0a868` VAC-B2
  (`catalog::ALL`, `registry.rs`, правило `global-state` в `.agentrig/lint.yaml`, навык
  `own-the-state` + `assets.rs`, ссылки D096, ротация P055 → `Archive/Plan/055.md`, B052 →
  `Archive/Backlog/052.md`); `5202340` финальный пин → `db0a868`.
- Не сделано: коммит этого State, `just feature-merge`; P056–P068.

## Verification

- Гейт VAC-B2 `db0a868`: все проверки PASS, rust-test 112 passed, python 308 passed (в том числе
  `project_setup/tests/test_lint_defaults.py` — I020), lint PASS с живым правилом, memory PASS.
  Гейт пина `5202340`: PASS на установленной ревизии `db0a868`.
- Догфуд `tooling/worker/run run candidate -- lint` — 0 находок; живая проба `static PROBE: Mutex<u8>`
  → `ERROR [global-state] global_state.rs:11 (PROBE)` с навыком `own-the-state`, после удаления
  снова чисто; `just lint-rule global-state` описывает вид; `just lint-config-check` — valid;
  `cargo test --lib global_state` — 2 passed (AC1–AC3 и не-находки B052).
- `just feature-merge` (тесты всей ветки) ещё не запускался.

## Blockers

Нет.

## Next action

Закоммитить State, затем `just feature-merge` (ветку сохранить, не пушить). Далее
`just feature-start silent-error-rust` для P056 (B049 AC2): прочитать python-детектор `silent-error`
и `languages/rust.rs`, выбрать механизм (расширение `silent-error` на `.rs` или новый `kind:`),
двухфазная доставка с мид-бранч пином, догфуд (единственный наш сайт —
`change_delivery/git/mod.rs:146`, чинится fail-closed по B062 AC1 в P066 либо обосновывается).
Свободные ID: D097+, P069+, B065+.
