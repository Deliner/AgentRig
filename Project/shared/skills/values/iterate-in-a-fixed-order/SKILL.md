---
name: iterate-in-a-fixed-order
description: Loop over an ordered collection or sorted keys instead of a HashMap or HashSet, so that output and effects keep their order between runs, or record why no order can change the result.
---

# Iterate in an order that stays the same between runs

A `HashMap` or `HashSet` yields its elements in an order that depends on a
random seed, so a `for` loop over one runs its body in another order on every
run. Written output, the first match found, the first error returned and the
order of effects change between runs and machines, and a test can pass or
fail by chance.

1. Read what the loop body produces and whether the result depends on the
   order: written output, a vector, the first match or error, effects on files.
2. When the order matters, keep the elements ordered: a `BTreeMap` or
   `BTreeSet` instead of the hash collection, or, when the hash collection must
   stay, its keys collected and sorted before the loop.
3. When the result cannot depend on the order, such as a sum, a count or a
   collection into another set, write it as that iterator expression, which
   states that the order is free. An order-dependent body moved into
   `for_each` is the same loop; do not move it to pass.
4. When the loop must stay over the hash collection and no order can change
   its results, effects or errors, record the reasoning as a decision with
   edit-decisions and put `#[expect(clippy::iter_over_hash_type)]` on the
   smallest item that holds the loop, with its exception
   `// DECISION: DNNN allow-justification` alone on the line directly above
   the attribute. List it in the decision as
   `[exception allow-justification](FILE)`; the user approves a new exception
   through propose-protected-change, as justify-allow describes.
5. Rerun the reported lint check and the affected behavior checks.

## Rule binding

The rig's Clippy profile denies `clippy::iter_over_hash_type` in the library,
binaries and examples of the configured Cargo manifest; tests are outside the
profile. Clippy reports a `for` loop over a `HashMap` or `HashSet` or one of
their iterators; iterator methods outside a `for` loop are not reported. The
lint check reports each message as an error naming this skill. Do not weaken
the profile merely to pass. Apply complexity-discipline; stop when the loop
runs in a fixed order or the code states an order-free result, or an approved
exception lifts the finding.
