---
name: classify-in-the-enum
description: Move a classification of enum variants into true and false that a match repeats in several places into one method of the enum and call it in each place, or record why the copy must stay.
---

# Keep one answer per question about an enum

A `match` that maps variants to `true` and `false`, such as
`match status { Status::Accepted => true, Status::Full | Status::Invalid => false }`,
answers one question about the enum. Written in several places, the answer has
several copies: a new variant or a changed answer must be found and decided at
each copy, and the copies can drift apart.

1. Open the first match the finding names and the reported copy, and confirm
   they answer the same question about the same enum. Two enums with one name
   in different modules ask different questions.
2. Give the enum a method named after the question, such as
   `const fn accepted(self) -> bool`, whose body is the classification. When the
   enum belongs to another crate, use a function of the project or an extension
   trait instead.
3. Replace every copy, the first one included, with a call to that method.
4. When a copy must answer on its own on purpose, such as a test oracle for the
   method, record the reasoning as a decision with edit-decisions and put its
   exception `// DECISION: DNNN RULE`, RULE being the rule ID the finding
   names, alone on the line directly above the match. List it in the decision
   as `[exception RULE](FILE)`; the user approves a new exception through
   propose-protected-change.
5. Rerun the reported lint check and the affected behavior checks.

## Rule binding

The measurement takes every Rust `match` whose arms are unguarded and map paths
`E::V`, alone or in an or-pattern, to the literals `true` and `false`, both
present, naming three or more paths, in product and test code. It reports such
a match when an earlier one among the files the rule selects, in path and line
order, has the same set of variant and result pairs, and names that first
match. An exception naming this rule, `repeated-classification` by default, on
the line directly above the reported match lifts the finding; a bare
`DECISION:` or `INVARIANT:` reference lifts nothing. A variant is the last
segment of `E` with `V`, so `crate::m::E::V` and an imported `E::V` are one
variant, while `Self::V` compares as `Self`. Do not weaken the rule or its
selectors merely to pass. Apply complexity-discipline; stop when one method
holds the classification or an approved exception lifts the finding.
