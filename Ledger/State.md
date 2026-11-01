# State

## Focus

**Реализация правил из отчёта воркфлоу-ревью Rust-кода** (Набор II C1–C10 завершён). Отчёт (10 ревьюеров
Sonnet, run `wf_b944977a-5c3`) дал Tier A ([B043](Backlog/043.md)) и Tier B ([B044](Backlog/044.md)),
согласованы пользователем на догфуд-исполнение. Идёт **B043 (Tier A) → [P052](Plan/052.md)**: три новых
вида правила (`tuple-as-record`, `eager-fallback-with-?`, `discarded-result`) + починка 2 багов. Затем B044.

## Workspace

Ветка `feature/tier-a-rules` (база master @ `e9133a7`). Идёт VAC-A (промоут B043→P052: `Plan/052.md`,
активная строка в `Plan.md`, проза, State). Машина — macOS. Сборка/коммит/git — через `just scratch -- '...'`
с восстановленным PATH (`cargo` в `~/.cargo/bin`); поиск — `just read -- grep`. `stable.txt` = `2f490ac…`.
**Двухфазная доставка** (новые виды правил читаются stable-бинарём из `lint.yaml`): VAC-B1 (source:
kind.rs/descriptor.rs/детекторы rust.rs/registry.rs/markers-тесты, компилируется в кандидат, НЕ трогает
lint.yaml/catalog) → мид-ветка пин+bootstrap → VAC-B2 (catalog::ALL += 3 вида, lint.yaml += 3 live-правила,
3 навыка в references + flat-symlink + architecture.yaml + assets.rs, D-решения).

## Progress

Отчёт собран, проверен по исходнику, durable-записан в Backlog (B043/B044, коммит `5499e9b`, влит в master).
B043 промоутнут в P052 на ветке feature/tier-a-rules. Реализация правил ещё не начата (идёт VAC-A).

## Verification

Нет (VAC-A не закоммичен). master зелёный. Топ-находки Tier A подтверждены чтением исходника:
`main.rs:58`/`cli.rs:49` (eager `env::current_dir()?`), `process.rs:101-102` (`.ok();`), `git/mod.rs:146`
(`Err(_)=>Ok(false)`), `sandbox.rs:12` `(bool,Capture)`, `gate/mod.rs:164/183` (кортежи-параметры).

## Blockers

Нет.

## Next action

1. Закоммитить VAC-A (промоут B043→P052 + State) через гейт на feature/tier-a-rules.
2. VAC-B1 (source): добавить три вида в `kind.rs`/`descriptor.rs`, детекторы в `rust.rs`
   (tuple-as-record: tuple_type в параметрах/возврате fn вне тестов; eager-fallback: try_expression в
   аргументе unwrap_or/ok_or/map_or; discarded-result: expression_statement `.ok()` + рука `Err(_)` без
   justified), `registry.rs`, markers-тесты, решения D088–D090. Компилируется в кандидат, lint.yaml не трогать.
3. Мид-ветка: пин stable.txt = голова VAC-B1 + `just bootstrap` (атомарно, откат при сбое).
4. VAC-B2 (delivery): `catalog::ALL` += 3 вида, `lint.yaml` += 3 live-правила, навыки
   (`name-the-record`/`eager-to-lazy`/`handle-discarded` или по месту) в references + flat-symlink +
   architecture.yaml + `assets.rs`; Decisions index-строки.
5. Догфуд: привести код к зелёному + починить 2 бага (env::current_dir в main.rs/cli.rs; `.ok();`/`Err(_)`
   в process.rs/git.rs). Финальный D051-пин. `just feature-merge`. Затем B044. Свободные ID: D088+, P053+.
