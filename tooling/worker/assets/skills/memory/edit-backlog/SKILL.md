---
name: edit-backlog
description: >
  Capture future ideas, improvements and hypotheses in project Backlog. Use
  proactively when one occurs outside current work or the user asks to save
  or plan something for later.
---

# Capture an idea without changing the task

When an improvement, observation or hypothesis occurs during work, use this skill while its
context is fresh. Recording an idea in a writable project is authorized without
asking the user to repeat it or approve the note. This does not authorize its
implementation, promotion to Plan, or extra investigation. In a read-only role,
return the idea to the caller instead of claiming to have saved it.

Read `paths.memory` in `agentrig.yaml` and use that directory's `Backlog.md` and
`Backlog/` alongside Plan. Check the Backlog and relevant Plan entries for an
existing idea; add useful context to that entry instead of creating a duplicate.
Do not revive an explicitly rejected idea without new evidence or instruction.

Keep the index compact: `ID | Idea | Expected benefit`. Use a stable next unused
`BNNN` ID linked to `Backlog/NNN.md`; check both index and existing cards and never
renumber other ideas. An empty index is valid. Each card has these sections:

- `Context`: what prompted the idea, affected owner/paths and relevant evidence
  or user request. Distinguish an observation from a hypothesis.
- `Proposal`: the smallest useful description of the improvement, leaving
  implementation choices open when they are not known.
- `Expected benefit`: who benefits and how; include a useful way to evaluate the
  idea if already apparent. A speculative benefit is not a verified result.

Write the card and its index row together and update architecture inventories
where the project uses them. Ordinary file editing does not require `just write`.
Keep notes short and self-contained, with relative links to relevant owners;
do not copy secrets or entire transcripts. If recording fails, report the failure
instead of saying the idea is saved. Use the existing memory check for structure.

Resume the current task after recording. Do not search for more ideas merely to
fill the Backlog. Work required by current acceptance remains in the current
task; saving a note must not defer a necessary fix or conceal a blocker.

Backlog has no execution order or priority; IDs only identify cards. A request to
save or plan an idea for later belongs here. Only promote an idea when explicitly
selected for execution now, using edit-plan and linking the resulting PNNN from
its card. Retain original context; when the Plan entry is completed, replace its
live link with the Archive card. Do not treat a saved hypothesis as a
verified defect or a commitment to implement it.

When current work is explicitly deferred, retain its requirements, actual
progress, blocker, branch and resumption condition here before removing its Plan
entry. Reconcile related Plan dependencies and live links together. A context
reset or interruption alone is not a decision to defer the task.
