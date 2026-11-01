# State

## Focus

**Набор II** — переносимые на любое приложение, agent-native улучшения качества (10 кластеров
C1–C10), по одной карточке за ветку, автономно. Порядок: C1 → C2 → C4 → C9 → **C6 эффекты/ошибки** →
C3 детерминизм → C5 контракты → C10 эволюция API → C7 анти-дублирование → C8 анти-переусложнение.
Доставлены **C1+C2 (P043)**, **C4 (P044)**, **C9 (P045)**. Сейчас — **C6 (B037/P046)**.

## Workspace

Ветка feature/effects (база master @ `756d69e` — merge-коммит C9), HEAD на старте C6. Машина — macOS.
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
(`#[cfg(test)]`/`#[test]`/`tests/`) исключён; механизм justified (как unsafe D065/allow D080).
**VAC-A готов в рабочем дереве, ещё не закоммичен**: B037 сохранён (карточка+строка Backlog), P046
промоутнут (active, карточка+строка Plan), C6-анализ записан прозой, State. Код правила ещё не писан.

## Verification

Пока нет (VAC-A — memory-only, проверится memory-check при коммите).

## Blockers

Нет.

## Next action

1. Закоммитить VAC-A (memory-only), сообщение `P046 VAC-A: register B037 (C6 effects), promote, record
   C6 analysis`.
2. **VAC-B1** (source): `Kind::PanicPath` (kind.rs) + descriptor + arm (descriptor.rs) + rust-handler
   rules (registry.rs) + детектор на `call_expression`/`macro_invocation` c именами
   `unwrap`/`expect`/`panic`/`todo`/`unimplemented`/`unreachable`, исключая `#[cfg(test)]`/`#[test]`,
   через `justified` + тест в `markers.rs` + решение D082. НЕ трогать `catalog::ALL`. Коммит.
3. Атомарный бустрап stable.txt на голову B1.
4. **VAC-B2** (delivery): правило в `lint.yaml`, `Kind::PanicPath` в `catalog::ALL`, навык (имя на
   выбор, напр. `handle-panic`) в `references/` + symlink + инвентарь + `assets.rs`, линки D082, догфуд
   (lib уже без unwrap по D060 → 0 находок, тесты исключены), State. СНАЧАЛА навык, ПОТОМ живое правило.
5. Финал: ротация B037/P046 в Archive, пин stable.txt + бустрап (D051), `just feature-merge`.
   Свободные ID: P047, B038, D082+.
