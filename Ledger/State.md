# State

## Focus

**Набор II** — переносимые на любое приложение, agent-native улучшения качества (10 кластеров
C1–C10), по одной карточке за ветку, автономно. Порядок: C1 → C2 → C4 → C9 → C6 → C3 → C5 → C10 →
**C7 анти-дублирование** → C8 анти-переусложнение.
Доставлены **C1+C2 (P043)**, **C4 (P044)**, **C9 (P045)**, **C6 (P046)**, **C3 (P047)**, **C5 (P048)**,
**C10 (P049)**. C10 функционально готов и зелёный; идёт интеграция ветки. Свободные ID: D086+, P050, B041+.

## Workspace

Ветка feature/evolution (база master @ `ef1a0b5` — merge-коммит C5). Машина — macOS.
Сборка/кандидат/коммит/бутстрап/git-мутации — через `just scratch -- '...'` с восстановленным PATH
(`cargo` в `~/.cargo/bin`); Agent-Bash — одна top-level `just`-команда без `&&`. Поиск — `just read --
grep ...` (без пайпов, globʼы в кавычках).

Механизм рантайма: хук/`config-check`/оркестрация гейта идут через `tooling/worker/run` →
УСТАНОВЛЕННЫЙ бинарь `.cache/development/<stable-hash>/` (из `tooling/distribution/stable.txt`, сейчас
`a4961dd…` — голова VAC-B1, установлена мид-веткой). `lint`/build/clippy/rustfmt/rust-test и `init`
(I020) используют КАНДИДАТА. Установленный бинарь уже знает вид `wildcard-import` (десериализует
`lint.yaml`).

## Progress

C10 доставлен правилом `wildcard-import` ([D085](Decisions/085.md)) через две фазы: VAC-A `cb10b45`
(память), VAC-B1 `a4961dd` (source: kind/descriptor/детекторы rust.rs+python.rs/registry/тесты),
мид-ветка пин stable=`a4961dd` (`971462b`), VAC-B2 `1f14e3c` (delivery: `catalog::ALL` += вид, живое
правило в `lint.yaml`, навык `narrow-import` + плоский симлинк + инвентарь + `assets.rs`). Детекторы:
Rust — узел `use_wildcard` (glob вне тестов, исключение обходом предков `in_test_context`), Python — узел
`wildcard_import` в `import_from_statement`; оба — находка без ссылки-обоснования прямо выше (механизм
`justified`). I020 сбалансирован (виды правил и навыки выросли на один).

Сейчас — **архивная ротация B040/P049**: созданы `Archive/Backlog/040.md`, `Archive/Plan/049.md`
(с `## Delivery`, глубина ссылок `../../`); строки добавлены в `Archive/Backlog.md`/`Archive/Plan.md`;
активная строка P049 убрана из `Plan.md` (+ проза завершения), строка B040 убрана из `Backlog.md`; ссылка
card в индексе D085 переведена на `Archive/Backlog/040.md`; State обновлён. Осталось `git rm` живых
`Backlog/040.md`+`Plan/049.md` и закоммитить архив через гейт.

## Verification

C10 функционально готов и зелёный: коммит-гейт VAC-B2 (`1f14e3c`) весь PASS — build/lint/memory/rustfmt/
clippy/rust-test 103/ruff-format/ruff/mypy/pytest 306 (вкл. test_delivery/I020)/typos/vulture. Наш код
зелёный: реальных glob-импортов в репозитории нет (единственное `::*` — строковый литерал в тест-фикстуре
`architecture/source/tests.rs`; Python `import *` — ноль), `just candidate lint` → 0 находок.

## Blockers

Нет.

## Next action

1. `git rm` живых `Ledger/Backlog/040.md` и `Ledger/Plan/049.md`; стейджить всю архивную ротацию; коммит
   через гейт (memory-check должен быть зелёным после ротации).
2. Финальный D051-пин: stable.txt = полный 40-символьный хеш gate-принятой головы архивного коммита +
   `just bootstrap` (один scratch, откат при сбое); коммит пина.
3. `just feature-merge` (ветку оставить, НЕ пушить). Отчёт по C10 на русском на merge-границе.
4. Затем C7 (анти-дублирование): завести карточку B041, промоутнуть в P050 (бланкет-approve действует).
