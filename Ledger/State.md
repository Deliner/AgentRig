# State

## Focus

**B044 (Tier B) выбран → [P053](Plan/053.md); VAC-A (планирование) в рабочем дереве, не закоммичен.**
B043 (Tier A) полностью доставлен и влит в master (merge `4ce7316`). Текущая работа — пять пунктов Tier B:
AC1 `unchecked-index`, AC2 `disabled-RAII` (новые виды правил), AC5 `sibling-cycle` (расширение
`directory-architecture`), AC3 TOCTOU и AC4 lossy-int-cast (правило-vs-норма при реализации).

## Workspace

Ветка `feature/tier-b-rules` (база master @ `4ce7316`). Revision HEAD = `4ce7316` (до коммита VAC-A;
рабочее дерево содержит незакоммиченную ротацию P052/B043 + выбор P053 + этот снапшот). `stable.txt` =
`b0ce9eebdbcee5992eec3079079de0fece04703d` (пин B043; Project на этой ветке ещё не менялся, D051-пин
придёт с первым изменением Project). macOS; коммиты — `just scratch` с восстановленным PATH
(`cargo`/`uv`), иначе гейт падает на `build: cargo not found`.

## Progress

VAC-A (ротация + выбор), в рабочем дереве: P052 перенесён в `Archive/Plan.md` + `Archive/Plan/052.md`
(добавлена секция Delivery); B043 — в `Archive/Backlog.md` + `Archive/Backlog/043.md` (изменены только
относительные ссылки `../Decisions/` → `../../Decisions/`); строки P052/B043 убраны из `Plan.md`/`Backlog.md`,
живые карточки `Plan/052.md`/`Backlog/043.md` подлежат удалению в коммите; ссылки card в D088–D090 переведены
на `Archive/Backlog/043.md`; B044 промоутнут в `Plan.md` строкой P053 + карточка `Plan/053.md` + запись в
журнал; B044 остаётся в живом `Backlog.md` (исполняется). Один активный пункт Plan — P053.

## Verification

Гейт для VAC-A ещё не прогонялся; memory-check (индекс Decisions ↔ маркеры, согласованность ссылок)
пройдёт в составе коммита. Последний зелёный полный гейт — на B043 (`b0ce9ee`/`62ea144`).

## Blockers

Нет.

## Next action

1. Закоммитить VAC-A через `just scratch` с PATH (в коммите: удалить живые `Plan/052.md`/`Backlog/043.md`,
   `git add -A Ledger/`, `git commit -F`); гейт прогонит memory-check.
2. Затем VAC-B1 — инертный источник для `unchecked-index` + `disabled-RAII` (enum-виды, дескрипторы,
   детекторы в `rust.rs`, тесты-маркеры; не подключать в `catalog`/`registry`/`lint.yaml`), затем мид-ветка
   бутстрап (stable = голова VAC-B1) → VAC-B2 активация + догфуд. AC5 `sibling-cycle` — отдельным VAC
   расширением `directory-architecture` (без нового вида/навыка). AC3/AC4 — оценить правило-vs-норма
   (решения D092+). D051 финальный пин, затем `just feature-merge` (ветку сохранить, не пушить).
   Свободные ID: D092+, P054+.
