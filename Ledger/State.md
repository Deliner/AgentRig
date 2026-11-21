# State

## Focus

P182 (B189): the exception check does not parse a file none of whose lines holds an exception.

## Workspace

Branch: feature/exception-scan
Revision: ee8b50cd3c7be5f9793b37250a94410f3f9a7e78

## Progress

VAC 1 `ee8b50c` saved and selected the card. VAC 2 makes `source_parsers::exceptions` return no exception without a
parse when no line suffix starting at `//` or `#` reads as an exception marker with a rule, and adds a memory test: an
exception in a trailing comment counts, a marker inside a string does not.

## Verification

Full suite 615 passed: the 614 of master and the new test. The new test fails when the scan reads only line starts.
`just lint` clean. On AgentRig the exception step parses 23 of 486 sources; release candidate, three runs: 268, 270
and 265 ms before, 115, 117 and 115 ms after. A first full run failed one test with a full disk; two stale grep job
logs of 5.8 GB and 0.7 GB in `.cache/runtime/jobs` were removed and the rerun passed.

## Blockers

None.

## Next action

Commit VAC 2, then record Delivery, archive P182 and B189 and run `just feature-merge`.
