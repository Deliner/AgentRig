# State

## Focus

P174 delivers B181: the gate and tests run faster and the code gets simpler while behavior stays the same.
VAC 8 reads the forked details and linked tests of all invariants through one Git process in invariant history.

## Workspace

Branch: feature/faster-gate
Revision: 3e00c322a87a52044c13edfddf9fa595fc5f6e92
VAC 8 stages Project/project_memory/invariant_history.rs. Unstaged temporary timing probes in other files stay out of
the commit and are removed before the branch ends.

## Progress

VAC 1 to 7 are committed. Invariant history spawned two Git reads per invariant at the fork. VAC 8 reads those
distinct paths ahead through the existing revision_memory::read_all batch. Paths whose link does not resolve keep the
single read, so their errors arise at the same point as before.

## Verification

Warm memory-check on this repository: 1.61 s before, 1.21 s after. just test project_memory: 43 passed, including
an_invariant_change_since_the_fork_needs_a_new_decision_that_names_it.

## Blockers

None.

## Next action

Commit VAC 8, remove the timing probes, remeasure the full suite and the longest groups, then archive P174 and B181.
