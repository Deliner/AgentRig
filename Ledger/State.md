# State

## Focus

P174 delivers B181: the gate and tests run faster and the code gets simpler while behavior stays the same.
VAC 6 parses each source file once per memory check: decisions, references and invariants share one parse cache.

## Workspace

Branch: feature/faster-gate
Revision: 263df9267bded8ea1db126b0af1b871e75570557
VAC 6 stages Project/project_memory/mod.rs and Project/project_memory/source.rs. Unstaged temporary timing probes in
other files stay out of the commit and are removed before the branch ends.

## Progress

VAC 1 to 5 are committed. VAC 6 adds Sources, a per-check cache of parsed sources keyed by the exact path, so a file
that several decisions link, the source scan and the invariant oracles parse it once. Errors arise at the same first
parse as before. Threading the cache into invariant history gave no measurable gain and was dropped.

## Verification

memory-check on this repository, warm: about 4.3 s before, 1.55 s after; decisions 660 to 254 ms, references 395 to
234 ms. just test project_memory: 43 passed.

## Blockers

None.

## Next action

Commit VAC 6, then commit the lint validation that checks a rule's error skill only when it differs from its warning
skill, then batch the forked invariant reads in invariant history.
