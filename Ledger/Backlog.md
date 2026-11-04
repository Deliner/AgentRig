# Backlog

**Backlog collects future ideas, improvements, observations and hypotheses. Row order and IDs do not express execution priority, deadlines or implementation authorization.**

Use [edit-backlog](../.agents/skills/edit-backlog/SKILL.md) to draft ideas and requirement changes in chat. Save the shown version only after explicit agreement. Preserve context, distinguish observations from hypotheses and check for duplicates, then resume the current task. Do not investigate merely to fill a card or interrupt autonomous execution to seek approval for an unrelated idea.

Rows link stable BNNN IDs to Backlog/NNN.md. Cards retain Context, Proposal and Expected benefit; edit-backlog defines the user-facing content, scenario-based acceptance and agreement process. Ideas may stay brief. Every new product outcome passes through Backlog before the current [Plan](Plan.md), after behavior and acceptance are agreed, consequential questions resolved, and execution and order selected. Plan references the same contract and its source version. After completion, the row and card move to [Archive/Backlog.md](Archive/Backlog.md) with the Plan card. Work required by current acceptance stays in the current task.

Earlier requirements and related ideas are context for later selection, not a delivery queue. Reassess actual prerequisites and refine selected cards in chat; historical saved requirements are not automatic agreement for autonomous delivery.

| ID | Idea | Expected benefit |
| --- | --- | --- |
| [B047](Backlog/047.md) | discarded-result-variants: молча отброшенные Option-комбинаторы (`map`/`and_then`/`map_or`) + `.err()` | Отброшенные fallible-результаты Option ловятся из коробки в дополнение к `let _ =`/`.ok()`. |
| [B048](Backlog/048.md) | panic-surface-extension: assert под `cfg(debug_assertions)`, `unwrap_err`/`expect_err`/`resume_unwind`, обоснование `catch_unwind` | Debug-only проверки не выдают себя за release-контракт; паник-/unwind-поверхность покрыта единообразно. |
| [B050](Backlog/050.md) | guard-across-await: std-гард блокировки, удерживаемый через `.await` | Потребитель-async не удержит гард std-блокировки через точку ожидания (дедлок/`!Send`); на нашем коде находок нет. |
| [B051](Backlog/051.md) | self-находки обзора: проза D066/D095 vs код, согласованность каста PID, явное условие `MaybeUninit::zeroed` | Собственный код соответствует предлагаемой дисциплине; обоснования не переобещают. |
| [B060](Backlog/060.md) | C-профиль dev-гейта: `clippy::exit`, `should_panic_without_expect`, `missing_fields_in_debug`, `rest_pat_in_fully_bound_structs`, `self_named_module_files`; `#![deny(unsafe_code)]` с единственным обоснованным allow | Dev-гейт ловит пять новых классов дефектов; `unsafe` не появляется вне объявленного поддерева. |
| [B061](Backlog/061.md) | владелец консольного вывода (73 сайта `print_*`) + `clippy::print_stdout`/`print_stderr` как защита владельца | Источник любой строки вывода — одно место; формат меняется одним изменением. |
| [B062](Backlog/062.md) | одноразовые ремонты нашего кода: fail-closed `merged_reference`/guard, enum вместо строк-режимов, именованные пары, `Ordering` с обоснованием, usage → `--help` | Собственный код соответствует нормам N1/N2; fail-open вердикты закрыты тестами. |
| [B063](Backlog/063.md) | нормы N1 `type-driven-discipline` и N2 `fail-closed-enforcement` + решение-вердикт дуги (Tier 2, покрытое, снятое) | Потребитель получает две переносимые дисциплины; отклонённые кандидаты не предлагаются заново без новых данных. |
| [B064](Backlog/064.md) | ratchet-baseline: внедрение строгих правил в проект с историей через зафиксированный baseline находок | Строгие правила включаются без выключения гейта (идея, не выбрана). |
