# State

## Focus

P184 (B191): an existing absolute path is resolved by one `canonicalize` call instead of one per component.

## Workspace

Branch: feature/fast-resolve
Revision: 15d8beecfdcb2bb44dc0575e798b55edd878878e

## Progress

VAC 1 saves B191 and selects it as P184 by the user's standing instruction to improve the project autonomously. The
resolver change and a temporary step probe are in the working tree, not staged.

## Verification

Before on `15d8bee`, release candidate on AgentRig, three warm runs: memory check 742, 783 and 734 ms. With the
unstaged change: 472, 475 and 467 ms.

## Blockers

None.

## Next action

Commit VAC 1, then remove the probe, add the existing-link test, run the suite and lint and commit the resolver.
