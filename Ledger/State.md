# State

## Focus

P204 (B208) resumes after the user's approval. This VAC commits the approved rig hooks.yaml and North Star, gives a
project after init the same glossary route, moves the purpose paragraph out of AGENTS.md and CLAUDE.md and records
D241, which supersedes the placement of D234.

## Workspace

Branch: feature/north-star-glossary
Revision: 83527bb3808980c17f1a94d2d6757688881b6d33

## Progress

- The user's `! just approve` applied .agentrig/hooks.yaml (Ledger/NorthStar.md protected, the Ledger/Glossary.md
  route to edit-glossary last) and Ledger/NorthStar.md (# North Star and the AGENTS.md paragraph verbatim).
- bundle/template.rs ends the init routes with the same glossary route; ledger_edit_guidance edits
  Ledger/Glossary.md and expects edit-glossary once.
- AGENTS.md and CLAUDE.md drop the purpose paragraph and stay identical; D241 records the move and D234 links
  NorthStar.md. P204 is active again, and Plan/204.md Delivery describes the resumption.

## Verification

- just run test with agent_hooks::tests::memory and project_setup::initialization::tests::rig passes 9 tests,
  including ledger_edit_guidance (I012) and init_matches_the_rig_setup_except_listed_differences (I023).
- just run rustfmt and just lint report nothing; just check --only memory passes on the working tree.
- The hook of the rig names edit-glossary for a shell write through hooks.routes[10].

## Blockers

None.

## Next action

Commit this VAC, run just propose-pin HEAD under the consent of the user and commit the pin, then archive P204 and
B208 and run just feature-merge.
