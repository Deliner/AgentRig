# State

## Focus

Пользователь 2026-11-10 задал стандарт рига, записан в AGENTS.md и D151. P110–P131 выполнены и перенесены в Archive.
B107 о сверке имён узлов с грамматикой сохранена, но в работу не взята: она требует дополнительного согласования,
пользователь вернётся к ней сам. B112 ждёт доступа к Linux-машине. B109 и B126 сохранены, в работу не выбраны.

2026-11-13 пользователь одобрил карточку B128 о Rust 1.100 как стандарте рига и проектов: «Давай оставим только 1.100
Сохраняй и выполняй». Выбрана P132, она активна.

## Workspace

Branch: `feature/rust-1100`, от master `b0ca86b`, где влита P131.
Revision: `c8b3434ad4fb2ebf6ce121d5e13e8be61d66a1ad`

Установленный рантайм `b28fd10`, он же в `stable.txt`. macOS: cargo, кандидат, коммиты и `feature-merge` через
`just scratch -- 'export PATH=/Users/admin/.cargo/bin:$PATH && …'`. Ветки прежних поставок сохранены,
`feature/init-probe` не трогать.

## Progress

- VAC 1 P132 `c55e2bd` сохранил B128 и выбрал P132.
- VAC 2 P132 перевёл риг на корневой `rust-toolchain.toml` с 1.100.0 по D173: гейт, сборка рантайма, bootstrap, CI,
  README и фикстуры `rustc` больше не называют версию.
- VAC 3 P132 вынес `cargo.rs` и `clippy.rs` в `Project/project_setup/rust/` без смены поведения: в
  `project_setup` нет места для модуля версии Rust. Бюджет зависимых `shared/fs_with_context` поднят до 23.

## Verification

- VAC 2: на 1.100.0 rustfmt, Clippy и 484 теста гейта прошли. Путь bootstrap с архивом файла toolchain проверит
  пин D051.
- VAC 3: кандидат собирается, `lint`, `memory-check` и `cargo fmt --check` проходят.

## Blockers

Нет.

## Next action

AC6 B128: проверка версии Rust проекта в init и doctor, затем AC7, AC3–AC4, правило старых форм AC8 двухфазно с
мид-бранч пином и AC2.
Свободные ID: D174+, I022+, P133+, B129+ (B066 и D111 зарезервированы).
