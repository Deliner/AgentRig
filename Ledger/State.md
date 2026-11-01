# State

## Focus

**Набор II** — переносимые на любое приложение, agent-native улучшения качества (10 кластеров
C1–C10), по одной карточке за ветку, автономно. Порядок: C1 → C2 → C4 → C9 → **C6 эффекты/ошибки** →
C3 детерминизм → C5 контракты → C10 эволюция API → C7 анти-дублирование → C8 анти-переусложнение.
Доставлены **C1+C2 (P043)**, **C4 (P044)**, **C9 (P045)**. Сейчас — **C6 (B037/P046)**.

## Workspace

Ветка feature/effects (база master @ `756d69e` — merge-коммит C9), HEAD `2a61311` (VAC-B1), VAC-B2 в
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
`catalog::ALL` держится вне B1 (иначе I020 падает). stable = `2a61311` (мид-ветка бустрап после B1;
`tooling/distribution/stable.txt` в рабочем дереве указывает на него, но НЕ закоммичен — финальный пин
D051 на голову B2 будет на финале). `.agents/skills` → симлинк на `Project/shared/skills`, так что для
config-check навык «существует» через файл+плоский симлинк (пересборка не нужна).

УРОК (порядок доставки): живое правило в `lint.yaml` нельзя добавлять раньше создания навыка, на
который оно ссылается (`config-check` падает `paths.lint: No such file`, хук fail-closed блокирует
Bash/Edit/Write): навык/`catalog`/`assets` сначала, живое правило — последним. УРОК (дедлок): бустрап
только АТОМАРНО (запись stable.txt + `bash tooling/distribution/bootstrap.sh` в одном `just scratch`).

## Progress

C6 ядро — panic-путь в ПОСТАВЛЯЕМОМ линтере: `.unwrap()`/`.expect(...)` и макросы
`panic!`/`todo!`/`unimplemented!`/`unreachable!` без ссылки-обоснования прямо выше, тестовый код
исключён; механизм justified (как unsafe D065/allow D080).

**VAC-A закоммичен** (`64e2b18`): B037/P046 заведены и промоутнуты, C6-анализ записан.

**VAC-B1 закоммичен** (`2a61311`, source, полный гейт зелёный): `Kind::PanicPath` (kind.rs); const
`PANICPATH` + arm (descriptor.rs); `Kind::PanicPath` в rust-правилах (registry.rs); детектор в rust.rs
(`// DECISION: D082`) — диспетчер `call_expression` + расширенный `macro_invocation`, хелперы
`panic_call`/`panic_macro`/`report_panic`/`in_test_context`/`test_attributed`/`test_marker`; тест
`rust_flags_panic_paths_outside_test_code` в markers.rs; карточка D082 + индекс. Факт AST (спайк):
атрибуты — предшествующие соседи элемента; `.unwrap()` = `call_expression{function: field_expression
{field: field_identifier}}`.

**VAC-B2 готов в рабочем дереве, ещё не закоммичен** (delivery): навык `handle-panic`
(`references/handle-panic/SKILL.md`+`architecture.yaml`, плоский симлинк, инвентарь
`references/architecture.yaml`, запись в `assets.rs`); `Kind::PanicPath` в `catalog::ALL`; линки D082
(+`[repository rule]`,`[skill]`); живое правило `panic-path` в `lint.yaml` (include `**`, exclude
`**/tests/**`+`**/tests.rs`, .rs, error, навык handle-panic). Инлайновые `#[cfg(test)]`-модули в боевых
файлах (git.rs/input.rs/shared) снимает обход предков; сиблинги `tests.rs` и каталоги `tests/` — exclude.

## Verification

VAC-B1: полный гейт зелёный при коммите `2a61311` (build/lint/memory/rustfmt/clippy/rust-test 74/ruff/
mypy/test 202/typos/vulture); `cargo +1.98.1 test --lib markers` → 5 passed. stable бустрапнут на
`2a61311`, `just config-check` зелёный (дедлока нет). VAC-B2: `just candidate lint` → 0 находок
(боевой код без паники, тесты сняты); `just config-check` валиден с живым правилом. Полный гейт — при
коммите B2.

## Blockers

Нет.

## Next action

1. Застейджить и закоммитить **VAC-B2** (handle-panic навык+symlink+inventory+assets, catalog PanicPath,
   lint.yaml правило, линки D082, State), сообщение `P046 VAC-B2: deliver panic-path rule and handle-panic
   skill`. Хук гоняет полный гейт (incl. I020 `test_delivery`).
2. Финал: ротация B037/P046 в Archive (карточки → Archive/Backlog/037.md, Archive/Plan/046.md; флип
   D082 `[card]` → Archive/Backlog/037.md); пин stable.txt на голову B2 + бустрап атомарно (D051), коммит
   пина; `just feature-merge` (ветку оставить, НЕ пушить). Затем C3 (следующий кластер Набора II).
   Свободные ID: P047, B038, D083+.
