---
name: anchor-at-crate
description: Replace a Rust path that reaches two module levels up through super::super with a path from crate:: or a re-export at the parent module, or record why the reach is intended.
---

# Name the owner, not the position in the tree

A path that starts with `super::super` — in a `use` declaration, an expression
or a type — binds the file to its position in the module tree instead of to
the item's owner. Moving the directory breaks the path, and a reader has to
climb the tree to learn whose item it is. A single `super::` names the
enclosing module, which moves together with the file; `crate::` names the
owner directly.

1. Write the path from the crate root, `crate::feature::part::Item`, so the
   owner is readable at the use site and survives a move of this directory. A
   descendant may name a private item of any ancestor this way, so no
   visibility change is needed; several reaches into one module can share a
   single `use crate::...::{self as owner, Item}` and call `owner::function`.
2. If the parent module already presents the item as part of its own surface,
   re-export it there (`pub(super) use` or `pub use` in the parent) and reach it
   through a single `super::Item`.
3. If the reach is genuinely intended — a macro-generated module tree, or a
   file whose position is the contract — record the reasoning as an owned
   `DECISION:`/`INVARIANT:` reference on the line directly above the
   declaration or statement.
4. Rerun the reported lint check and the affected behavior checks.

## Rule binding

The measurement flags every Rust path whose segments contain the pair
`super::super`, in a `use` declaration, an expression or a type, outside test
code, without a `DECISION:` or `INVARIANT:` reference on the line directly above
it; each path yields one finding at its line. A single `super::` and a `crate::`
path are not findings. Test code is out of scope. Rerun the reported lint check
and relevant behavior checks. Do not weaken the rule or its selectors merely to
pass. Apply complexity-discipline; stop when every reach names its owner, or the
reach is justified by an owned reference.
