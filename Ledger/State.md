# State

## Focus

P153 (B154 AC1–AC2), VAC2: pin `3af5afb`, switch the rig's test command to `operands`, archive P153 and B154 (D202).

## Workspace

Branch: feature/program-arguments
Revision: 3af5afb731d13bca812e544335215c1e53094bc5

## Progress

- `just bootstrap HEAD` installed `3af5afb` and wrote it to stable.txt.
- The rig's test command takes `accepts_args: operands`; its rig difference is removed.
- P153 and B154 moved to the archive with delivery evidence; D202 links point to the archive.

## Verification

- `just config-check` passes with the installed runtime.
- Live: `just read -- rg --pr cat x` and `just read -- git grep -iOcat x` are refused; `just run test -- --config
  foreign.toml` is refused; `just read -- rg --pretty -c …` and `just run test -- project_config::tests::settings …`
  pass, 7 tests.
- `just check --only memory` and `just lint` pass.

## Blockers

None.

## Next action

Commit VAC2, run `just feature-merge`, then start P154.
