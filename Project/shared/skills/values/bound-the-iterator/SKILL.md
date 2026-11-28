---
name: bound-the-iterator
description: Bound an iterator that may never end before a search consumes it, so that the search stops on every input, or record why the iteration runs until the process stops.
---

# Bound an iterator that may never end

`0..`, `iter::repeat(..)` and `cycle()` produce elements without end. A search
that consumes one, such as `find`, `position` or `any`, stops only when an
element matches, so for an input where none matches the program hangs.

1. Find the bound the search actually has: the number of candidates, the
   length of the data a counter indexes or a retry limit. A free name among
   `taken` names is found within `taken.len() + 1` tries.
2. Iterate within that bound: `1..=limit` instead of `1..`, `.take(limit)`
   before the search, or the finite collection itself, such as
   `items.iter().enumerate()` instead of a counter that indexes `items`.
3. Handle a bound reached without a match: return an error or a `None` that
   the caller sees.
4. When the iteration is meant to run until the process stops, such as a
   server's stream of events, record the reasoning as a decision with
   edit-decisions and put `#[expect(clippy::maybe_infinite_iter)]` on the
   smallest item that holds it, with its exception
   `// DECISION: DNNN allow-justification` alone on the line directly above
   the attribute. List it in the decision as
   `[exception allow-justification](FILE)`; the user approves a new exception
   through propose-protected-change, as justify-allow describes.
5. Rerun the reported lint check and the affected behavior checks.

## Rule binding

The rig's Clippy profile denies `clippy::maybe_infinite_iter` in the library,
binaries and examples of the configured Cargo manifest; tests are outside the
profile. Clippy reports a search or another consumer that may stop on an
iterator without end. It does not analyze a condition that would stop the
iteration, so only a bound written into the iterator counts. The lint check
reports each message as an error naming this skill. Do not weaken the profile
merely to pass. Apply complexity-discipline; stop when the iteration has a
bound, or an approved exception lifts the finding.
