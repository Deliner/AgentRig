# State

## Focus

**Программа реализации находок исследовательской дуги (семь раундов) через догфудинг.**
Пользователь поручил (/goal, 2026-11-04) реализовать все находки последовательно и для каждой
убедиться, что она работает. Карточки B052–B064 (B064 — идея, не выбрана), дополнение B049 (AC2);
выбраны в Plan **P055–P068** (порядок — в Plan.md). P055 (`33477fd`) и P056 (`8c3142c`) выполнены и
слиты. Текущая карточка — **P057** (B053 AC1–AC4, `allow-justification`: `expect`, `cfg_attr`,
`[lints]` Cargo.toml): расширение существующего вида, решение [D098](Decisions/098.md).

## Workspace

Ветка `feature/allow-justification-ext` (база master `8c3142c`), наблюдаемый HEAD `8c3142c`; VAC-B1 в
рабочем дереве и коммитится этим VAC. macOS: коммиты/cargo — через `just scratch` с восстановленным
PATH (`/Users/admin/.cargo/bin`), манифест — `Project/Cargo.toml`; рабочая форма догфуда —
`tooling/worker/run run candidate -- lint` внутри `just scratch`. Расширение `.toml` для
`allow-justification` в `lint.yaml` принимается только после пина `registry.rs` (иначе конфигурация
уходит в режим починки) — двухфазная доставка ([D051](Decisions/051.md)).

## Progress

- VAC-B1 (этот коммит): диспетчеризация атрибутов вынесена из `rust.rs` в `languages/suppression.rs`
  (`allow`/`expect`/`ignore`, голова `cfg_attr` — прямой аргумент после запятой; `ignore` → `disabled-test`),
  TOML-обработчик `languages/manifest.rs` (`DeTable` крейта `toml`, таблицы `lints`/`workspace.lints`,
  `"allow"`/`{ level = "allow" }`, ошибка синтаксиса → диагностика `syntax`), `Inspect::{Tree, Document}`
  в `registry.rs` + обработчик `toml`, `justified_line` в `languages/mod.rs`, дескрипторы
  `allow-justification`/`disabled-test` (AC4: `reason =` назван в сообщении), тесты
  `tests/markers.rs` (AC1–AC2) и `tests/manifest.rs` (AC3 + ошибка синтаксиса), D098 (+ ссылки D074/D080
  на `suppression.rs`), P057 → active.
- Не сделано: мид-бранч пин → после этого коммита; VAC-B2 — `.toml` в правиле `allow-justification`
  `.agentrig/lint.yaml`, навык `justify-allow` с тремя формами, догфуд, живая проба каждой формы
  (`expect`, `cfg_attr`, `[lints]`), ротация P057/B053; финальный пин; `just feature-merge`; P058–P068.

## Verification

- Локально 2026-11-04 на рабочем дереве VAC-B1: `cargo fmt --check` OK (после правки раскладки),
  `cargo clippy --all-targets -D warnings` OK, `cargo test --lib` — 118 passed (в том числе
  `rust_flags_expect_and_conditional_suppressions_like_allow`: allow-находки строки 1, 6, 8, 10, 18,
  disabled-test 20, 23; `toml_flags_allow_levels_under_lint_tables_without_a_reference_above`:
  `1:private_intra_doc_links`, `5:unused`, `8:missing_docs`, `12:pedantic`, `15:unused_imports`,
  обоснованный `dead_code` и `deny`/`warn`/`[dependencies] allow` не-находки;
  `toml_reports_the_line_of_a_syntax_error` — строка 3).
- Догфуд кандидата с новым Rust-детектором (`.rs`): `tooling/worker/run run candidate -- lint` — 0 находок
  (exit 0). Гейт коммита VAC-B1 ещё не запускался; `.toml` в правиле и живая проба — после пина (VAC-B2).

## Blockers

Нет.

## Next action

Закоммитить VAC-B1 через гейт, затем пин D051 (`git rev-parse HEAD > tooling/distribution/stable.txt`
+ bootstrap, отдельный коммит). Далее VAC-B2: `.toml` в `extensions` правила `allow-justification`,
навык `justify-allow` (формы `expect`/`cfg_attr`/`[lints]`, `reason =` не ссылка), `just lint-config-check`,
догфуд (ожидается 0 находок: форм в нашем коде нет, `[lints]` в `Project/Cargo.toml` нет), живая проба
трёх форм, `just lint-rule allow-justification` (Rust и TOML), ротация P057 → Archive, финальный пин,
`just feature-merge` (ветку сохранить, не пушить). Свободные ID: D099+, I022+, P069+, B065+.
