# State

## Focus

P204 (B208), VAC 1 for AC3: when the agent's context starts anew, the session start text gives the full North Star
and glossary before the State and Plan reminder, and names a file it lacks while the session still starts.

## Workspace

Branch: feature/north-star-glossary
Revision: 5fcd9430d1e949d1575e3403c208924ca3197ef9

## Progress

- `project_memory/briefing.rs` reads `NorthStar.md` and `Glossary.md` of the memory directory and gives each in full,
  or a line that names it as missing or unreadable.
- The agent hook puts that text first in its SessionStart context; the reminder of State and Plan and the
  complexity refresh stay after it.
- Three hook tests: startup, clear and compact each carry both texts; a missing glossary and an unreadable North
  Star are named and the session still starts.

## Verification

- `just run test -- agent_hooks::tests::memory::`: 8 passed, with the 3 new tests.
- `just run test -- agent_hooks:: project_setup::tests::doctor shared::testing::tests::launchers`: 50 passed.
- `just run rustfmt` and `just lint` pass.

## Blockers

None for this VAC. The installed runtime gives the files only after a pin of this code.

## Next action

Commit, then check the glossary entries and repeated words in the memory check (AC2) as the next VAC.
