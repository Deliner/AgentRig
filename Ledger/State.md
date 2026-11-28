# State

## Focus

P204 (B208), VAC 3: init creates the North Star and the glossary with one header, protects the North Star and bounds
both in lint; the rig gains its glossary, the edit-glossary skill and the same lint bounds (AC2, AC4–AC7 parts).

## Workspace

Branch: feature/north-star-glossary
Revision: 23692d44d435bf33b85185b2ea4437dbacfd6ff1

## Progress

- VAC 1 (07ac675) gives both files in full at every session start; VAC 2 (23692d4) makes the memory check refuse a
  malformed glossary entry, a missing header or a repeated word.
- This VAC: init writes Ledger/NorthStar.md and Ledger/Glossary.md as one header line, lists the North Star in
  hooks protected and ships north-star and glossary size and width rules (30 and 100 nonblank lines, 120 characters)
  repaired by propose-protected-change and edit-glossary. bundle/lint.rs builds the State, North Star and glossary
  bounds in one function. The rig gains Ledger/Glossary.md, the edit-glossary skill, its link and inventory, and the
  same four rules in .agentrig/lint.yaml.
- The D196 hook test clears protection before its Ledger tree write, which the protected North Star now refuses.

## Verification

- `just run test`: 644 passed, 0 failed, on this content.
- `just run rustfmt`, `just lint` and `just lint-config-check` pass; `just candidate memory-check` passes with the
  known D112, D175 and D231 warnings.
- New tests: defaults::shipped_north_star_and_glossary_are_headed_and_bounded (project_setup) and
  approval::the_north_star_of_an_initialized_project_commits_only_after_the_user_approves_it (change_delivery).

## Blockers

- The rig North Star, its protection and the glossary edit route (AC5) change protected .agentrig/hooks.yaml and need
  the user's `! just approve`. The proposal waits in the session scratchpad as b208-proposal: the rig hooks.yaml with
  Ledger/NorthStar.md in protected and a last route for Ledger/Glossary.md (edit-glossary, edit and shell), and
  Ledger/NorthStar.md holding `# North Star` and the first AGENTS.md paragraph verbatim. .cache/runtime/proposal/
  holds the B206 lint.yaml proposal, so this one enters it only after that approval.
- The pin of this Project change needs the user's `! just approve` or consent.

## Next action

Commit this VAC, then pause P204 in Plan with its Delivery section and the morning steps, then take P205 (B209).
