---
name: match-the-ranges
description: Replace an if and else-if chain or a match with guards that compares one value with integer bounds by a match on range patterns, so that the compiler checks gaps and unreachable branches, or record why the chain must stay.
---

# Let the compiler check the ranges

`if size < 10 { .. } else if size < 100 { .. } else { .. }` describes ranges
through their order. A bound written as `<` where `<=` was meant leaves a gap,
and reordering the branches makes one unreachable, with no compile error. A
`match` with guards such as `_ if size < 10` has the same weakness.

1. Read the chain and establish the range each branch is meant to cover and what
   happens outside all of them.
2. Replace it with a `match` on the value whose arms are range patterns, such as
   `0..=9`, `10..=99` and `100..`, and keep the outcomes of the branches.
3. Name a bound that has a meaning as a `const` and use it in the patterns.
4. Let the compiler's exhaustiveness and unreachable-pattern checks confirm the
   coverage; a remaining gap becomes an explicit arm.
5. When the chain must stay, such as bounds that are not constants of the
   value's type, record the reasoning as a decision with edit-decisions and
   put its exception `// DECISION: DNNN RULE`, RULE being the rule ID the
   finding names, alone on the line directly above the reported line. List it
   in the decision as `[exception RULE](FILE)`; the user approves a new
   exception through propose-protected-change.
6. Rerun the reported lint check and the affected behavior checks.

## Rule binding

The measurement reports every Rust `if` and `else if` chain of two or more
conditions, and every `match` with two or more guards, that compares one
token-equal expression with an integer literal by `<`, `<=`, `>` or `>=`. Test
code is out of scope. An exception naming this rule, `range-chain` by default,
on the line directly above the reported line lifts the finding; a bare
`DECISION:` or `INVARIANT:` reference lifts nothing. Do not weaken the rule or
its selectors merely to pass. Apply complexity-discipline; stop when the ranges
are patterns of one match or an approved exception lifts the finding.
