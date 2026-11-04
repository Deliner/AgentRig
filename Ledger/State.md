# State

## Focus

**Программа реализации находок исследовательской дуги (семь раундов) через догфудинг.**
Пользователь поручил (/goal, 2026-11-04) реализовать все находки последовательно и для каждой
убедиться, что она работает. Карточки B052–B064 (B064 — идея, не выбрана), дополнение B049 (AC2);
выбраны в Plan **P055–P068** (порядок — в Plan.md). P055–P064 слиты (последний merge `4cf520c`).
**P065** (B060, C-профиль clippy dev-гейта и ограждение `unsafe`): VAC-1…VAC-4 (`clippy::exit`,
`should_panic_without_expect`, `missing_fields_in_debug`, `rest_pat_in_fully_bound_structs`)
закоммичены; VAC-5 (`-D clippy::self_named_module_files` + перенос `rules.rs` → `rules/mod.rs`)
завершается этим коммитом; далее VAC-6 (`#![deny(unsafe_code)]`, D107, ротация P065/B060),
финальный пин D051 и `just feature-merge`.

## Workspace

Ветка `feature/clippy-c-profile` (база master `4cf520c`), наблюдаемый HEAD `9fb446f` (VAC-4);
stable.txt → `5a373d2`. macOS: коммиты/cargo — через `just scratch` с восстановленным
PATH (`/Users/admin/.cargo/bin`), манифест — `Project/Cargo.toml`; профиль clippy гейта —
`tooling/worker/run run clippy` (без `--all-targets`: код под `#[cfg(test)]` профилем не виден);
догфуд — `tooling/worker/run run candidate -- lint`; pytest одного файла —
`tooling/worker/run run test -- <файл>`.

## Progress

- Базовый прогон пяти линтов на `4cf520c` (с `--all-targets` и без): единственная находка —
  `self_named_module_files` на `Project/structural_lint/rules.rs` («move to rules/mod.rs»); по
  остальным четырём — 0. Ожидание карточки «0 ремонта» по AC5 не подтвердилось: VAC-5 переносит
  `rules.rs` → `rules/mod.rs` (`git mv`) и правит `structural_lint/architecture.yaml`.
- `unsafe`-сайты продукта — только `command_runner/jobs/{identity/macos.rs, identity/linux.rs,
  streams.rs, process.rs, group.rs}`; в `.agentrig/commands.yaml` маркеров DECISION нет
  (YAML-владелец D060 без маркера; для D107 аналогично).
- VAC-1 (`27f2dd2`): `-D clippy::exit` в профиле `clippy` `.agentrig/commands.yaml`; строка P065 в
  Plan.md — active.
- VAC-2 (`3b625a4`): `-D clippy::should_panic_without_expect`. Ограничение: профиль гейта без
  `--all-targets` (D060) не видит код под `#[cfg(test)]`, поэтому линт держит только атрибуты в
  компилируемом не-тестовом коде; фиксируется в D107 и Delivery.
- VAC-3 (`9e60186`): `-D clippy::missing_fields_in_debug`.
- VAC-4 (`9fb446f`): `-D clippy::rest_pat_in_fully_bound_structs`.
- VAC-5 (этот коммит): `-D clippy::self_named_module_files`; `git mv structural_lint/rules.rs
  structural_lint/rules/mod.rs`. Перенос показал догфуду цикл `catalog → languages → rules → catalog`
  (реэкспорт каталога из файла, который теперь лежит в `rules/`): реэкспорт `catalog::{ALL, catalog,
  describe, example, …}` из `rules/mod.rs` удалён, потребители (`structural_lint/{mod,cli}.rs`,
  `project_setup/lint.rs`) обращаются к `catalog::` напрямую (`config` и `selection` уже делали так);
  входящие рёбра на `rules::Kind` теперь относятся к `rules/`, а не к родителю — бюджет
  `rules/architecture.yaml` поднят до фактического радиуса 7 (D079). `structural_lint/architecture.yaml`
  без `rules.rs`, `rules/architecture.yaml` описывает `mod.rs`.
- Не сделано: VAC-6
  (`#![deny(unsafe_code)]` в `Project/lib.rs`, `#[allow(unsafe_code)]` + `// DECISION: D107` над
  `pub mod jobs;` в `command_runner/mod.rs`, карточка D107 и строка индекса, ротация P065/B060),
  финальный пин D051, `just feature-merge` (ветку сохранить, не пушить).

## Verification

- VAC-1: проба `pub fn probe() { std::process::exit(1) }` в `Project/shared/fs/mod.rs` →
  `tooling/worker/run run clippy` падает (exit 101, «usage of `process::exit` … `-D clippy::exit`»),
  после отката — exit 0; гейт `27f2dd2` прошёл (128 rust-test, 112 pytest).
- VAC-2: проба `#[should_panic] pub fn probe() {}` там же → exit 101 («#[should_panic] attribute
  without a reason … `-D clippy::should-panic-without-expect`»), после отката — exit 0; гейт `3b625a4`
  прошёл (128 rust-test, 112 pytest).
- VAC-3: проба — `pub struct Probe { first, second }` с ручным `Debug`, печатающим только `first`
  через `debug_struct(..).finish()`, там же → exit 101 («manual `Debug` impl does not include all
  fields … `-D clippy::missing-fields-in-debug`»), после отката — exit 0; гейт `9e60186` прошёл
  (128 rust-test, 112 pytest).
- VAC-4: проба — `match value { Probe { first, second, .. } => … }` там же → exit 101 («unnecessary
  use of `..` pattern in struct binding. All fields were already bound …
  `-D clippy::rest-pat-in-fully-bound-structs`»; рядом `match-single-binding` из `-D warnings`), после
  отката — exit 0; гейт `9fb446f` прошёл (128 rust-test, 112 pytest).
- 2026-11-05, рабочее дерево VAC-5: профиль с линтом на нетронутом дереве → exit 101 («`mod.rs` files
  are required, found `structural_lint/rules.rs` … `-D clippy::self-named-module-files`») — реальная
  находка вместо пробы; после `git mv` — exit 0, `--all-targets` тоже чисто. Догфуд после `git mv`:
  2 находки (цикл и бюджет 7 > 6); после удаления реэкспорта — только бюджет; после `dependents: 7` —
  см. гейт этого VAC. `cargo test --lib`: 128 ok. Коммит — следующий шаг.

## Blockers

Нет.

## Next action

Закоммитить VAC-5 через гейт («P065 VAC-5: deny clippy::self_named_module_files and move rules.rs to
rules/mod.rs (B060 AC5)»); затем VAC-6: `#![deny(unsafe_code)]` в `Project/lib.rs` (строка
`// DECISION: D107` над ним), `// DECISION: D107` + `#[allow(unsafe_code)]` над `pub mod jobs;` в
`command_runner/mod.rs`, проба `unsafe` вне поддерева (ошибка компиляции), карточка D107 (расширяет
D060: набор, невыбранные линты, deny-по-умолчанию `derived_hash_with_manual_eq` /
`derive_ord_xor_partial_ord`, ограничение `#[cfg(test)]`), строка индекса, ротация P065/B060, затем
финальный пин D051 и `just feature-merge`. Свободные ID: D107+, I022+, P069+, B065+.
