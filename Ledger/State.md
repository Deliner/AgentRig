# State

## Focus

P205 (B209, D242), the approval VAC: `init` routes `edit-plan` for `Plan/**` and `Archive/**` and `edit-backlog` for
`Backlog/**`, its `file-size` skips the `assets/` and `results/` of cards and of the indexes only `Decisions.md` and
`Invariants.md`, and its `directory-size` skips the three card directories; the rig has the same settings, which the
user's `just approve` installed from the proposal (AC2, AC3, AC9).

## Workspace

Branch: feature/RIG-209-one-card-per-task. Revision: 1508893af1ace4e605b4c35982587f2732fdcd7e plus the approval VAC
diff, the new `Ledger/Decisions/243.md` and the approved protected files. The installed runtime is c60786d; the
candidate is built from this branch.

## Progress

- project_memory exports `BACKLOG`, `ARCHIVE` and `STORED_MATERIALS`, which the card check and the init templates
  share.
- bundle/template.rs writes the card routes in positions 1 to 3, as the proposed hooks.yaml has them; bundle/lint.rs
  writes the excludes of the proposed lint.yaml except `Ledger/Assets/**`, `Archive/Plan` and `Archive/Backlog`.
- The hook tests use card paths; defaults.rs checks that file-size skips the two indexes and the materials of a card
  but measures `Decisions/001.md` and the card file; rig.rs initializes with `--key RIG`.
- D243 records that I007's test checks the guidance on `Plan/RIG-7.md` after compaction; D029 and D030 link their
  successor D242; D048, D056 and D172 no longer link bundle/lint.rs; D242 links the lint and route settings, the init
  lint and routes and their tests.
- The user's `just approve` installed `key: RIG`, the three card routes, the lint excludes, `--key RIG` in
  bootstrap.sh and the rig_differences entry, exactly as proposed.

## Verification

- Before the approval: `just run rustfmt`, `just run lint` and `just check --only memory` passed, and the whole suite
  passed except I023, which named only differences that the approval resolves or lists.
- After the approval: I023 (`project_setup::initialization::tests::rig`) and `just run lint` pass.

## Blockers

- P201 waits for the user's decision on its AC2; P206 waits for P205.

## Next action

Commit this VAC with the approved files; then R2b (the key that `init` requires), `just propose-pin HEAD` under the
user's consent and the migration (AC8) in the commit of the pin.
