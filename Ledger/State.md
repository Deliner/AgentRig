# State

## Focus

P205 (B209) VAC B, the next part of AC1: `init --key KEY` checks the key through the `Key` type and writes it into
`.agentrig/project.yaml`; another form refuses before any file is written. Without `--key`, init still writes no key,
so the rig comparison (I023) holds until the rig module names `key: RIG` with the user's approval.

## Workspace

Branch: feature/card-queue
Revision: 188d691a198acf70c083c26b81f2dedf527bd333

## Progress

- `template.rs` accepts `--key`, and `config` parses it once with `Key::try_from` under the context `init --key`;
  `project_config` exports `Key` as part of its configuration API.
- The usage line of init, the CLI help and the project_setup README name `--key`.
- Requiring the key waits for the approved rig module and the bootstrap passing `--key RIG`: a pinned runtime that
  accepts `--key` lets that bootstrap install any later revision.

## Verification

- `project_setup::initialization::tests::layout::init_writes_the_queue_key_and_refuses_another_form_before_writing`:
  `--key rig-1` errs and leaves the consumer unchanged, `--key RIG` writes `key: RIG`; the 9 layout tests pass.
- `just run rustfmt` and `just lint` pass on this content; the commit gate runs the affected groups.

## Blockers

None for this VAC.

## Next action

Commit VAC B, pin its head under the consent of the user, then pause P205 until the user approves the rig module
with `key: RIG` and the bootstrap with `--key RIG`.
