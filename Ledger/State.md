# State

## Focus

P205 (B209), one card per task. The branch delivers R2a, the Project changes before the user's approval. VAC A: the
memory check reads the cards in Backlog/, Plan/ and Archive/ instead of the Plan and Backlog indexes, and D242 records
the choice.

## Workspace

Branch: feature/RIG-209-one-card-per-task
Revision: 3df67fd, with VAC A staged for its commit.

The installed runtime c60786d accepts the optional `key` in project.yaml. The proposal under .cache/runtime/proposal
holds the protected changes that wait for the user's `! just approve`: `key: RIG` in project.yaml, `--key RIG` in
bootstrap.sh, the card routes in hooks.yaml, the card excludes in lint.yaml and the rig_differences entry.

## Progress

- cards.rs checks card names by the queue key, a number held by one card, card directories and their materials, the
  heading and sections, and After order without a cycle; the registries kept in Archive/ stay unread; a memory with
  cards needs the `key` of project.yaml.
- mod.rs calls cards::check instead of the Plan and Backlog index checks; format.rs offers sections_of.
- Key gains Display and Config gains queue_key; the shared project fixture writes `key: RIG`.
- Tests: plan.rs keeps I009 for the card order, cards.rs replaces backlog.rs, validation.rs checks I008 on Decisions.
- D242 with its detail and supersession note; D012, D031 and D127 link cards.rs and D050 drops its mod.rs and
  backlog.rs links; I008 and I009 revised under D242.

## Verification

- `just run test -- project_memory::tests`: 40 passed.
- `just run rustfmt`: clean; `just run lint -- --warnings`: warnings only, none in the changed files.
- `just check --only memory` with the installed runtime: PASS.
- `just run test`: 648 passed and project_key_takes_two_to_ten_uppercase_latin_letters failed, since it appended a
  second key to the fixture; it now replaces the key, and `just run test -- project_config::tests` passes 11.

## Blockers

The rig's own Ledger keeps the old layout until the card runtime is pinned, so the candidate's memory check refuses
it until the migration, which lands with the pin after the user's `! just approve`.

## Next action

Commit VAC A, then VAC B: the session start and resume name the active card or Plan/.
