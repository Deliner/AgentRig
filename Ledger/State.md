# State

## Focus

**Набор II** — переносимые на любое приложение, agent-native улучшения качества (10 кластеров
C1–C10), по одной карточке за ветку, автономно. Порядок: C1 → C2 → C4 → C9 → **C6 эффекты/ошибки** →
C3 детерминизм → C5 контракты → C10 эволюция API → C7 анти-дублирование → C8 анти-переусложнение.
Доставлены **C1+C2 (P043)**, **C4 (P044)**, **C9 (P045)**. Сейчас — **C6 (B037/P046)**.

## Workspace

Ветка feature/effects (база master @ `756d69e` — merge-коммит C9), HEAD `64e2b18` (VAC-A), VAC-B1 в
рабочем дереве. Машина — macOS.
Сборка/кандидат/коммит/бутстрап — через `just scratch -- '...'` с восстановленным PATH (`cargo` в
`~/.cargo/bin`); Agent-Bash — одна top-level `just`-команда без `&&`.

Механизм рантайма: хук/`config-check`/оркестрация гейта идут через `tooling/worker/run` →
УСТАНОВЛЕННЫЙ бинарь `.cache/development/<stable-hash>/` (из `tooling/distribution/stable.txt`); НЕ
per-HEAD. `lint`/build/clippy/rustfmt/rust-test и `init` (I020) используют КАНДИДАТА. stable =
`ace0781` (установлен). Новый вид правила нельзя назвать в живом `.agentrig/lint.yaml`, пока его не
знает stable-бинарь → правило идёт в ДВА VAC: **B1** = enum/descriptor/detector/registry/тесты
(+решение, memory-чекер требует marker→decision в том же коммите), затем мид-ветка бустрап stable.txt
на голову B1, затем **B2** = `lint.yaml` + `catalog::ALL` + навык + `assets.rs` + линки + догфуд.
`catalog::ALL` держится вне B1 (иначе I020 падает).

УРОК (порядок доставки): живое правило в `lint.yaml` нельзя добавлять раньше создания навыка, на
который оно ссылается (`config-check` падает `paths.lint: No such file`, хук fail-closed блокирует
Bash/Edit/Write): навык/`catalog`/`assets` сначала, живое правило — последним. УРОК (дедлок): бустрап
только АТОМАРНО (запись stable.txt + `bash tooling/distribution/bootstrap.sh` в одном `just scratch`).

## Progress

C6 ядро — panic-путь в ПОСТАВЛЯЕМОМ линтере: `.unwrap()`/`.expect(...)` и макросы
`panic!`/`todo!`/`unimplemented!`/`unreachable!` без ссылки-обоснования прямо выше, тестовый код
исключён; механизм justified (как unsafe D065/allow D080).

**VAC-A закоммичен** (`64e2b18`): B037/P046 заведены и промоутнуты, C6-анализ записан.

**VAC-B1 готов в рабочем дереве, ещё не закоммичен** (source): `Kind::PanicPath` (kind.rs); const
`PANICPATH` + arm (descriptor.rs, `// DECISION: D082`); `Kind::PanicPath` в rust-правилах
(registry.rs); детектор в rust.rs (`// DECISION: D082`) — диспетчер `call_expression` + расширенный
`macro_invocation`, хелперы `panic_call`/`panic_macro`/`report_panic`/`in_test_context`/
`test_attributed`/`test_marker`; тест `rust_flags_panic_paths_outside_test_code` в markers.rs
(`// DECISION: D082`); карточка D082 + строка индекса. `catalog::ALL`/`lint.yaml`/навык НЕ тронуты
(это B2). Факт AST (спайк): атрибуты — предшествующие соседи элемента; `.unwrap()` =
`call_expression{function: field_expression{field: field_identifier}}`.

## Verification

`cargo +1.98.1 test --lib markers` на кандидате → 5 passed (incl.
`rust_flags_panic_paths_outside_test_code`, находки `[2,3,6,9,10]`; обоснованные `.unwrap()`/`todo!`,
строковый литерал и тело `#[cfg(test)] mod`+`#[test]` не дают находок). Компиляция чистая. Полный
гейт — при коммите B1.

## Blockers

Нет.

## Next action

1. Застейджить и закоммитить **VAC-B1** (kind/descriptor/registry/rust/markers + D082 карточка+индекс +
   State), сообщение `P046 VAC-B1: add panic-path rule kind and detector; record D082`. Хук гоняет
   полный гейт (clippy на lib — мой код без unwrap/panic).
2. Атомарный бустрап stable.txt на голову B1 (запись stable.txt + `bash tooling/distribution/bootstrap.sh`
   в одном `just scratch`), коммит пина? Нет — пин stable на финале (D051). Здесь бустрап нужен, чтобы
   stable-бинарь узнал `panic-path` перед живым правилом в B2.
3. **VAC-B2** (delivery): правило `panic-path` в `lint.yaml` с `exclude: ['**/tests/**']`, `Kind::PanicPath`
   в `catalog::ALL`, навык `handle-panic` в `references/` (SKILL.md + architecture.yaml + flat symlink +
   инвентарь references/architecture.yaml) + `assets.rs`, линки D082 (+`[repository rule]`,`[skill]`),
   догфуд (`just candidate lint` → 0 находок), State. СНАЧАЛА навык/catalog/assets, ПОТОМ живое правило.
4. Финал: ротация B037/P046 в Archive, пин stable.txt на голову B2 + бустрап (D051), коммит пина,
   `just feature-merge` (ветку оставить, НЕ пушить). Свободные ID: P047, B038, D083+.
