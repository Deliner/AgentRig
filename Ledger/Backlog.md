# Backlog

**Backlog collects future ideas, improvements, observations and hypotheses. Row order and IDs do not express execution priority, deadlines or implementation authorization.**

Use [edit-backlog](../.agents/skills/edit-backlog/SKILL.md) to draft ideas and requirement changes in chat. Save the shown version only after explicit agreement. Preserve context, distinguish observations from hypotheses and check for duplicates, then resume the current task. Do not investigate merely to fill a card or interrupt autonomous execution to seek approval for an unrelated idea.

Rows link stable BNNN IDs to Backlog/NNN.md. Cards retain Context, Proposal and Expected benefit; edit-backlog defines the user-facing content, scenario-based acceptance and agreement process. Ideas may stay brief. Every new product outcome passes through Backlog before the current [Plan](Plan.md), after behavior and acceptance are agreed, consequential questions resolved, and execution and order selected. Plan references the same contract and its source version. After completion, the row and card move to [Archive/Backlog.md](Archive/Backlog.md) with the Plan card. Work required by current acceptance stays in the current task.

Earlier requirements and related ideas are context for later selection, not a delivery queue. Reassess actual prerequisites and refine selected cards in chat; historical saved requirements are not automatic agreement for autonomous delivery.

| ID | Idea | Expected benefit |
| --- | --- | --- |
| [B085](Backlog/085.md) | `config-check` проверяет карту affected каждой проверки только на файлах из её include | Проекты с отдельными наборами тестов под разные фичи проходят `config-check` честной картой, без целей, которые ничего не выбирают. |
