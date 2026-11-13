# State

## Focus

Пользователь 2026-11-10 задал стандарт рига, записан в AGENTS.md и D151. P110–P131 выполнены и перенесены в Archive.
B107 о сверке имён узлов с грамматикой сохранена, но в работу не взята: она требует дополнительного согласования,
пользователь вернётся к ней сам. B112 ждёт доступа к Linux-машине. B109 и B126 сохранены, в работу не выбраны.

2026-11-13 пользователь одобрил карточку B128 о Rust 1.100 как стандарте рига и проектов: «Давай оставим только 1.100
Сохраняй и выполняй». Выбрана P132, она активна.

## Workspace

Branch: `feature/rust-1100`, от master `b0ca86b`, где влита P131.
Revision: `d638a34950f75dcd049bf91d4cfe80988b9dcf6b`

Установленный рантайм `d638a34`, он же в `stable.txt`. macOS: cargo, кандидат, коммиты и `feature-merge` через
`just scratch -- 'export PATH=/Users/admin/.cargo/bin:$PATH && …'`. Ветки прежних поставок сохранены,
`feature/init-probe` не трогать.

## Progress

- VAC 1 P132 `c55e2bd` сохранил B128 и выбрал P132.
- VAC 2 P132 перевёл риг на корневой `rust-toolchain.toml` с 1.100.0 по D173: гейт, сборка рантайма, bootstrap, CI,
  README и фикстуры `rustc` больше не называют версию.
- VAC 3 P132 вынес `cargo.rs` и `clippy.rs` в `Project/project_setup/rust/` без смены поведения: в
  `project_setup` нет места для модуля версии Rust. Бюджет зависимых `shared/fs_with_context` поднят до 23.
- VAC 4 P132 закрыл AC6 по D174: `init --language rust` и `doctor` требуют `rustc` не старше версии рига.
- VAC 5 P132 `3f6bda1` закрыл AC7 по D175: раздел «Rust 1.100» в AGENTS.md рига и Rust-проектов.
- VAC 6 P132 `10390e9` закрыл AC3 и AC4 по D176: `unused_dependencies = "deny"` и карантин новых версий в 3 дня.
- VAC 7 P132 `ea6a586` собрал детекторы тестового кода `false_pass.rs` и `test_placement.rs` и их тесты в папки
  `test_code` без смены поведения: в `languages/rust` и `languages/tests` по 15 записей, предел lint.
- VAC 8 P132 `494d987` ввёл по D177 вид `superseded-form` с детектором, тестами и навыком `use-the-current-form`,
  пока вне каталога: I020 требует одинаковых видов в init и `.agentrig/lint.yaml`.
- VAC 9 P132 `d638a34` добавил вид в реестр Rust: без этого пинованный рантайм отвергнет правило для `.rs`.
- VAC 10 P132 закрыл AC8 и AC2: пин D051 на `d638a34`, вид в `capabilities::ALL` и правило в `.agentrig/lint.yaml`,
  10 мест рига переписаны на `trim_prefix`, `trim_suffix` и `Vec::from_fn`, ручные Drop-стражи тестов
  `change_delivery` заменены на `DropGuard`.

## Verification

- VAC 2–5 прошли гейт, VAC 5 со всеми 488 тестами; путь bootstrap с архивом toolchain проверит пин D051.
- VAC 6: Clippy гейта в копии рига с лишней `hex` даёт код 101, без неё 0; `generate-lockfile` из корня берёт
  `sonic-rs` 0.5.10 вместо опубликованной минуту назад 0.5.11. Транзитивную `memchr` cargo не замечает.
- VAC 7 прошёл гейт с 433 тестами.
- VAC 8 прошёл гейт с 438 тестами. С видом в каталоге до включения в `.agentrig/lint.yaml` падал тест I020.
- VAC 9 прошёл гейт с 435 тестами. Кандидат с одним правилом `superseded-form` нашёл в риге 10 мест:
  8 `strip_*(..).unwrap_or(..)` и 2 `(0..n).map(f).collect()`.
- VAC 10: rustfmt чист, 395 тестов прошли, включая I020 и `cleanup`. lint установленного рантайма `d638a34` с
  включённым правилом находок не дал; `lint-rules` кандидата описывает `superseded-form`.

## Blockers

Нет.

## Next action

Проверка AC5 через `git grep` номера версии вне разрешённых мест, затем закрытие P132: доказательства приёмки в
`Plan/132.md`, перенос P132 и B128 в Archive, финальный пин D051 и `just feature-merge` с сохранением ветки.
AGENTS.md рига на пределе в 70 строк.
Свободные ID: D178+, I022+, P133+, B129+ (B066 и D111 зарезервированы).
