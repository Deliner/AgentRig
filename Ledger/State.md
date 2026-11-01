# State

## Focus

**B044 (Tier B) → [P053](Plan/053.md). AC2 `disabled-raii` — VAC-B1 доставлен (инертный источник),
мид-ветка бутстрап выполнен; далее VAC-B2 (активация).**
Заземлённая форма Tier B: AC2 `disabled-raii` — ПРАВИЛО (near-zero-FP); AC5 `sibling-cycle` — расширение
`directory-architecture`; AC1 `unchecked-index`, AC3 TOCTOU, AC4 lossy-int-cast — НОРМЫ (нужны вывод типов/
поток данных, семейство [D059](Decisions/059.md); бланкет-индексация ловит непаникующую `serde_json::Value`).

## Workspace

Ветка `feature/tier-b-rules` (база master @ `4ce7316`). `stable.txt` = `515e585…` (голова VAC-B1,
забутстраплено — stable-бинарь десериализует `disabled-raii`). macOS; коммиты — `just scratch` с
восстановленным PATH (`cargo`/`uv`), иначе гейт падает на `build: cargo not found`.

## Progress

VAC-A `d10996d` (ротация P052/B043 в Archive, выбор P053). VAC-B1 `515e585` (полный гейт зелёный): вид
`DisabledRaii` в `kind.rs`; дескриптор `disabled-raii` + `// DECISION: D092` в `descriptor.rs`; детектор
`disabled_raii`/`raii_marker` на диспетче `call_expression` в `rust.rs` (методы `keep`/`into_path` по имени;
пути `mem::forget`/`ManuallyDrop::new`/`Box::leak` по точной паре сегментов); тест
`rust_flags_disabled_raii_outside_tests` в `markers.rs`; решение D092 + `Decisions/092.md`. Инертно: НЕ в
`registry`/`catalog::ALL`/`lint.yaml`. Мид-ветка пин (этот коммит): `stable.txt` = `515e585` + `just bootstrap`.

## Verification

Гейт VAC-B1 зелёный (build/lint/memory/rustfmt/clippy/rust-test/ruff/mypy/test вкл. delivery/typos/vulture).
`cargo test --lib disabled_raii` — находки [2,3,4,5,6,7]. Мид-пин проходит гейт на stable=`515e585` (lint.yaml
ещё не ссылается на `disabled-raii`, kind инертен).

## Blockers

Нет.

## Next action

1. VAC-B2: активировать `disabled-raii` — `registry` rust.rules, `catalog::ALL`, `lint.yaml` (error, только
   Rust, `exclude` tests, `# DECISION: D092`), навык `own-the-resource` (SKILL.md + leaf `architecture.yaml` +
   плоский симлинк + `assets.rs` include_str!), расширить `references/architecture.yaml`, I020 +1/+1; дополнить
   строку D092 ссылками `[repository rule]`+`[skill]`; догфуд — обосновать `command_runner/jobs/storage.rs`
   `.keep()` `// DECISION: D092`. Полный гейт.
2. AC5 `sibling-cycle`: проверить анализатор графа (graph.rs/analysis.rs/features.rs/resolution) на файловые
   рёбра внутри каталога; если возможно — расширение `directory-architecture` (без нового вида/навыка) + догфуд;
   иначе — норма. Решение D093+.
3. Нормы AC1/AC3/AC4: решения D093+ (ограничение движка D059, бар near-zero-FP D035, покрытие clippy
   restriction-линтами).
4. D051 финальный пин (голова, принятая гейтом) + `just bootstrap` одной командой, коммит до
   `just feature-merge` (сохранить ветку, не пушить). Свободные ID: D093+, P054+.
