# State

## Focus

P174 B181: the gate gets faster and the code simpler while behavior stays the same. VAC 1 saves the self-approved
card B181 under the user's AFK instruction and selects it as P174.

## Workspace

Branch: feature/faster-gate
Revision: a0f7ee8c79b90c0a0be2597ea8eda5ac065a3fa9

## Progress

- P171 and P172 are archived and merged into master a0f7ee8; the pin 23aca35 is on master.
- Baseline on a0f7ee8: rig lint without findings; full run of 614 tests 103 s; groups change_delivery::git 41 s,
  project_setup 37 s, agent_hooks 26 s, structural_lint 2.8 s.

## Verification

- memory check passes with B181 and P174.

## Blockers

- The checks.yaml impact-map proposal in .cache/runtime/proposal waits for the user approve.
- P174 is not under consent: its pin needs the user's `! just approve` after the merge.

## Next action

Commit VAC 1, then measure the cost of init, feature-start, a hooked commit and feature-merge in the git delivery
tests and remove the largest avoidable cost without changing what the tests check.
