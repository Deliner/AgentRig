---
name: edit-backlog
description: >
  Capture concrete improvement ideas in project Backlog. Use proactively when
  work reveals a useful code, architecture or tooling improvement outside the
  current task, or when the user asks to save an idea for later.
---

# Capture an idea without changing the task

When a concrete improvement occurs to you during work, use this skill while its
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

Backlog is not an execution queue. Only promote an idea under an instruction
authorizing planning, using edit-plan and linking the resulting PNNN from its
card. Keep the original context so the idea is not lost after promotion.
