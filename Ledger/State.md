# State

## Focus

Программа **P083 → P086** (пользователь 2026-11-07: «бери все и делай») — карточки B075–B078 по одной за ветку,
автономно. P083 (B077) и P084 (B078) выполнены и слиты, master `36f3dbe`. Сейчас **P085** (B076 exit-code), VAC 1 —
AC1: новая форма `silent-error` для `Ok(<status>.success())` в функции `Result<bool>` без чтения `.code()`, навыки
`handle-error` и `handle-discarded`, решение D126.

## Workspace

Branch: `feature/exit-code`
Revision: `36f3dbeb413b1327e52e73ff50b633414d59279a`

База — master `36f3dbe`. Ветки `feature/disallowed-methods`, `feature/impact-coverage` и прежние, а также
`feature/init-probe`, сохранены. Установленный рантайм — `757bc30`. macOS: в PATH `just write`/`just run`/
`just check`/`just feature-merge` нет cargo — коммиты и вызовы с cargo через
`just scratch -- 'export PATH=/Users/admin/.cargo/bin:$PATH && cd /Users/admin/projects/AgentRig && …'`.

## Progress

- P085 VAC 1: `exit_verdict` в `Project/structural_lint/languages/rust/silent_error.rs`, вызов из
  `languages/rust/mod.rs`; обход `descendants` перенесён из `ord_equality.rs` в `languages/measure/mod.rs`; три
  теста в `languages/tests/silent_error.rs`; текст правила в `rules/descriptor.rs`; оба навыка; D126 с индексом.
  Конфигурация не меняется, поэтому нужен только финальный пин D051.
- Осталось для P085 — VAC 2, AC2: `std::process::Command` в `disallowed` вне `Project/shared/vcs`,
  `Project/command_runner` и тестов; сайты `change_delivery/git/mod.rs`, `change_delivery/git/integration.rs` и
  `project_setup/doctor.rs` вызывают владельцев; Delivery, ротация P085 и B076.
- Plan: P085 — active, P086 (B075) — pending.

## Verification

- Проба кандидата на копиях исторических файлов в scratch: `src/change_delivery_git.rs:151` на `04e7200^` и
  `src/scaffold_git.rs:15` на `50853e0^`, символ `success`, код выхода 1.
- На рабочем дереве: `cargo fmt` и `cargo clippy --all-targets -- -D warnings` чисто; `cargo test structural_lint` —
  118 passed; `just candidate lint --root /Users/admin/projects/AgentRig` — код 0.

## Blockers

Нет.

## Next action

Коммит VAC 1 через хук, затем VAC 2 для AC2 P085. Свободные ID: D127+, I022+, P087+, B079+ (B066 и D111
зарезервированы).
