# State

## Focus

**B043 (Tier A) → [P052](Plan/052.md): три новых правила `init`** из отчёта воркфлоу-ревью Rust-кода.
Двухфазная доставка. **VAC-B1 (source) реализован**: виды `tuple-as-record`/`eager-fallback`/`discarded-result`
в `kind.rs`/`descriptor.rs`, детекторы в `rust.rs`, markers-тесты, решения D088–D090. Правила пока инертны
(не в `catalog::ALL`, не в `lint.yaml`) → находок ноль, I020 сбалансирован. Затем B044.

## Workspace

Ветка `feature/tier-a-rules` (база master @ `e9133a7`, до VAC-B1 HEAD = `a910ecb`). macOS. Сборка/коммит/git —
через `just scratch -- '...'` с восстановленным PATH (`cargo` в `~/.cargo/bin`); поиск — `just read -- grep`.
`stable.txt` = `2f490ac…`. Новые виды правил читаются stable-бинарём из `lint.yaml`, поэтому порядок:
VAC-B1 (source, инертно) → **мид-ветка: пин `stable.txt`=голова B1 + `just bootstrap`** (иначе `lint.yaml`
с новым видом падает closed у старого stable) → VAC-B2 (по правилу: `catalog::ALL`+`registry`+live-правило
в `lint.yaml`+навык+догфуд).

## Progress

VAC-A (промоут B043→P052) закоммичен `a910ecb`. VAC-B1 реализован (не закоммичен): kind.rs (+3 варианта),
descriptor.rs (+3 дескриптора), rust.rs (детекторы `record_signature`/`eager_fallback`+`propagates`/
`discarded_result` + диспетчеризация), markers.rs (+3 теста), D088–D090 (детали + index-строки, владельцы
пока card/rule-kind/Rust/tests). `registry.rs`/`catalog::ALL`/`lint.yaml`/навыки — в VAC-B2.

## Verification

`cargo test --lib languages::tests::markers` → 11 passed (3 новых теста: tuple [1,2], eager [2], discarded [2]).
Кандидат компилируется чисто. Гейт по VAC-B1 ещё не прогонялся. Догфуд-цели (чинить в B2): `discarded-result`
— `jobs/process.rs:101-102` (`.ok();`); `eager-fallback` — `command_line/main.rs:58`, `structural_lint/cli.rs:49`,
`jobs/mod.rs:263`; `tuple-as-record` — `sandbox.rs:12`, `gate/mod.rs:164/183` (+перечислить candidate lint-ом).

## Blockers

Нет.

## Next action

1. Закоммитить VAC-B1 (4 исходника + D088–090 + State) через гейт на feature/tier-a-rules.
2. Мид-ветка: `stable.txt`=голова B1 + `just bootstrap` (атомарно, откат при сбое).
3. VAC-B2 по одному правилу (tuple → eager → discarded), каждое атомарно: `catalog::ALL`+=вид,
   `registry` rust.rules+=вид, live-правило в `lint.yaml` (стиль panic-path: Rust-only, exclude tests,
   level error, `# DECISION`), навык (`name-the-record`/`eager-to-lazy`/`handle-discarded`: канон в
   `references/`, flat-symlink, `assets.rs`), расширить index-строку решения ссылками repository-rule+skill,
   починить догфуд-цели этого правила. I020 растёт по 1/1.
4. Финальный D051-пин (`stable.txt`=голова B2) + `just feature-merge` (ветку сохранить, не пушить). Затем B044.
   Свободные ID: D091+, P053+.
