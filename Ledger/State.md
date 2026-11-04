# State

## Focus

**Программа реализации находок исследовательской дуги (семь раундов) через догфудинг.**
Пользователь поручил (/goal, 2026-11-04) реализовать все находки последовательно и для каждой
убедиться, что она работает. Карточки B052–B064 (B064 — идея, не выбрана), дополнение B049 (AC2);
выбраны в Plan **P055–P068** (порядок — в Plan.md). P055–P064 слиты (последний merge `4cf520c`).
**P065** (B060, C-профиль clippy dev-гейта и ограждение `unsafe`): VAC-1 (`-D clippy::exit`, строка
Plan → active) завершается этим коммитом; далее VAC-2…VAC-5 (по одному линту), VAC-6
(`#![deny(unsafe_code)]`, D107, ротация P065/B060), финальный пин D051 и `just feature-merge`.

## Workspace

Ветка `feature/clippy-c-profile` (база master `4cf520c`), наблюдаемый HEAD `4cf520c` (коммитов на
ветке ещё нет); stable.txt → `5a373d2`. macOS: коммиты/cargo — через `just scratch` с восстановленным
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
- VAC-1 (этот коммит): `-D clippy::exit` в профиле `clippy` `.agentrig/commands.yaml`; строка P065 в
  Plan.md — active; State.
- Не сделано: VAC-2 `should_panic_without_expect`, VAC-3 `missing_fields_in_debug`, VAC-4
  `rest_pat_in_fully_bound_structs`, VAC-5 `self_named_module_files` (+ перенос `rules.rs`), VAC-6
  (`#![deny(unsafe_code)]` в `Project/lib.rs`, `#[allow(unsafe_code)]` + `// DECISION: D107` над
  `pub mod jobs;` в `command_runner/mod.rs`, карточка D107 и строка индекса, ротация P065/B060),
  финальный пин D051, `just feature-merge` (ветку сохранить, не пушить).

## Verification

- 2026-11-05, рабочее дерево VAC-1: проба `pub fn probe() { std::process::exit(1) }` в
  `Project/shared/fs/mod.rs` → `tooling/worker/run run clippy` падает (exit 101: «usage of
  `process::exit` … requested on the command line with `-D clippy::exit`»); после отката — exit 0.
- Гейт этого VAC ещё не запускался (коммит — следующий шаг).

## Blockers

Нет.

## Next action

Закоммитить VAC-1 через гейт («P065 VAC-1: deny clippy::exit in the dev-gate profile (B060 AC1)»);
затем VAC-2: `-D clippy::should_panic_without_expect` с пробой `#[should_panic] pub fn` вне
`#[cfg(test)]` (профиль без `--all-targets`); если проба не ловится — честно зафиксировать
ограничение в State и Delivery. Свободные ID: D107+, I022+, P069+, B065+.
