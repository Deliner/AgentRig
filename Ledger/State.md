# State

## Focus

P203 (B138 AC1–AC3): a finding of a lint of the rig's Clippy profile leads to a skill naming that lint; other findings
keep the configuration skill. VAC 1 of 4: move the canonical name-if-condition skill into the naming group, so that
the top skills directory, at its limit of 15 entries, has room for the new group of Clippy skills.

## Workspace

Branch: feature/clippy-lint-skills
Revision: 76d8a75
P203 is active in Plan. VAC 1 moves Project/shared/skills/name-if-condition to naming/name-if-condition, keeps the
flat alias as a symlink, and updates the init bundle path and the architecture of the top and naming groups.

## Progress

The mapping is settled: 23 profile lints lead to 14 existing skills (handle-panic, remove-debug, expect-the-panic,
name-the-states, name-the-placeholders, match-every-variant, justify-allow, name-the-target, handle-discarded,
subtract-checked, handle-error, justify-unsafe, release-the-guard, destructure-the-fields), and 14 lints to 13 new
skills: nine in a new values group, choose-field-visibility and mark-must-use in signature, print-through-one-owner
in ownership and move-into-mod-rs in refactor. Next VACs: the new skills with init bundle entries, the existing skills
naming their lints, then the profile pairs each lint with its skill and D240 records the choice of skill.

## Verification

VAC 1: `just lint` passes; `just run test -- init_delivers_the_universal_capabilities_of_this_repository` passes
(I020: init delivers the same skill set as Project/shared/skills). `just config-check` passes with the flat alias;
.agentrig/lint.yaml is unchanged.

## Blockers

None.

## Next action

Commit VAC 1, then write the 13 new skills with their init bundle entries (VAC 2).
