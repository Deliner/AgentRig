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
   value's type, record the reasoning as an owned `DECISION:`/`INVARIANT:`
   reference on the line directly above it.
6. Rerun the reported lint check and the affected behavior checks.

## Rule binding

The measurement reports every Rust `if` and `else if` chain of two or more
conditions, and every `match` with two or more guards, that compares one
token-equal expression with an integer literal by `<`, `<=`, `>` or `>=`. Test
code and a chain with a `DECISION:` or `INVARIANT:` reference on the line
directly above it are out of scope. Do not weaken the rule or its selectors
merely to pass. Apply complexity-discipline; stop when the ranges are patterns
of one match or the chain is justified by an owned reference.
