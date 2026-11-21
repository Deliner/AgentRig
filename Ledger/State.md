# State

## Focus

P183 (B190): the memory check parses sources on several threads and checks them in the former order.

## Workspace

Branch: feature/parallel-parse
Revision: 187a90a96d0ee18441f739ed83d97d72dd4e85f8

## Progress

VAC 1 saves B190 and selects it as P183 by the user's standing instruction to improve the project autonomously.

## Verification

Before measurement on `187a90a`, release candidate on AgentRig, three runs: the decision marker step took 259, 256 and
257 ms and the reference step 161, 159 and 161 ms.

## Blockers

None.

## Next action

Commit VAC 1, then let `Sources` parse a set of paths on several threads, keep each result or error for the ordered
walk, add the two-malformed-sources test and measure again.
