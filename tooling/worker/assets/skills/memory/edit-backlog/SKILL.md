---
name: edit-backlog
description: >
  Draft and agree feature cards in chat, then save the approved version in
  Backlog. Use when capturing an idea, clarifying requirements, revising an
  existing card or preparing features for autonomous Plan execution.
---

# Agree the outcome before saving or executing

Use proactively for concrete ideas outside current work and when the user asks
to prepare or revise a feature. Read the configured Backlog and relevant Plan
entries to avoid duplicates and preserve context. Distinguish historical
observations, current evidence and hypotheses. Do not investigate merely to fill
a card or revive rejected ideas without new evidence or instruction.

## Draft in chat

Keep new cards and proposed requirement changes in chat until the user explicitly
approves saving the shown version. Discussing or preparing a card, praise for an
idea, silence and elapsed time are not approval. An explicit instruction to save
an already shown version is sufficient; no special phrase or second confirmation
is required. Do not save unapproved drafts in State or another file instead.

Show the complete current card in the user's language. After revisions, also show
the substantive difference: previous behavior -> new behavior -> consequence.
Do not reinterpret the approved content while saving. Already authorized
mechanical link, ID and delivery-evidence updates do not reopen agreement on
unchanged requirements.

A future idea may stay short. Elaborate only cards selected for discussion or
upcoming execution. Offer unrelated ideas without stopping authorized work to
seek approval; keep an unapproved draft in chat and continue the current task.

## Draft the report — approved B

The user approved B from the A/B/C comparison: mutation 3, outcome
distinguishability. The following instructions govern drafting in chat.
The later save and handoff sections govern authorized file changes.

Построй рапорт вокруг различий, существенных для приёмки. Читатель должен
понимать не только желаемое свойство, но и почему похожего результата
может быть недостаточно.

Сначала установи, какое изменение требуется и какие исходные ограничения
нельзя потерять. Для каждого результата мысленно сопоставь:
— требование выполнено;
— внешне похожее состояние, в котором нарушено именно это требование.
Выводи лишь такие различия, которые следуют из исходника. Не придумывай
сценарии отказа и новые гарантии ради выразительного контраста.

Каждый основной пункт рапорта содержит требуемое состояние или поведение
и проверяемый признак его достижения. Если исходник уже содержит приёмку,
сохрани её смысл и идентификаторы. Пример допустим только как заменяемая
иллюстрация существующего различия, без новых правил.

Различай эффект и способ: выполнение предложенного действия не доказывает
достижения эффекта. Средство становится условием приёмки только при прямом
основании. Не выбирай внутреннее устройство за исполнителя и не превращай
его работу в пользовательскую функцию.

Дай название и коротко изложи смысл изменения. Затем изложи основные
пункты как прямые обязательства; общие границы и существенное неизвестное
помести в конце. Не выдавай анализ альтернатив и ход рассуждения.

Проверь, что контрасты не усилили исходный запрос, частные примеры не сузили
общее требование, а наблюдения прошлого не стали новыми обязательствами.
Если желаемую границу определить нельзя, назови именно этот пробел.
Отсутствующие сведения о текущем состоянии отдели от решения пользователя.

Выведи только законченный рапорт на русском. Не более 1000 слов во всём итоговом сообщении, включая заголовки и таблицы; счёт по пробельным символам. Реализацию не начинай, файлы не изменяй.

## Repository card envelope

Keep Context, Proposal and Expected benefit in the shown card so saving does
not require rewriting its meaning or changing the memory schema. Context retains
the user's request and applicable basis; Proposal contains the report's
obligations, acceptance, boundaries and material unknowns; Expected benefit
states the intended benefit. Preserve existing criterion IDs. The former
mandatory scenario-table and journey template is replaced by the approved
report instructions above.

## Save the agreed version

Read paths.memory in agentrig.yaml. Maintain Backlog.md with
ID | Idea | Expected benefit rows and Backlog/NNN.md cards. Allocate a stable
unused BNNN from index and cards; never renumber other ideas. Preserve an existing
card's identity and agreed context. Save the approved body with only necessary
ID/navigation metadata, update index and applicable architecture inventories
together, and use the existing memory check. Ordinary editing does not require
just write. Do not copy secrets or full transcripts. Report failed saves.

Backlog has no execution priority. Saving is not implementation authorization.
Existing saved ideas are not automatically agreed for delivery under this
process; refine selected cards in chat before changing their contract.
This is a skill-guided policy, not a new approval registry or proof of
machine-verified understanding.

## Hand off and check the same contract

All new product outcomes pass through Backlog before Plan. Promote only cards
with agreed behavior and acceptance, resolved behavior-changing questions, and
execution and order selected by the user. Use edit-plan and execute-plan-feature.
One instruction to execute a selected set authorizes the whole set; do not ask
again before each card. Plan still covers current or resumed work, and Backlog
is not an automatic queue.

Plan references the agreed Backlog contract rather than maintaining a second
independently rewritten specification. Preserve wording and criterion IDs; use
existing Git history to identify the agreed version for delivery and review.
Saving a card or changing its status does not establish user agreement.
Do not change acceptance to fit the implementation.

For acceptance by the executor, a subagent or a reviewer, provide the agreed
card version, implementation/artifacts and means to exercise the scenarios.
Use delegation or review only under existing authorization and workflow.
Report each criterion as met, violated or not verified, with concrete evidence.
Not verified is not success. Passing implementation-authored tests alone does
not establish all criteria. Also inspect new rejection conditions, transformations,
formats and side effects against agreed scope. The verifier follows the same
contract and applicable project rules without inventing feature requirements.
Do not claim that a review covered criteria outside its scope.

Keep required work in the current task. For explicit deferral, preserve its
requirements, progress, blocker, branch and resume condition before removing it
from Plan, and reconcile dependencies. Preserve source context and delivery links;
replace live Plan links with Archive links on completion. Interruption alone is
not deferral. Follow edit-plan for automatic completed-card archival.
