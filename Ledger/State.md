# State

## Focus

P205 (B209), VAC C: the memory check refuses malformed cards named by the queue key in Backlog/, Plan/ and Archive/
(AC2-AC5) beside the old indexes. Entries without the key prefix stay with the old index checks until the last code
VAC; a project without a key has no card to check yet.

## Workspace

Branch: feature/card-queue
Revision: d8076488ea3bbe8417712320ec864a8af16bee33

## Progress

- Project/project_memory/cards.rs checks each entry named `{KEY}-...`: a file `{KEY}-N.md` or a directory `{KEY}-N`
  holding `{KEY}-N.md` and only assets, results and scenarios.md, N without a leading zero; first line
  `# {KEY}-N — name`; the Backlog card sections; at most one `After: {KEY}-N, ...` line before the first section;
  one number in one place; After names existing cards without a cycle.
- The Plan cycle check moved from mod.rs into cards.rs as a generic `acyclic`, shared by Plan rows and cards; the
  D127 marker stays on the Plan graph in mod.rs.
- `Key` implements Display for card names and messages.

## Verification

- `just run test project_memory::tests::cards::`: 6 passed; `just run test project_memory::tests::`: 47 passed.
- `just run rustfmt` and `just lint` pass.

## Blockers

- Protected files of AC1 (b209-proposal/ in the session scratchpad) wait for the `! just approve` of the user after
  the P199-P204 merge, the rebase and a pin of the rebased head.

## Next action

Commit VAC C, then VAC D: consent records and holds by Plan/ cards (AC7).
