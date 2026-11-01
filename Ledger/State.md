# State

## Focus

**Набор II** — переносимые на любое приложение, agent-native улучшения качества (10 кластеров
C1–C10), по одной карточке за ветку, автономно. Порядок: C1 → C2 → C4 → C9 → C6 → **C3 детерминизм**
→ C5 контракты → C10 эволюция API → C7 анти-дублирование → C8 анти-переусложнение.
Доставлены **C1+C2 (P043)**, **C4 (P044)**, **C9 (P045)**, **C6 (P046)**. Следующий — **C3**.

## Workspace

Ветка feature/effects (база master @ `756d69e`). C6 (B037/P046) доставлен и заархивирован; идёт финал
ветки (пин stable + merge). Машина — macOS.
Сборка/кандидат/коммит/бутстрап — через `just scratch -- '...'` с восстановленным PATH (`cargo` в
`~/.cargo/bin`); Agent-Bash — одна top-level `just`-команда без `&&`. Удаление/перенос файлов — тоже
через scratch (`git rm`); scratch исполняется в своей песочнице, но эффект git применяется к репозиторию.

Механизм рантайма: хук/`config-check`/оркестрация гейта идут через `tooling/worker/run` →
УСТАНОВЛЕННЫЙ бинарь `.cache/development/<stable-hash>/` (из `tooling/distribution/stable.txt`); НЕ
per-HEAD. `lint`/build/clippy/rustfmt/rust-test и `init` (I020) используют КАНДИДАТА. Новый вид правила
идёт в ДВА VAC (B1 source → мид-ветка бустрап → B2 delivery); `catalog::ALL` держится вне B1 (иначе I020
падает). stable сейчас = `2a61311` (мид-ветка бустрап после B1, `stable.txt` в рабочем дереве указывает
на него, НЕ закоммичен). Финальный пин D051 — на gate-принятую голову архивного коммита C6.

УРОК (порядок доставки): живое правило в `lint.yaml` нельзя добавлять раньше создания навыка, на
который оно ссылается (`config-check` падает `paths.lint: No such file`, хук fail-closed): навык/
`catalog`/`assets` сначала, живое правило — последним. УРОК (дедлок): бустрап только АТОМАРНО (запись
stable.txt + `bash tooling/distribution/bootstrap.sh` в одном `just scratch`).

## Progress

C6 ядро — panic-путь в ПОСТАВЛЯЕМОМ линтере: `.unwrap()`/`.expect(...)` и макросы
`panic!`/`todo!`/`unimplemented!`/`unreachable!` без ссылки-обоснования прямо выше, тестовый код
исключён; механизм justified (как unsafe D065/allow D080).

- **VAC-A** `64e2b18`: B037/P046 заведены и промоутнуты, C6-анализ записан.
- **VAC-B1** `2a61311` (source): `Kind::PanicPath`, const `PANICPATH`+arm, registry, детектор в rust.rs
  (`call_expression` + расширенный `macro_invocation`; хелперы `panic_call`/`panic_macro`/`report_panic`/
  `in_test_context`/`test_attributed`/`test_marker`), тест `rust_flags_panic_paths_outside_test_code`,
  карточка D082 + индекс.
- **VAC-B2** `011e446` (delivery): навык `handle-panic` (references/SKILL.md+architecture.yaml, плоский
  symlink, инвентарь, `assets.rs`), `Kind::PanicPath` в `catalog::ALL`, линки D082, живое правило
  `panic-path` в `lint.yaml` (include `**`, exclude `**/tests/**`+`**/tests.rs`, .rs, error).
- **Архивный VAC (идёт)**: B037/P046 перенесены в Archive (Archive/Plan/046.md с `## Delivery`,
  Archive/Backlog/037.md; индексы; Plan-проза; оригиналы удалены `git rm`); ссылка `[card]` D082
  переведена на `Archive/Backlog/037.md`; State обновлён.

## Verification

VAC-B2 коммит `011e446`: полный гейт зелёный (build/lint/memory/rustfmt/clippy/rust-test 97/ruff-format/
ruff/mypy/test 306 вкл. test_delivery(I020)/typos/vulture). `just candidate lint` → 0 panic-path находок;
`just config-check` валиден. Архивный VAC — проверяется коммит-гейтом (memory-check после ротации).

## Blockers

Нет.

## Next action

1. Закоммитить **архивный VAC** (Ledger-ротация C6, НЕ stable.txt), сообщение
   `P046: archive B037/P046 (C6 delivered)`. Хук гоняет полный гейт + memory-check.
2. **Пин D051**: записать `tooling/distribution/stable.txt` = голова архивного коммита + `bash
   tooling/distribution/bootstrap.sh` АТОМАРНО в одном `just scratch`; закоммитить пин
   `P046: pin stable.txt to <hash> (D051)`.
3. `just feature-merge` в master (ветку оставить, НЕ пушить). Отчёт по C6 на русском.
4. Затем **C3** (детерминизм) — следующий кластер Набора II. Свободные ID: P047, B038, D083+.
