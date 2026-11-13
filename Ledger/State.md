# State

## Focus

Пользователь 2026-11-10 задал стандарт рига, записан в AGENTS.md и D151. P110–P131 выполнены и перенесены в Archive.
B107 о сверке имён узлов с грамматикой сохранена, но в работу не взята: она требует дополнительного согласования,
пользователь вернётся к ней сам. B112 ждёт доступа к Linux-машине. B109 и B126 сохранены, в работу не выбраны.

2026-11-13 пользователь одобрил карточку B128 о Rust 1.100 как стандарте рига и проектов: «Давай оставим только 1.100
Сохраняй и выполняй». Выбрана P132, она активна.

## Workspace

Branch: `feature/rust-1100`, от master `b0ca86b`, где влита P131.
Revision: `0f5bb42a25aa3fdb7b8d51f699a58fa4b360f34f`

Установленный рантайм `b28fd10`, он же в `stable.txt`. macOS: cargo, кандидат, коммиты и `feature-merge` через
`just scratch -- 'export PATH=/Users/admin/.cargo/bin:$PATH && …'`. Ветки прежних поставок сохранены,
`feature/init-probe` не трогать.

## Progress

- VAC 1 P132 `c55e2bd` сохранил B128 и выбрал P132.
- VAC 2 P132 перевёл риг на корневой `rust-toolchain.toml` с 1.100.0 по D173: гейт, сборка рантайма, bootstrap, CI,
  README и фикстуры `rustc` больше не называют версию.
- VAC 3 P132 вынес `cargo.rs` и `clippy.rs` в `Project/project_setup/rust/` без смены поведения: в
  `project_setup` нет места для модуля версии Rust. Бюджет зависимых `shared/fs_with_context` поднят до 23.
- VAC 4 P132 закрыл AC6 по D174: `init --language rust` и `doctor` требуют `rustc` не старше версии рига.

## Verification

- VAC 2 и VAC 3 прошли гейт; путь bootstrap с архивом файла toolchain проверит пин D051.
- VAC 4: rustfmt, Clippy, `lint` и `memory-check` проходят; `just run test -- project_setup::` дал 45 из 45, включая
  3 новых теста с поддельным `rustc` в PATH. Кандидат `doctor` на риге печатает `found 1.100.0, requires 1.100.0`.

## Blockers

Нет.

## Next action

AC7 B128: раздел о Rust в AGENTS.md рига и Rust-проектов из одного источника, затем AC3–AC4, правило старых форм
AC8 двухфазно с мид-бранч пином и AC2. AGENTS.md рига уже на пределе lint в 70 строк, раздел потребует сжатия.
Свободные ID: D175+, I022+, P133+, B129+ (B066 и D111 зарезервированы).
