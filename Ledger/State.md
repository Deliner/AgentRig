# State

## Focus

Пользователь 2026-11-10 задал стандарт рига, записан в AGENTS.md и D151. P110–P137 выполнены и перенесены в Archive.
B107 о сверке имён узлов с грамматикой сохранена, но в работу не взята: она требует дополнительного согласования,
пользователь вернётся к ней сам. B112 ждёт доступа к Linux-машине. B109 и B126 сохранены, в работу не выбраны.

2026-11-13 пользователь одобрил карточки B133 и B134 и поручил: «Сохрани обе и сделай сначала вторую». P137 по B134
выполнена и влита. Идёт P138 по B133, решение D182.

## Workspace

Branch: `feature/rust-flags`, от master `b50b14e`, где влита P137.
Revision: `b50b14ed6bf58ab6a8248ac6a112a7d343a20e84`

Установленный рантайм `ca7e034`, он же в `stable.txt`; он ещё не знает вид `cargo-rustflags` и разбор YAML. macOS:
cargo, кандидат, коммиты и `feature-merge` через `just scratch -- 'export PATH=/Users/admin/.cargo/bin:$PATH && …'`.
Ветки прежних поставок сохранены и отправлены в `origin`, `feature/init-probe` не трогать.

## Progress

- P138 фаза 1, VAC 1: список `rustflags` в `.agentrig/rust.yaml`, `CARGO_ENCODED_RUSTFLAGS` для каталога, гейта,
  фоновых задач, doctor и scratch; `allow-justification` читает YAML, новый вид `cargo-rustflags`, навык
  `move-rustflags`; `init` копирует `build.rustflags`. Навыки `justify-allow` и `move-rustflags` собраны в
  `references/lint-levels`, тесты TOML и YAML — в `languages/tests/configuration`. D182, README и AGENTS.md обновлены.
- Обработчик TOML уже принимает `cargo-rustflags`, но в каталог `init` вид войдёт в фазе 2 вместе с правилом рига
  (I020); до этого тест `shipped_rules_keep_compiler_flags_in_the_rig_list` ждёт только `allow-justification`.
- Фаза 2 ещё не начата: пин D051, затем правила в `.agentrig/lint.yaml` и модуль `.agentrig/rust.yaml` рига.

## Verification

На рабочем дереве VAC 1: `just run clippy` чист, `just lint` чист, тесты `project_setup::` и `structural_lint::`
356 прошли, фокусные тесты флагов и навыков прошли.

## Blockers

Нет.

## Next action

Закоммитить VAC 1, записать его голову в `stable.txt` с `just bootstrap` и закоммитить пин. Затем фаза 2: правила
`allow-justification` для `.yaml` и `cargo-rustflags` в `.agentrig/lint.yaml`, `.agentrig/rust.yaml` с пустым списком.
Свободные ID: D183+, I022+, P139+, B135+.
