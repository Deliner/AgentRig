# State

## Focus

**Новый арк: правила из отчёта воркфлоу-ревью Rust-кода.** Набор II (C1–C10) завершён и влит в master.
Воркфлоу-ревью нашего Rust-кода (10 ревьюеров Sonnet, run `wf_b944977a-5c3` / task `we6yjitjd`, 96
находок, кросс-валидация + ручная проверка по исходнику) дал Tier A и Tier B. Пользователь согласовал
**все** пункты обоих тиров на исполнение через догфудинг. Полное содержание тиров durable-записано в
карточках [B043](Backlog/043.md) (Tier A) и [B044](Backlog/044.md) (Tier B) — scratchpad-дамп эфемерен.
Следующее — реализовать B043, затем B044.

## Workspace

master @ `e9133a7`. Ветка `feature/workflow-findings` — коммит durable-записи (карточки B043/B044 +
строки Backlog + State). Машина — macOS. Сборка/коммит/git — через `just scratch -- '...'` с
восстановленным PATH (`cargo` в `~/.cargo/bin`); поиск — `just read -- grep`. `stable.txt` = `2f490ac…`
(голова C7). Новые виды правил → stable-бинарь должен их десериализовать → **двухфазная доставка**
(source → мид-ветка пин+bootstrap → delivery) на каждый новый вид.

## Progress

Отчёт собран и проверен по исходнику; содержание продублировано в Backlog (B043 Tier A: tuple-as-record,
eager-fallback-with-?, discarded-result + 2 бага; B044 Tier B: unchecked-index, disabled-RAII,
TOCTOU/path-exists, lossy-int-cast, sibling-cycle). Tier C (магические числа/stringly-typed/
`serde_json::Value`/лишние clone) отсеян как уже-норма (D073/D059) или высокий FP. Карточки согласованы
пользователем; реализация ещё не начата.

## Verification

Нет (реализация не начата). Текущий master зелёный (merge-гейт C8 PASS). Топ-находки Tier A подтверждены
чтением исходника: `main.rs:58`/`cli.rs:49` (eager `env::current_dir()?`), `process.rs:101-102` (`.ok();`),
`git/mod.rs:146` (`Err(_)=>Ok(false)`), `sandbox.rs:12` `(bool,Capture)`, `gate/mod.rs:164/183` (кортежи).

## Blockers

Нет.

## Next action

1. Закоммитить durable-запись (B043/B044 + строки Backlog + State) и влить `feature/workflow-findings`
   в master (Ledger-only → без D051-пина).
2. Промоутнуть **B043** (Tier A) → P052 (`Plan/052.md`, активная строка в `Plan.md`, проза), ветка
   `feature/tier-a-rules`. Двухфазная доставка трёх новых видов в одной ветке: VAC-B1 (kind/descriptor/
   детекторы rust.rs/registry/markers-тесты) → мид-ветка пин+bootstrap → VAC-B2 (catalog::ALL += 3,
   lint.yaml += 3 live-правила, 3 навыка в references + flat-symlink + assets.rs, D-решения D088+).
3. Догфуд: привести код к зелёному, включая починку 2 багов (env::current_dir в main.rs/cli.rs;
   `.ok();`/`Err(_)` в process.rs/git.rs — обосновать/исправить, проверить направление в `merged_reference`).
   Финальный D051-пин (ветка меняет Project). `just feature-merge`.
4. Затем **B044** (Tier B) той же схемой. Свободные ID: P052+, D088+.
