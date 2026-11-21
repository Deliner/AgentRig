# State

## Focus

P193 (B200): the architecture rule parses each source once and resolves dependencies on several threads.

## Workspace

Branch: feature/faster-rules
Revision: 8fef711f6dcbbaddc981ab7cd2cfb3daf7a2c6d3

## Progress

VAC 1 to VAC 3 are committed: the card, parallel dependency resolution and the single parse. VAC 4 archives P193 and
B200: the cards move to Ledger/Archive with Delivery, the timing table and the acceptance results, and their rows
move from Plan.md and Backlog.md to the archive indexes.

## Verification

On `8fef711`: `just test` passes 617 tests, as on master `ca6d64a`; `just lint` reports no findings. Median of the
architecture rule on AgentRig 307 ms before and 179.5 ms after.

## Blockers

None.

## Next action

Commit VAC 4, then run `just feature-merge`.
