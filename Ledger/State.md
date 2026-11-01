# State

## Focus

**B043 (Tier A) → [P052](Plan/052.md): три новых правила `init`** из отчёта воркфлоу-ревью Rust-кода.
**VAC-B1 (инертный источник) закоммичен; VAC-B2 (доставка) собран в рабочем дереве, проверен, готов к одному
коммиту.** Три вида (`tuple-as-record`/`eager-fallback`/`discarded-result`) выведены в `catalog::ALL`+`registry`+
`lint.yaml`+навыки, догфуд-находки исправлены. Все три правила доставлены одним связным VAC (единый мид-ветка
bootstrap, единый рост I020 +3/+3). Затем D051-пин, `feature-merge`, потом B044.

## Workspace

Ветка `feature/tier-a-rules` (база master @ `e9133a7`). Revision HEAD = `76874a1` (VAC-B1); VAC-B2 ещё не
закоммичен. macOS; shell — `just scratch -- '...'` с восстановленным PATH; поиск — `just read -- grep`.
**Мид-ветка bootstrap выполнен:** `stable.txt` в рабочем дереве = `76874a16c6c2f2d6812083af85fd85095c547fef`
(установлен в `.cache/development/76874a1…`), в индексе всё ещё `2f490ac…` — намеренная временная локальная
установка, чтобы стабильный бинарь гейта десериализовал `lint.yaml` с новыми видами. Финальный D051-пин
(`stable.txt`=голова B2) + commit — перед `just feature-merge`.

## Progress

VAC-A `a910ecb`; VAC-B1 `76874a1` (kind/descriptor/rust/markers + D088–090, инертно). **VAC-B2 в рабочем дереве:**
`catalog::ALL`+=3, `registry` rust.rules+=3, три live-правила в `lint.yaml` (Rust-only, `exclude` tests, level
error, `# DECISION: D088/D089/D090`), `assets.rs` SKILLS+=3, навыки `references/{name-the-record,eager-to-lazy,
handle-discarded}` (SKILL.md + leaf `architecture.yaml` + flat-symlink), родительский `references/architecture.yaml`
расширен, строки D088–090 дополнены `[repository rule]`+`[skill]`, добавлены строка D091 и `Decisions/091.md`.
**Догфуд-исправления:** 2 `eager-fallback` → `match` (`main.rs`, `cli.rs`); 10 кортежей-сигнатур → именованные
структуры (`Selected`/`CheckRun`/`Outcome`+`change_index` в gate; `Sandboxing` в sandbox+mod; `Arrangement` в
process + 2× `// DECISION: D091` на `.ok()`; `Invocation` в diagnostics; `Loaded` в policy+mod+runner; `Dependencies`
в analysis; `Analysis` в runner; `TokenPath` в macros); `tests.rs` переведён на `Loaded` (11 мест; 2 zone-теста —
bind+field-access, чтобы остаться ≤40 строк).

## Verification

Рабочее дерево чисто по всем шагам гейта (кандидат собран в `.cache/worker/release`): rustfmt `--check` чисто;
**кандидат lint rc=0** (три правила + architecture + function-lines); **clippy** точными флагами гейта
(`+1.98.1 --locked -D warnings -D clippy::panic/unwrap_used/expect_used/…`) rc=0; `cargo test` 106 passed;
delivery-тесты 4 passed, в т.ч. `test_dogfooding_capabilities_stay_universal` (наборы видов/навыков сбалансированы,
I020); `typos` rc=0. Память сверена вручную: `lint.yaml` несёт `# DECISION: D088/D089/D090` под `[repository rule]`,
строки D088–D091 ссылаются на реальных владельцев, wiring-файлы без маркеров и не слинкованы (паттерн D085).
Коммит ещё не сделан — гейт как хук не прогонялся.

## Blockers

Нет.

## Next action

1. Застейджить все файлы VAC-B2 и закоммитить одним VAC через гейт (сработает hard-stop route[7] edit-state →
   прочитать edit-state, повторить тот же коммит).
2. Финальный D051-пин: `stable.txt`=голова B2 + `just bootstrap` + commit пина, затем `just feature-merge`
   (ветку сохранить, не пушить).
3. Затем B044 (Tier B) той же двухфазной схемой на своей ветке. Свободные ID: D092+, P053+.
