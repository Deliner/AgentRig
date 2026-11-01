# State

## Focus

**Набор II** — переносимые на любое приложение, agent-native улучшения качества (10 кластеров
C1–C10), по одной карточке за ветку, автономно. Порядок: C1 → C2 → C4 → C9 → C6 → C3 →
**C5 контракты поведения** → C10 эволюция API → C7 анти-дублирование → C8 анти-переусложнение.
Доставлены **C1+C2 (P043)**, **C4 (P044)**, **C9 (P045)**, **C6 (P046)**, **C3 (P047)**, **C5 (P048)**.
Следующий — **C10 эволюция API/данных**. Свободные ID: P049, B040, D085+.

## Workspace

Ветка feature/contracts (база master @ `fdc4188` — merge-коммит C3). Машина — macOS.
Сборка/кандидат/коммит/бутстрап/git-мутации — через `just scratch -- '...'` с восстановленным PATH
(`cargo` в `~/.cargo/bin`); Agent-Bash — одна top-level `just`-команда без `&&`. Поиск — `just read --
grep ...` (без пайпов, globʼы в кавычках).

Механизм рантайма: хук/`config-check`/оркестрация гейта идут через `tooling/worker/run` →
УСТАНОВЛЕННЫЙ бинарь `.cache/development/<stable-hash>/` (из `tooling/distribution/stable.txt`, сейчас
`8a97e0e…` — голова архива C3). `lint`/build/clippy/rustfmt/rust-test и `init` (I020) используют
КАНДИДАТА. C5 — **новый вид правила** `debug-assertion` (как C9/C6), поэтому путь двухфазный:
**VAC-B1** (source: kind/descriptor/детектор в rust.rs/registry/тест/D084 — компилируется в кандидат,
`lint.yaml` не трогается) → **мид-ветка атомарный бутстрап** (stable → голова B1, установить) →
**VAC-B2** (delivery: `catalog::ALL` += вид, `lint.yaml` += живое правило, навык `firm-up-assertion` +
плоский симлинк + инвентарь + `assets.rs`). Иначе stable-бинарь не десериализует новый `kind:` из
`lint.yaml` → гейт закрывается.

## Progress

C5: новый вид `debug-assertion` + механизм justified (как unsafe/allow/panic/skip). Детектор —
`assertion_macro`/`report_assertion` в `languages/rust.rs` (параллельно `panic_macro`/`report_panic`),
вызов в ветке `"macro_invocation"`; имена `debug_assert`/`debug_assert_eq`/`debug_assert_ne`; обход
предков `in_test_context` исключает `#[cfg(test)]`/`#[test]`; каталоги `tests/` и `tests.rs` — паттерны
exclude живого правила. Навык ремонта `firm-up-assertion` (поднять до `assert!`, убрать или обосновать).
`assert!` не трогается. В репозитории вызовов `debug_assert!` нет (только строковые литералы в
`resolution/rust/macros.rs` и `architecture/source/rust/conditions.rs` — AST-детектор их не ловит) →
догфуд будет зелёным.

C5 доставлен и функционально завершён: VAC-A `fa3755a` (память), VAC-B1 `5061b18` (source), мид-ветка
пин stable=`5061b18` (`787b27f`), VAC-B2 `1cb93b8` (delivery) — все через гейт, зелёные. Правило
`debug-assertion` ([D084](Decisions/084.md)) + навык `firm-up-assertion` поставлены в `init`; I020
сбалансирован (delivered == live); догфуд зелёный (`just candidate lint` → 0, `just config-check`
валидно).

**Ротация B039/P048 в Archive применена в рабочем дереве** (не закоммичена): созданы
`Archive/Plan/048.md` (с `## Delivery`) и `Archive/Backlog/039.md`, строки в `Archive/Plan.md`/
`Archive/Backlog.md`, проза завершения в `Plan.md`, активная строка P048 убрана, строка B039 убрана из
`Backlog.md`, ссылка card D084 → `Archive/Backlog/039.md`. Осталось `git rm` живых `Plan/048.md`+
`Backlog/039.md` и архив-коммит, затем финальный пин D051 и `feature-merge`.

## Verification

C3 влит в master (`fdc4188`). На ветке feature/contracts закоммичены VAC-A `fa3755a`, VAC-B1 `5061b18`,
мид-ветка пин `787b27f` (stable.txt=`5061b18`, установлен/бутстрапнут), VAC-B2 `1cb93b8` — каждый через
коммит-гейт, все зелёные. VAC-B2 весь гейт PASS: build/lint/memory/rustfmt/clippy/rust-test 101/
ruff-format/ruff/mypy/pytest 306 (вкл. test_delivery/I020)/typos/vulture. Догфуд: `just candidate lint`
→ 0 находок, `just config-check` → валидно. Архив-коммит ротации, финальный D051-пин и `feature-merge`
— предстоят.

## Blockers

Нет.

## Next action

1. `git rm` живых `Plan/048.md`+`Backlog/039.md` и коммит ротации `P048: archive B039/P048 (C5 delivered)`
   через гейт (всё остальное из ротации уже в рабочем дереве).
2. Финальный пин D051: stable.txt = полный 40-символьный хеш головы архив-коммита + атомарный бутстрап
   (один scratch, откат при сбое); коммит `P048: pin stable.txt to <short> (D051)`.
3. `just feature-merge` (ветку оставить, НЕ пушить). Отчёт по C5 на русском. Затем C10 (эволюция
   API/данных). Свободные ID: P049, B040, D085+.
