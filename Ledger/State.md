# State

## Focus

**Набор II** — переносимые на любое приложение, agent-native улучшения качества (10 кластеров
C1–C10), по одной карточке за ветку, автономно. Порядок: C1 → C2 → C4 → C9 → C6 → C3 →
**C5 контракты поведения** → C10 эволюция API → C7 анти-дублирование → C8 анти-переусложнение.
Доставлены **C1+C2 (P043)**, **C4 (P044)**, **C9 (P045)**, **C6 (P046)**, **C3 (P047)**. Текущий — **C5
контракты поведения (B039/P048)**.

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

**VAC-A закоммичен** (`fa3755a`, память/промоушен; гейт весь PASS). **VAC-B1 закоммичен** (source):
`kind.rs` (+`DebugAssertion`), `descriptor.rs` (+`DEBUGASSERTION`), `rust.rs`
(+`assertion_macro`/`report_assertion`, вызов в `macro_invocation`), `registry.rs` (+вид в rust-handler),
тест `rust_flags_debug_assertions_outside_test_code` в `markers.rs`, `Decisions/084.md` + индекс-строка
(на B1 линкует только существующих владельцев: card + descriptor + rust + tests). `lint.yaml`,
`catalog::ALL` и навык НЕ трогались → гейт зелёный, I020 сбалансирован (init без правила, live без правила).
Далее — **мид-ветка атомарный бутстрап** (stable → голова B1), затем VAC-B2 (delivery), ротация,
финальный пин D051, `feature-merge`.

## Verification

C3 влит в master (`fdc4188`), пин stable `8a97e0e` (установлен, бутстрапнут). На ветке
feature/contracts коммитов ещё нет; рабочее дерево — только память VAC-A (Backlog/039.md,
Backlog.md, Plan/048.md, Plan.md, State.md). Коммит-гейт для VAC-A, затем код VAC-B1/VAC-B2 через гейт —
предстоят.

## Blockers

Нет.

## Next action

1. Коммит **VAC-A** (память/промоушен) через гейт: `P048 VAC-A: register B039 (C5 behavior contracts),
   promote, record analysis`. (Первый VAC-коммит контекста — ожидается Required-маршрут edit-state:
   прочитать `edit-state/SKILL.md`, повторить тот же коммит.)
2. **VAC-B1** (source): `rules/kind.rs` (+`DebugAssertion`), `rules/descriptor.rs` (+`DEBUGASSERTION`
   const + arm, `// DECISION: D084`), `languages/rust.rs` (+`assertion_macro`/`report_assertion`),
   `languages/registry.rs` (+`Kind::DebugAssertion` в rust-handler), тест в `languages/tests/markers.rs`,
   `Decisions/084.md` + индекс. Коммит `P048 VAC-B1: register debug-assertion kind + detector; record D084`.
3. **Мид-ветка атомарный бутстрап**: stable.txt → полный 40-символьный хеш головы VAC-B1 + бутстрап
   (один scratch, откат при сбое). Коммит `P048 VAC-B1: pin stable.txt to <short> for mid-branch delivery (D051)`.
4. **VAC-B2** (delivery): `catalog::ALL` += `DebugAssertion`; `.agentrig/lint.yaml` += блок
   `debug-assertion` (зеркало excludes `panic-path`, навык `firm-up-assertion`, `# DECISION: D084`);
   создать `Project/shared/skills/references/firm-up-assertion/SKILL.md` + `architecture.yaml`; плоский
   симлинк `Project/shared/skills/firm-up-assertion -> references/firm-up-assertion` (ln через scratch);
   `references/architecture.yaml` += directories/purpose; `project_setup/assets.rs` += firm-up-assertion;
   ссылки D084; догфуд (`just candidate lint`, `config-check`). Коммит
   `P048 VAC-B2: deliver debug-assertion rule + firm-up-assertion skill`.
5. **Ротация** B039/P048 в Archive, проза завершения, `git rm` живых, флип card-ссылки D084. Коммит
   `P048: archive B039/P048 (C5 delivered)`.
6. **Финальный пин D051**: stable.txt = полный хеш головы архив-коммита + атомарный бутстрап; коммит
   `P048: pin stable.txt to <short> (D051)`. Затем `just feature-merge` (ветку оставить, НЕ пушить).
   Отчёт по C5 на русском. Свободные ID после C5: P049, B040, D085+.
