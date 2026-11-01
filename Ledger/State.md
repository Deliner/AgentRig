# State

## Focus

**B043 (Tier A) ПОЛНОСТЬЮ доставлен → [P052](Plan/052.md).** Три правила (`tuple-as-record`/`eager-fallback`/
`discarded-result`) активны в `catalog`+`registry`+`lint.yaml`+навыки, репозиторий догфуд-чист, D051-пин сделан.
Ветка готова к `just feature-merge`. Далее — B044 (Tier B).

## Workspace

Ветка `feature/tier-a-rules` (база master @ `e9133a7`). Revision HEAD = `62ea144` (D051-пин). Коммиты ветки:
`a910ecb` VAC-A, `76874a1` VAC-B1 (инертный источник), `b0ce9ee` VAC-B2 (активация+догфуд), `62ea144` пин.
`stable.txt` = `b0ce9eebdbcee5992eec3079079de0fece04703d` (голова VAC-B2, последний коммит, меняющий Project),
установлен через `just bootstrap` в `.cache/development/b0ce9ee…`. Рабочее дерево чистое. macOS; коммиты —
`just scratch` с восстановленным PATH (`cargo`/`uv`), иначе гейт падает на `build: cargo not found`.

## Progress

VAC-A `a910ecb`; VAC-B1 `76874a1` (kind/descriptor/rust/markers + D088–090, инертно). **VAC-B2 `b0ce9ee`:**
`catalog::ALL`+=3, `registry` rust.rules+=3, три live-правила в `lint.yaml` (Rust-only, tests excluded, error,
`# DECISION: D088/D089/D090`), навыки `name-the-record`/`eager-to-lazy`/`handle-discarded` (SKILL.md + leaf
`architecture.yaml` + flat-symlink + `assets.rs` include_str!), I020 +3/+3; строки D088–090 дополнены
`[repository rule]`+`[skill]`; добавлены D091 + `Decisions/091.md`. Догфуд: 2 `eager-fallback`→`match`
(`main.rs`/`cli.rs`); 10 кортежей→структуры (`Selected`/`CheckRun`/`Outcome`+`change_index`; `Sandboxing`;
`Arrangement`; `Invocation`; `Loaded`; `Dependencies`; `Analysis`; `TokenPath`); 2 teardown-`.ok()` с `DECISION: D091`;
`tests.rs` переведён на `Loaded`. Пин `62ea144`.

## Verification

**Полный гейт зелёный дважды** (на `b0ce9ee` и `62ea144`): build, lint, memory, rustfmt, clippy, rust-test
(106 passed), ruff-format, ruff, mypy, test (358 Python passed, включая `test_dogfooding_capabilities_stay_universal`
I020), typos, vulture. `just bootstrap` собрал и установил `b0ce9ee` (`agentrig 0.3.0`).

## Blockers

Нет.

## Next action

1. `just feature-merge` (через `just scratch` с PATH) — интегрировать ветку в master, прогнав тесты всей ветки;
   ветку сохранить, не пушить.
2. Затем B044 (Tier B) той же двухфазной схемой (инертный источник → активация+догфуд → пин) на своей ветке.
   Свободные ID: D092+, P053+.
