# State

## Focus

Пользователь 2026-11-10 задал стандарт рига, записан в AGENTS.md и D151. P110–P137 выполнены и перенесены в Archive.
B107 о сверке имён узлов с грамматикой сохранена, но в работу не взята: она требует дополнительного согласования,
пользователь вернётся к ней сам. B112 ждёт доступа к Linux-машине. B109 и B126 сохранены, в работу не выбраны.

2026-11-13 пользователь одобрил карточки B133 и B134 и поручил: «Сохрани обе и сделай сначала вторую». P137 по B134
выполнена и влита. Идёт P138 по B133, решение D182.

## Workspace

Branch: `feature/rust-flags`, от master `b50b14e`, где влита P137.
Revision: `a0acb292101980b2ce07561c61ea23285b3fde5f`

Установленный рантайм `a0acb29` — VAC 1 этой ветки, он же в `stable.txt`. macOS: cargo, кандидат, коммиты и
`feature-merge` через `just scratch -- 'export PATH=/Users/admin/.cargo/bin:$PATH && …'`. Ветки прежних поставок
сохранены и отправлены в `origin`, `feature/init-probe` не трогать.

## Progress

- VAC 1 `a0acb29`: список `rustflags` в `.agentrig/rust.yaml`, `CARGO_ENCODED_RUSTFLAGS` для каталога, гейта,
  фоновых задач, doctor и scratch; `allow-justification` читает YAML, вид `cargo-rustflags`, навык `move-rustflags`;
  `init` копирует `build.rustflags`; D182.
- VAC 2: пин `a0acb29`; в `.agentrig/lint.yaml` правило `allow-justification` читает `.yaml`, новое правило
  `cargo-rustflags` на `**/.cargo/config.toml`; `.agentrig/rust.yaml` рига с пустым списком; вид в каталоге `init`
  (I020); сообщение `allow-justification` называет и флаг, глушащий lint-ы.

## Verification

На рабочем дереве VAC 2: `just lint-config-check`, `just config-check` и `just lint` чисты; 356 тестов
`project_setup::` и `structural_lint::` прошли. Проба: `--cap-lints allow` в `.agentrig/rust.yaml` дал ошибку
`allow-justification`, `rustflags` во вложенном `.cargo/config.toml` — ошибку `cargo-rustflags`; файлы восстановлены.

## Blockers

Нет.

## Next action

Закоммитить VAC 2. Затем VAC 3: Delivery с таблицей AC, перенос P138 и B133 в Archive. Финальный пин головы и
`just feature-merge`. Свободные ID: D183+, I022+, P139+, B135+.
