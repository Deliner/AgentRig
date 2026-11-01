# State

## Focus

**Захват Backlog-находок межпроектного обзора.** P054 (B045) завершён и влит в master (`9b5fedd`).
Пользователь попросил завести карточки по результатам межпроектного обзора Rust-кода
(`rust-lint-mining` `wf_f2a6f63a-7ff`, репозитории bevy/rustdesk/openidea): 5 правил-кандидатов и
3 self-находки собственного кода. Заведены как Backlog **B046–B051** (не выбраны в Plan). Plan пуст.

## Workspace

Ветка `feature/review-findings-backlog` (база master `9b5fedd`), наблюдаемый HEAD до
capture-VAC — `9b5fedd`. Коммит — memory-only (только `Ledger/`); новых `kind:` не вводит, мид-бранч
пин не требуется. macOS; коммиты/cargo — через `just scratch` с восстановленным PATH
(`cargo`/`uv`), иначе гейт падает на `build: cargo not found`.

## Progress

Созданы карточки и строки индекса:
- **B046** unsafe-site-justification (P1, medium): расширить D065 на `unsafe impl`/`unsafe trait`,
  `static mut`, тела макросов.
- **B047** discarded-result-variants (P2): расширить D090 на Option-комбинаторы + `.err()`.
- **B048** panic-surface-extension (P3): assert под `cfg(debug_assertions)` → D084;
  `unwrap_err`/`expect_err`/`resume_unwind` → D082; новый `unwind-justification` для `catch_unwind`.
- **B049** rust-silent-error (P4): новый `kind:` — проглоченные ошибки обхода `read_dir`.
- **B050** guard-across-await (P5): новый `kind:` — std-гард через `.await`.
- **B051** self-находки: проза D066/D095 vs код (дрейф обоснования), каст PID `process.rs` vs
  `group.rs`, явное условие `MaybeUninit::zeroed` в `macos.rs`.
Собственный код AgentRig чист по B046–B050 (проверено грепом `Project`); единственные реальные
наблюдения — в B051, все low/без изменения поведения.

## Verification

Карточки — чистый захват Backlog; реализация не начиналась, код не менялся. Capture-VAC —
memory-only, проходит через гейт (exported-index + затронутые тест-группы + smoke). Приёмочные
критерии правил не верифицированы (это будущая работа при доставке).

## Blockers

Нет.

## Next action

Capture-VAC закоммитить memory-only через `.agentrig/hooks/pre-commit`
(`Ledger/Backlog.md`, `Ledger/Backlog/046..051.md`, `Ledger/State.md`), затем `just feature-merge`
на master; ветку сохранить, не пушить. Карточки B046–B051 ждут выбора пользователя в Plan
(ожидаемый порядок доставки правил P1→P5). Любое правило с новым `kind:` (B048 AC3, B049, B050)
при доставке требует двухфазного пина (D051). Custom-allocator (B034) припаркован. Свободные ID:
D096+, P055+, B052+.
