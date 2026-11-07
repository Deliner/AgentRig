# State

## Focus

Программа **P083 → P086** (пользователь 2026-11-07: «бери все и делай») — карточки B075–B078 по одной за ветку,
автономно. P083 (B077) и P084 (B078) выполнены и слиты, master `36f3dbe`. **P085** (B076 exit-code) выполнена в VAC 2:
`std::process::Command` вне владельцев запрещён в `.agentrig/lint.yaml`, три сайта вызывают владельцев; P085 и B076 в
Archive, решение D126. Этот коммит — финальный пин D051.

## Workspace

Branch: `feature/exit-code`
Revision: `76de95940766aa62c233b42fe89a481728848bd8`

База — master `36f3dbe`. Ветки `feature/disallowed-methods`, `feature/impact-coverage` и прежние, а также
`feature/init-probe`, сохранены. Установленный рантайм — `76de959`. macOS: в PATH `just write`/`just run`/
`just check`/`just feature-merge` нет cargo — коммиты и вызовы с cargo через
`just scratch -- 'export PATH=/Users/admin/.cargo/bin:$PATH && cd /Users/admin/projects/AgentRig && …'`.

## Progress

- P085 VAC 1 `f067061`: форма `silent-error` для `Ok(<status>.success())` в функции `Result<bool>` без `.code()`,
  тесты, текст правила, навыки `handle-error` и `handle-discarded`, D126.
- P085 VAC 2: запись `std::process::Command` в `disallowed`; колбэк `integration::integrate` получает аргументы git,
  `execute_vcs` строит команду через `Repository::command`; `doctor` запускает бинарник через `sandbox::tracked`.
  Delivery в `Archive/Plan/085.md`, ротация P085 и B076, ссылки D126 на Archive и конфиг.
- Plan: P086 (B075) — pending.

## Verification

- Запись `std::process::Command` до ремонта: `just candidate lint` — 4 находки в трёх файлах, как в карточке.
- После ремонта: `cargo fmt` и `cargo clippy --all-targets -- -D warnings` чисто; `just candidate lint` — код 0;
  `test_doctor.py` и `test_git.py` — 13 passed.
- Гейт коммита `f067061`: все проверки PASS, test 299 passed.
- Гейт коммита `76de959`: все проверки PASS, rust-test 155 и test 224 passed; `just bootstrap` на `76de959` —
  установлен `agentrig 0.3.0`.

## Blockers

Нет.

## Next action

`just feature-merge` ветки `feature/exit-code`. Потом ветка `feature/clippy-types` для
P086 (B075). Свободные ID: D127+, I022+, P087+, B079+ (B066 и D111 зарезервированы).
