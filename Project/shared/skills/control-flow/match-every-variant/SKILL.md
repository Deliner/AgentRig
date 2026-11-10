---
name: match-every-variant
description: Replace == or != with an enum variant, or a matches! naming one, with an exhaustive match so that the compiler names each variant added later, or record why the comparison must stay.
---

# Let the compiler name every variant

`if kind == Kind::File`, `state != State::Done` or
`matches!(mode, Mode::Fast)` decides one variant and sends every other variant to
the other branch. A variant added later lands there silently, whether or not
that branch is right for it.

1. Read the comparison and the enum, and establish what each variant should do
   at this site.
2. Replace the comparison with a `match` that lists every variant, grouping
   variants with `|` and mapping each group to its outcome, without a `_` arm
   for an enum the project owns; a variant added later then fails to compile
   until this site decides it.
3. When several sites need the same answer, give the enum one method that holds
   the exhaustive match, such as `fn is_terminal(&self) -> bool`, and call it.
4. When the enum is foreign and closed by definition, such as
   `std::path::Component`, record the reasoning as an owned
   `DECISION:`/`INVARIANT:` reference on the line directly above the comparison.
5. When the enum is foreign and open, such as `io::ErrorKind`, a full match does
   not fit each site: keep the comparison in one owner, such as a function,
   that carries the reference, and call it from the sites.
6. Rerun the reported lint check and the affected behavior checks.

## Rule binding

The measurement reports every Rust `==` or `!=` with an operand that is a path
of two or more segments ending in an UpperCamelCase name, and every `matches!`
whose pattern names such a path. Test code and a comparison with a `DECISION:`
or `INVARIANT:` reference on the line directly above it are out of scope. Do not
weaken the rule or its selectors merely to pass. Apply complexity-discipline;
stop when the site matches every variant or the comparison is justified by an
owned reference.
