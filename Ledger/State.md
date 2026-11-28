# State

## Focus

P200 (B109 AC1–AC2), decision VAC: record D238, the successor of D157, in the three sections of D236. D238 binds
a cargo oracle to a module path of its linked file joined with the file-local name of the marked function.

## Workspace

Branch: feature/cargo-oracle-file
Revision: b7587b09d9a23386e621021bc48cded11684ea36
The branch is rebased onto master 032fddc (P199 merged); the obsolete pause commit of P200 was dropped, so P200
stays active. Staged: Decisions.md, Decisions/238.md, the markers and State.

## Progress

- Decisions.md: the D238 row with links to oracles.rs, cargo_crates.rs, tests/oracles.rs and the setup guide; the
  D157 row keeps only its card and the D144 link; the succession note says D238 supersedes D157.
- Decisions/238.md: Decision, Reasons and Alternatives.
- Markers: D157 becomes D238 in oracles.rs and tests/oracles.rs; cargo_crates.rs gains D238 above cargo_modules.

## Verification

- `just candidate memory-check`: passes with the three known marker warnings (D112, D175, D231).
- The commit gate checks this staged tree with the affected test groups.

## Blockers

None for this VAC. Completing P200 needs a pin, which `just propose-pin HEAD` sets under the consent of the user.

## Next action

Commit this VAC through the gate, then run `just propose-pin HEAD`, archive P200 and B109 and merge.
