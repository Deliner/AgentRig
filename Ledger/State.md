# State

## Focus

Реализация набора улучшений качества (кластеры 1–8, [B026](Backlog/026.md)–
[B033](Backlog/033.md)) через догфудинг, по одной карточке. Текущая — [P037](Plan/037.md)
([B026](Backlog/026.md)): линты безопасности. 1.1 обоснование `unsafe` ссылкой, 1.2
Bandit-группа ruff `S` (влито), 1.3 скан хардкод-секретов.

## Workspace

Ветка: feature/security-lint (от master `f4f853d`). Коммиты: `474ff9b` VAC-A (ruff `S`, D064);
VAC-B1 (этот коммит) — структурное правило `unsafe-justification`, только машинерия, не
активно. `tooling/distribution/stable.txt` = `06ef95e` (ещё не перепиннен под VAC-B1).

Машина — macOS. Сборка/clippy/fmt/тесты/коммит/мерж/бустрап — через `just scratch -- '...'` с
PATH (`cargo` в `~/.cargo/bin`; для гейта с Python-тестами venv `tooling/.venv/bin` первым).

## Progress

VAC-A (1.2) влит (`474ff9b`). VAC-B1 (двухкоммитная схема, enabling): добавлен
`Kind::UnsafeJustification` + дескриптор + запись в Rust-handler + измерение (unsafe-блок или
`unsafe fn` без маркера `DECISION:`/`INVARIANT:` строкой прямо выше) + юнит-тест. НЕ в
`catalog::ALL`, НЕ в `.agentrig/lint.yaml`, без маркеров, скилл ещё не создан — установленный
бинарь должен узнать Kind через пин, прежде чем lint.yaml сошлётся. `inspect` в `rust.rs` разбит:
вынесен `declaration()` (по образцу python `measure()`), чтобы функция осталась под function-size.

## Verification

`cargo fmt`/`clippy` чисто; `cargo test` 84 passed (+1 новый `rust_flags_unsafe_...`); `just lint`
чисто (правило не активно — ~11 `unsafe` в `command_runner/jobs` пока не флагаются).

## Blockers

Нет.

## Next action

1) Запинить head VAC-B1 в `tooling/distribution/stable.txt` + `just bootstrap` (один scratch;
stable.txt НЕ коммитить — промежуточный пин). 2) VAC-B2 (go-live): создать скилл `justify-unsafe`
(SKILL.md + architecture.yaml), внести в `Project/shared/skills/architecture.yaml`, добавить в
`project_setup/assets.rs` SKILLS и в init-поставляемый lint.yaml (I020: delivered==live по `skills`
и `rule-kinds`), добавить `unsafe-justification` в `.agentrig/lint.yaml` (Level/error, Rust `.rs`),
добавить Kind в `catalog::ALL`, записать D065 (правило) + маркеры на
descriptor.rs/rust.rs/registry.rs/kind.rs/tests, записать D066 (обоснование FFI) + маркер
`// DECISION: D066` строкой выше каждого из ~11 `unsafe` (identity/macos.rs, linux.rs, process.rs,
group.rs), refresh State; коммит VAC-B2. 3) VAC-C (1.3): оценить статическую осуществимость; правило
ИЛИ norm-or-choice. 4) Финал P037: финальный пин (D051) + архив + `just feature-merge`. Не пушить,
ветку сохранить.
