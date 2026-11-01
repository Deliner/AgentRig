# Backlog

**Backlog collects future ideas, improvements, observations and hypotheses. Row order and IDs do not express execution priority, deadlines or implementation authorization.**

Use [edit-backlog](../.agents/skills/edit-backlog/SKILL.md) to draft ideas and requirement changes in chat. Save the shown version only after explicit agreement. Preserve context, distinguish observations from hypotheses and check for duplicates, then resume the current task. Do not investigate merely to fill a card or interrupt autonomous execution to seek approval for an unrelated idea.

Rows link stable BNNN IDs to Backlog/NNN.md. Cards retain Context, Proposal and Expected benefit; edit-backlog defines the user-facing content, scenario-based acceptance and agreement process. Ideas may stay brief. Every new product outcome passes through Backlog before the current [Plan](Plan.md), after behavior and acceptance are agreed, consequential questions resolved, and execution and order selected. Plan references the same contract and its source version. After completion, the row and card move to [Archive/Backlog.md](Archive/Backlog.md) with the Plan card. Work required by current acceptance stays in the current task.

Earlier requirements and related ideas are context for later selection, not a delivery queue. Reassess actual prerequisites and refine selected cards in chat; historical saved requirements are not automatic agreement for autonomous delivery.

| ID | Idea | Expected benefit |
| --- | --- | --- |
| [B043](Backlog/043.md) | Tier A отчёта воркфлоу: tuple-as-record, eager-fallback-with-?, discarded-result (+ починка 2 багов) | Закрывают обход parameter-count/boolean-parameter/silent-error и ловят реальные латентные баги до коммита; переносимо. |
| [B044](Backlog/044.md) | Tier B отчёта воркфлоу: unchecked-index, disabled-RAII, TOCTOU/path-exists, lossy-int-cast, sibling-cycle | Доп. охват панических/ресурсных/файловых/детерминизм-смеллов + внутрикаталожные циклы; переносимо. |
