---
name: anchor-at-crate
description: Replace a Rust path that reaches two module levels up through super::super with a path from crate:: to the file that owns the item, or record why the reach is intended.
---

# Name the owner, not the position in the tree

A path that starts with `super::super` — in a `use` declaration, an expression,
a type or a `pub(in super::super)` visibility — binds the file to its position
in the module tree instead of to the item's owner. Moving the directory breaks
the path, and a reader has to climb the tree to learn whose item it is. A single
`super::` names the enclosing module, which moves together with the file;
`crate::` names the owner directly.

1. Write the path from the crate root to the file that defines the item,
   `crate::feature::part::Item`, so the owner is readable at the use site and
   survives a move of this directory. A private module of an ancestor is
   visible to its descendants, so only the item may need a restricted
   visibility such as `pub(in crate::feature)`; several reaches into one
   module can share a single `use crate::...::{self as owner, Item}` and call
   `owner::function`. Write a restricted visibility the same way:
   `pub(in super::super)` becomes `pub(in crate::feature)` naming the same
   module, so the item stays exactly as visible.
2. Never name an item that an ancestor's entry defines or reexports: a
   directory's own code does not use its entry, entry-use reports such a use,
   and move-out-of-entry moves the item into a file.
3. If the reach is genuinely intended — a macro-generated module tree, or a
   file whose position is the contract — record the reasoning as a decision
   with edit-decisions and put its exception `// DECISION: DNNN RULE`, RULE
   being the rule ID the finding names, alone on the line directly above the
   path's line. List it in the decision as `[exception RULE](FILE)`; the user
   approves a new exception through propose-protected-change.
4. Rerun the reported lint check and the affected behavior checks.

## Rule binding

The measurement flags every Rust path whose segments contain the pair
`super::super`, in a `use` declaration, an expression, a type or a `pub(in ..)`
visibility, outside test code; each path yields one finding at its line. An
exception naming this rule, `super-super-reach` by default, on the line
directly above that line lifts the finding; a bare `DECISION:` or `INVARIANT:`
reference lifts nothing. A single `super::` and a `crate::` path are not
findings. Test code is out of
scope: an item inside a `#[cfg(test)]` module or a `#[test]` function of the
same file, and every file under the shipped rule's exclude, `**/tests/**`. A
test module kept in a file outside a `tests/` directory is measured, because
its `#[cfg(test)]` sits in the parent file. Rerun the reported lint
check and relevant behavior checks. Do not weaken the rule or its selectors
merely to pass. Apply complexity-discipline; stop when every reach names its
owner, or an approved exception lifts the finding.
