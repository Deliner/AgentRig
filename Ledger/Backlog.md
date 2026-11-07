# Backlog

**Backlog collects future ideas, improvements, observations and hypotheses. Row order and IDs do not express execution priority, deadlines or implementation authorization.**

Use [edit-backlog](../.agents/skills/edit-backlog/SKILL.md) to draft ideas and requirement changes in chat. Save the shown version only after explicit agreement. Preserve context, distinguish observations from hypotheses and check for duplicates, then resume the current task. Do not investigate merely to fill a card or interrupt autonomous execution to seek approval for an unrelated idea.

Rows link stable BNNN IDs to Backlog/NNN.md. Cards retain Context, Proposal and Expected benefit; edit-backlog defines the user-facing content, scenario-based acceptance and agreement process. Ideas may stay brief. Every new product outcome passes through Backlog before the current [Plan](Plan.md), after behavior and acceptance are agreed, consequential questions resolved, and execution and order selected. Plan references the same contract and its source version. After completion, the row and card move to [Archive/Backlog.md](Archive/Backlog.md) with the Plan card. Work required by current acceptance stays in the current task.

Earlier requirements and related ideas are context for later selection, not a delivery queue. Reassess actual prerequisites and refine selected cards in chat; historical saved requirements are not automatic agreement for autonomous delivery.

| ID | Idea | Expected benefit |
| --- | --- | --- |
| [B050](Backlog/050.md) | guard-across-await: std-гард блокировки, удерживаемый через `.await` | Потребитель-async не удержит гард std-блокировки через точку ожидания (дедлок/`!Send`); на нашем коде находок нет. |
| [B064](Backlog/064.md) | ratchet-baseline: внедрение строгих правил в проект с историей через зафиксированный baseline находок | Строгие правила включаются без выключения гейта (идея, не выбрана). |
| [B068](Backlog/068.md) | ord-eq-mismatch: ручной `Ord`/`PartialOrd` читает не те поля, что `PartialEq` (идея, условие срабатывания открыто) | Нарушение контракта std между порядком и равенством, невидимое компилятору и clippy, ловится на любом Rust-коде; намеренное расхождение требует обоснования. |
| [B069](Backlog/069.md) | Форк tree-sitter-rust в составе AgentRig: `safe fn` в `unsafe extern`, `str![…]`, `dyn 'static + Trait` разбираются; решение D112 о причинах | 85 % файлов корпуса, которые компилируются, но не разбираются, возвращаются под правила; причина форка и правило обновления записаны. |
| [B070](Backlog/070.md) | blocking-in-async: `thread::sleep` или `block_on` внутри `async fn` / `async`-блока (идея, не измерена) | Блокировка потока исполнителя и паника вложенного `block_on` ловятся до рантайма; переносимо, на нашем синхронном коде находок нет. |
| [B071](Backlog/071.md) | Длинное логическое выражение: больше трёх частей, соединённых логическими «и» и «или», в `if`, `while`, охранных условиях `match`, `let` (идея, не измерена) | Каждое условие читается с одного взгляда; обход named-if через имя над длинным выражением закрыт. |
| [B072](Backlog/072.md) | Исходящие связи файла: не больше заданного числа внутренних модулей на файл, точка входа исключена (идея, не измерена) | Файл-свалка видна по списку `use`. |
| [B073](Backlog/073.md) | Большой текст внутри кода: многострочный литерал SQL/JSON/HTML вместо отдельного файла через `include_str!` (идея, не измерена) | Данные лежат отдельным файлом, код читается как код. |
