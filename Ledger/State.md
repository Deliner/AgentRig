# State

## Focus

P204 (B208), VAC 2 for the entry part of AC2: the memory check refuses a glossary whose first line is not a header,
a later nonblank line that is not `- **word** — meaning` with a word and a meaning, or a word written twice in any
letter case. The sizes and widths of AC1 and AC2 are lint rules of the next VAC.

## Workspace

Branch: feature/north-star-glossary
Revision: 07ac675b98bbc6a7ba6bfba22ddedc93293b2be0

## Progress

- `project_memory/glossary.rs` checks `Glossary.md` of the memory directory after the State sections; a missing
  file is not checked, since init creates it and a session names its absence.
- A refusal names the file and line with the entry form, the first line for a missing header, and both lines with
  the word for a repeat.
- The file name comes from `briefing.rs`, which gives the glossary at session start.
- Four memory-check scenarios cover a valid glossary, four malformed lines, a missing header and a repeat.

## Verification

- `just run test -- project_memory::tests::glossary::`: 4 passed.
- `just run test -- project_memory::`: 50 passed.
- `just run rustfmt` and `just lint` pass.

## Blockers

None for this VAC.

## Next action

Commit, then add the size and width lint rules, the init files, the edit-glossary skill and the rig glossary (VAC 3).
