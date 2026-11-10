---
name: pin-the-order
description: Give each variant of an enum that derives PartialOrd or Ord an explicit discriminant, compare through a named rank, or record why declaration order is the intended order.
---

# Make an enum's order explicit

A derived `PartialOrd` or `Ord` on an enum compares variants by their
discriminants, which follow the declaration order when none is written. Nothing
at the declaration marks that order as a contract, so inserting, sorting or
moving a variant silently changes every `<`, `max`, sort and `BTreeMap` order,
and comparisons with stored or transmitted values stop matching.

1. Decide whether the order means something, such as a severity, a priority or
   a phase. If nothing compares the values, remove the derive.
2. When the order is part of the meaning, give every variant an explicit
   discriminant that states it, such as `Low = 10` and `High = 20`; gaps let a
   later variant go between without renumbering.
3. When the variants carry fields, or the order is not a numbering, remove the
   derive and implement `Ord` through a named method such as
   `fn rank(&self) -> u8` whose `match` lists every variant, so a new variant
   does not compile until its rank is chosen.
4. When any stable order serves, such as a map key that needs some total
   order, record that as an owned `DECISION:`/`INVARIANT:` reference on the
   line directly above the enum or its attributes.
5. Rerun the reported lint check and the affected behavior checks.

## Rule binding

The measurement flags every Rust enum of two or more variants that derives
`PartialOrd` or `Ord`, directly or inside `cfg_attr`, when a variant lacks an
explicit discriminant, outside test code, without a `DECISION:` or
`INVARIANT:` reference on the line directly above the enum or its attributes.
A variant with fields takes a discriminant only under a primitive `repr`, so
such an enum is usually resolved by step 3 or 4. Rerun the reported lint check
and relevant behavior checks. Do not weaken the rule or its selectors merely to
pass. Apply complexity-discipline; stop when every variant carries its
discriminant, the order goes through a named rank, or the order is justified by
an owned reference.
