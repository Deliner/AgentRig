# State

## Focus

**Набор II** — переносимые на любое приложение, agent-native улучшения качества (10 кластеров
C1–C10), по одной карточке за ветку, автономно. Порядок: C1 → C2 → C4 → C9 → C6 → C3 → C5 →
**C10 эволюция API/данных** → C7 анти-дублирование → C8 анти-переусложнение.
Доставлены **C1+C2 (P043)**, **C4 (P044)**, **C9 (P045)**, **C6 (P046)**, **C3 (P047)**, **C5 (P048)**.
Сейчас — **C10 эволюция API/данных** как [P049](Plan/049.md) (B040). Свободные ID: D085+, P050, B041+.

## Workspace

Ветка feature/evolution (база master @ `ef1a0b5` — merge-коммит C5). Машина — macOS.
Сборка/кандидат/коммит/бутстрап/git-мутации — через `just scratch -- '...'` с восстановленным PATH
(`cargo` в `~/.cargo/bin`); Agent-Bash — одна top-level `just`-команда без `&&`. Поиск — `just read --
grep ...` (без пайпов, globʼы в кавычках).

Механизм рантайма: хук/`config-check`/оркестрация гейта идут через `tooling/worker/run` →
УСТАНОВЛЕННЫЙ бинарь `.cache/development/<stable-hash>/` (из `tooling/distribution/stable.txt`, сейчас
`c527bf0…` — голова C5). `lint`/build/clippy/rustfmt/rust-test и `init` (I020) используют КАНДИДАТА.
C10 — **новый вид правила** `wildcard-import` (как C9/C6/C5), поэтому путь двухфазный:
**VAC-B1** (source: kind/descriptor/детекторы в rust.rs+python.rs/registry/тесты/D085 — компилируется в
кандидат, `lint.yaml` не трогается) → **мид-ветка атомарный бутстрап** (stable → голова B1, установить) →
**VAC-B2** (delivery: `catalog::ALL` += вид, `lint.yaml` += живое правило, навык `narrow-import` + плоский
симлинк + инвентарь + `assets.rs`). Иначе stable-бинарь не десериализует новый `kind:` из `lint.yaml` →
гейт закрывается.

## Progress

C10: новый вид `wildcard-import` + механизм justified (как panic-path/debug-assertion). Детекторы:
Rust — узел `use_wildcard` (glob `use …::*`, верхнего уровня или в группе), исключение тестов обходом
предков `in_test_context`; Python — узел `wildcard_import` внутри `import_from_statement`
(`from … import *`). Оба — находка без ссылки-обоснования `DECISION:`/`INVARIANT:` на строке прямо выше.
Навык ремонта `narrow-import` (сузить до явного списка имён или обосновать). В репозитории реальных
glob-импортов нет (единственное `::*` — строковый литерал в тест-фикстуре `architecture/source/tests.rs`,
AST-детектор его не ловит; Python `import *` — ноль) → догфуд будет зелёным.

VAC-A (память): созданы `Backlog/040.md`, `Plan/049.md`; строки в `Backlog.md`/`Plan.md`; активная строка
P049; проза промоушена; State обновлён. Коммитится через гейт.

## Verification

C5 влит в master (`ef1a0b5`), гейт `just feature-merge` зелёный (rust-test 101, pytest 306). Ветка
feature/evolution создана от `ef1a0b5`. Для C10 кода ещё нет (VAC-A — только память). Имена узлов
tree-sitter подтверждены по анализатору зависимостей: Rust `use_wildcard`
(`architecture/source/rust/mod.rs`), Python `import_from_statement`/`wildcard_import`
(`architecture/source/python.rs`).

## Blockers

Нет.

## Next action

1. VAC-B1 (source): `rules/kind.rs` += `WildcardImport`; `rules/descriptor.rs` += дескриптор + match-рука;
   `languages/rust.rs` += детектор на `use_wildcard`; `languages/python.rs` += детектор на
   `import_from_statement`/`wildcard_import`; `languages/registry.rs` += вид в оба списка `rules`;
   `languages/tests/markers.rs` += тесты Rust+Python; `Ledger/Decisions/085.md` + строка индекса + маркеры
   `// DECISION: D085`. Коммит через гейт (`lint.yaml`/`catalog` НЕ трогать).
2. Мид-ветка атомарный бутстрап: stable.txt = полный 40-символьный хеш головы VAC-B1 + `just bootstrap`
   (один scratch, откат при сбое); коммит пина.
3. VAC-B2 (delivery): `catalog::ALL` += `WildcardImport`; навык `narrow-import`
   (`references/narrow-import/SKILL.md`+`architecture.yaml` + плоский симлинк + инвентарь
   `references/architecture.yaml`) + `assets.rs`; `lint.yaml` += живое правило `wildcard-import`
   (навык-файлы ПЕРВЫМИ, живое правило ПОСЛЕДНИМ); расширить ссылки D085. Коммит через гейт (I020 зелёный).
4. Архив B040/P049 (`git rm` живых карточек, перенос в Archive, флип ссылки card в D085), финальный
   D051-пин, `just feature-merge` (ветку оставить, НЕ пушить). Отчёт по C10 на русском. Затем C7
   (анти-дублирование).
