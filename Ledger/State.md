# State

## Focus

P183 (B190): the memory check parses sources on several threads and checks them in the former order.

## Workspace

Branch: feature/parallel-parse
Revision: 29f6da014460c6d31d697e4d2835e70b8f77e6d1

## Progress

VAC 2: `Sources::prepare` parses a path set on several threads and caches each success; a failed source is parsed
again at its turn, so its error keeps its place. The decision step resolves its links first, prepares them, then checks
each row in the former order through `markers`; the reference step prepares its selected sources. A new test links two
malformed sources.

## Verification

Full suite before the lint repair: 616 passed. `just lint` after it: clean; rustfmt passes. Mutation returning the
first sorted error fails the new test. Release candidate on AgentRig, three runs: decision step 143, 146 and 134 ms
against 259, 256 and 257 ms; reference step 132, 151 and 133 ms against 161, 159 and 161 ms. A probe showed
`applications` holds 89 ms of the reference step.

## Blockers

None.

## Next action

Commit VAC 2 through the hook, then write Delivery and archive P183 and B190.
