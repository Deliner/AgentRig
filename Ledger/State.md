# State

## Focus

P205 (B209, one card per task), stage R2a, VAC C: `just consent` records the cards in Plan/ and holds while one of
them stays there.

## Workspace

- Branch: feature/RIG-209-one-card-per-task
- Revision: e697aed, with VAC C staged for its commit.
- Installed runtime: c60786d; `just run test` and `just run lint` build the candidate from the branch.
- Protected proposals waiting for `! just approve` lie in `.cache/runtime/proposal/`: `key: RIG` in project.yaml,
  `--key RIG` in bootstrap.sh, the card routes in hooks.yaml, the card excludes in lint.yaml and an entry in
  rig_differences.yaml.

## Progress

- VAC A (da03ef9) checks cards; VAC B (e697aed) names the card of the branch at session start and in resume.
- VAC C: `project_memory::planned_cards` lists the names of the cards in Plan/ by number. `just consent` records
  them and refuses when Plan/ holds none; consent holds while any recorded card stays in Plan/, so a card that
  enters Plan/ later does not renew it.
- The git test fixture runs `init` with `--key RIG`; the consent tests plan and archive cards instead of editing a
  Plan.md row. D242 links consent and its tests.

## Verification

- `just run test -- change_delivery::git::tests`: 43 passed.
- `just run rustfmt`: clean; `just run lint`: no errors.
- `just check --only memory` (installed runtime): PASS.

## Blockers

The rig's Ledger keeps the old layout until the user's `! just approve` admits the protected proposals; the pin and
the migration then land in one commit.

## Next action

Commit VAC C, then VAC D: `init` creates Backlog/, Plan/ and Archive/ without indexes and its instructions describe
cards.
