# State

## Focus

**B044 (Tier B) → [P053](Plan/053.md). AC2 `disabled-raii` — VAC-B1 (инертный источник).**
Заземлённая форма Tier B: AC2 `disabled-raii` — ПРАВИЛО (near-zero-FP), AC5 `sibling-cycle` — расширение
`directory-architecture`; AC1 `unchecked-index`, AC3 TOCTOU, AC4 lossy-int-cast — НОРМЫ (нужны вывод типов/
поток данных, семейство [D059](Decisions/059.md); бланкет-индексация ловит непаникующую `serde_json::Value`).

## Workspace

Ветка `feature/tier-b-rules` (база master @ `4ce7316`). `stable.txt` = `b0ce9ee…` (пин B043). macOS; коммиты —
`just scratch` с восстановленным PATH (`cargo`/`uv`), иначе гейт падает на `build: cargo not found`.
Двухфазная доставка AC2: после коммита VAC-B1 — мид-ветка `just bootstrap` на голову VAC-B1 (чтобы stable-бинарь
десериализовал новый вид), затем VAC-B2 с живым правилом в `lint.yaml`.

## Progress

VAC-A `d10996d` (ротация P052/B043 в Archive, выбор P053). VAC-B1 (в рабочем дереве, тест зелёный, не
закоммичен): вид `DisabledRaii` в `kind.rs`; дескриптор `disabled-raii` + `// DECISION: D092` в `descriptor.rs`;
детектор `disabled_raii`/`raii_marker` на диспетче `call_expression` в `rust.rs` (методы `keep`/`into_path` по
имени, пути `mem::forget`/`ManuallyDrop::new`/`Box::leak` по точной паре сегментов); тест
`rust_flags_disabled_raii_outside_tests` в `markers.rs`; решение D092 + `Decisions/092.md`. Инертно: НЕ в
`registry`/`catalog::ALL`/`lint.yaml` — stable-бинарь его не видит. Навык `own-the-resource` и живое правило —
в VAC-B2.

## Verification

`cargo test --lib disabled_raii` зелёный (находки [2,3,4,5,6,7]; justified/не-совпадающие методы/строка/тест
исключены). Полный гейт VAC-B1 — на коммите.

## Blockers

Нет.

## Next action

1. Закоммитить VAC-B1 через `just scratch`+PATH (гейт: build/lint/memory/clippy/rustfmt/test).
2. Мид-ветка бутстрап: `stable.txt` = голова VAC-B1 + `just bootstrap` в одной команде.
3. VAC-B2: активировать `disabled-raii` (`registry` rust.rules, `catalog::ALL`, `lint.yaml` error/Rust-only/
   tests excluded `# DECISION: D092`, навык `own-the-resource` SKILL.md+leaf+symlink+`assets.rs`), I020 +1/+1;
   дополнить строку D092 ссылками `[repository rule]`+`[skill]`; догфуд — обосновать `storage.rs:14 .keep()`
   `// DECISION: D092`.
4. Затем AC5 `sibling-cycle` (расширение `directory-architecture`, проверить анализатор графа на файловые рёбра
   внутри каталога) и нормы AC1/AC3/AC4 (решения D093+). D051 финальный пин, `just feature-merge` (сохранить, не
   пушить). Свободные ID: D093+, P054+.
