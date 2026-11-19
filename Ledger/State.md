# State

## Focus

P150 (B151) allowed program languages, VAC3: pin afb7239, the runtime with the rig's program-language rule, and
archive P150 and B151 with their acceptance evidence (D198).

## Workspace

Branch: feature/allowed-languages
Revision: afb7239df854f76e32f87e7fbb58666f69875d00

## Progress

`just bootstrap HEAD` installed afb7239 and wrote it to stable.txt. P150 moved to Archive/Plan/150.md with
Delivery and the AC table, B151 to Archive/Backlog/151.md, both index rows moved and the D198 card and delivery
links point to the archive.

## Verification

The memory check passes after the move. The gate of afb7239 passed 534 tests.

## Blockers

None.

## Next action

Commit VAC3, then run `just feature-merge` and start P151 (B152).
