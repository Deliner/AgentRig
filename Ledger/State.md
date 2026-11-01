# State

## Focus

**B044 (Tier B) → [P053](Plan/053.md). AC2 `disabled-raii` — ДОСТАВЛЕНО (VAC-B2 активация).**
Заземлённая форма Tier B: AC2 `disabled-raii` — ПРАВИЛО (готово); AC5 `sibling-cycle` — расширение
`directory-architecture` (следующее); AC1 `unchecked-index`, AC3 TOCTOU, AC4 lossy-int-cast — НОРМЫ (нужны вывод
типов/поток данных, семейство [D059](Decisions/059.md); бланкет-индексация ловит непаникующую `serde_json::Value`).

## Workspace

Ветка `feature/tier-b-rules` (база master @ `4ce7316`). `stable.txt` = `515e585…` (голова VAC-B1,
забутстраплено). macOS; коммиты — `just scratch` с восстановленным PATH (`cargo`/`uv`), иначе гейт падает на
`build: cargo not found`.

## Progress

VAC-A `d10996d` (ротация P052/B043 в Archive, выбор P053). VAC-B1 `515e585` (инертный источник: вид
`DisabledRaii`, дескриптор+детектор `disabled_raii`/`raii_marker`, тест, D092). Мид-пин `81ed95c`
(`stable.txt`=`515e585` + bootstrap). VAC-B2 (этот коммит): `disabled-raii` в `registry` rust.rules и
`catalog::ALL`; живое правило в `lint.yaml` (error, только `.rs`, tests excluded, `# DECISION: D092`); навык
`own-the-resource` (SKILL.md + leaf `architecture.yaml` + плоский симлинк + `references/architecture.yaml` +
`assets.rs` include_str!); строка D092 дополнена `[repository rule]`+`[skill]`; догфуд — `.keep()` в
`command_runner/jobs/storage.rs` обоснован `// DECISION: D092`.

## Verification

`just lint-rule disabled-raii` описывает правило (кандидат, полная проводка). `just lint` (кандидат) по репозиторию
— ноль находок (storage.rs обоснован, tests исключены, в `tooling/` сайтов нет). `config-check` зелёный. Полный
гейт VAC-B2 — на коммите (build/lint/memory/rustfmt/clippy/rust-test/ruff/mypy/test вкл. I020/typos/vulture).

## Blockers

Нет.

## Next action

1. AC5 `sibling-cycle`: проверить анализатор графа (graph.rs/analysis.rs/features.rs/resolution) на файловые
   рёбра внутри каталога; если возможно — расширение `directory-architecture` (без нового вида/навыка, без мид-пина)
   + догфуд; иначе — норма. Решение D093+.
2. Нормы AC1/AC3/AC4: решения D093+ (ограничение движка D059, бар near-zero-FP D035, покрытие clippy
   restriction-линтами).
3. D051 финальный пин (голова, принятая гейтом) + `just bootstrap` одной командой, коммит до
   `just feature-merge` (сохранить ветку, не пушить). Свободные ID: D093+, P054+.
