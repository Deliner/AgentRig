---
name: justify-unsafe
description: Justify every unsafe site (block, function, impl, trait, static mut or unsafe inside a macro body) with a recorded reference, or remove the unsafe.
---

# Record why each unsafe is sound, or remove it

Every unsafe site must carry a DECISION: or INVARIANT: reference on the line
directly above it: an unsafe block, an unsafe function, an unsafe impl or unsafe
trait, a static mut item, and an unsafe keyword inside a macro_rules! body or a
macro invocation's arguments other than one opening a `#[unsafe(..)]` or
`#[cfg_attr(cond, unsafe(..))]` attribute. Attribute lines between the reference
and an item do not break the adjacency; inside a macro body the reference sits
on the line directly above the unsafe keyword. The reference points to the
recorded reason the unsafe is sound.

1. Read the unsafe construct and establish the exact obligation it takes on: the
   FFI or pointer contract it must satisfy, the arguments it builds, the return
   value it checks, the aliasing or lifetime it relies on, the Send or Sync
   guarantee an unsafe impl asserts, or the exclusive access a static mut needs.
2. Prefer removing the unsafe. A safe standard API, a checked wrapper, or a crate
   that already owns the invariant discharges the obligation instead of recording
   a reason to keep it.
3. When the unsafe is genuinely required, record the soundness reason as a
   decision or bind it to an existing invariant, then put its `// DECISION: DNNN`
   or `// INVARIANT: INNN` reference on the line directly above the unsafe.
4. Keep one reason per obligation: distinct unsafe constructs that rely on the
   same recorded reason each carry that reference on their own line above.
5. Rerun the reported lint check and the affected behavior checks.

The reference is the only accepted form: an exact `// DECISION: DNNN` or
`// INVARIANT: INNN` comment on the line directly above the site. By project
convention the rule does not read a `// SAFETY:` comment or a `# Safety` doc
section, even when it already explains the soundness: record that explanation
as the decision or invariant and reference it. Any other comment does not
justify the unsafe either. Do not silence the finding with a plain comment or by
widening the rule.

## Rule binding

The measurement flags every Rust unsafe site, an unsafe block, unsafe function,
unsafe impl, unsafe trait or static mut item and every unsafe keyword inside a
macro_rules! body or a macro invocation's arguments except one opening a
`#[unsafe(..)]` or `#[cfg_attr(cond, unsafe(..))]` attribute, without a
DECISION: or INVARIANT: reference comment on the line directly above it;
attribute lines directly above an item keep the adjacency, so the reference may
stand above `#[cfg(..)]`. No other comment or doc section is read. Rerun the reported lint check and relevant behavior checks.
Do not weaken the rule or its selectors merely to pass. Apply
complexity-discipline; stop when the finding is resolved and the unsafe is
either removed or justified.
