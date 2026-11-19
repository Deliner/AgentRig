# State

## Focus

P150 (B151) allowed program languages, VAC2: `init` adds the program-language rule with its `--language`, the rig's
rule allows rust, shell and python, and the pin moves to c805f9a, the runtime that knows the kind (D198).

## Workspace

Branch: feature/allowed-languages
Revision: c805f9a99dec24c824e93314243425580303b8d7

## Progress

VAC1 c805f9a added the rule kind, its table, validation, evaluation and skill. VAC2 adds the init rule excluding
the generated hooks and launcher, the rig rule with the same excludes, the listed rig difference for shell and
python, the init test and the D198 owner links.

## Verification

`just run test -- project_setup structural_lint`: 367 passed. Candidate lint, Clippy and the memory check pass.
With a temporary `tooling/probe.js` the rig's lint reported `javascript program; allowed languages: rust, shell,
python` with the absolute rewrite-in-allowed-language skill path.

## Blockers

None.

## Next action

Commit VAC2, run `just bootstrap HEAD`, archive P150 and B151 with the pin, then `just feature-merge`.
