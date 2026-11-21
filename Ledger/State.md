# State

## Focus

P174 delivers B181: the gate and tests run faster and the code gets simpler while behavior stays the same.
VAC 7 validates a lint rule's error skill only when it differs from its warning skill, which checked the same file.

## Workspace

Branch: feature/faster-gate
Revision: 46ee90279ea940588c0319237c177f1fad0833f6
VAC 7 stages Project/structural_lint/rule_set/lint_config/validation.rs. Unstaged temporary timing probes in other
files stay out of the commit and are removed before the branch ends.

## Progress

VAC 1 to 6 are committed. In the delivered lint configuration 84 of 85 rules name one skill for both levels, so every
process that loads the configuration canonicalized and read each such SKILL.md twice. A distinct error skill is still
validated, in the same order, with the same error.

## Verification

Context load per process, measured in the commit flow probe: about 10.4 ms before, 8.3 ms after.
just test structural_lint::tests::configuration: 8 passed.

## Blockers

None.

## Next action

Commit VAC 7, then read the forked invariant details and oracle sources through one Git process in invariant history.
