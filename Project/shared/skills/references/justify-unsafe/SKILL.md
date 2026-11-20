---
name: justify-unsafe
description: Justify every unsafe site (block, function, impl, trait, static mut or unsafe inside a macro body) with an approved exception, or remove the unsafe.
---

# Record why each unsafe is sound, or remove it

Every unsafe site must carry an approved exception on the line directly above
its reported line: an unsafe block, an unsafe function, an unsafe impl or
unsafe trait, a static mut item, and an unsafe keyword inside a macro_rules!
body or a macro invocation's arguments other than one opening a
`#[unsafe(..)]` or `#[cfg_attr(cond, unsafe(..))]` attribute. An item's
exception goes below its attributes, directly above the item's own line;
inside a macro body it sits on the line directly above the unsafe keyword. The
exception names the decision that records why the unsafe is sound.

1. Read the unsafe construct and establish the exact obligation it takes on: the
   FFI or pointer contract it must satisfy, the arguments it builds, the return
   value it checks, the aliasing or lifetime it relies on, the Send or Sync
   guarantee an unsafe impl asserts, or the exclusive access a static mut needs.
2. Prefer removing the unsafe. A safe standard API, a checked wrapper, or a crate
   that already owns the invariant discharges the obligation instead of recording
   a reason to keep it.
3. When the unsafe is genuinely required, record the soundness reason as a
   decision with edit-decisions and put its exception `// DECISION: DNNN RULE`,
   RULE being the rule ID the finding names, alone on the line directly above
   the unsafe, below an item's attributes. List it in the decision as
   `[exception RULE](FILE)`; the user approves a new exception through
   propose-protected-change.
4. Keep one reason per obligation: distinct unsafe constructs that rely on the
   same recorded reason each carry that exception on their own line above.
5. Rerun the reported lint check and the affected behavior checks.

The exception is the only accepted form: an exact `// DECISION: DNNN RULE`
comment naming this rule, `unsafe-justification` by default, on the line
directly above the site. By project convention the rule does not read a
`// SAFETY:` comment or a `# Safety` doc section, even when it already explains
the soundness: record that explanation as the decision and put its exception
there. A bare `DECISION:` or `INVARIANT:` reference or any other comment does
not justify the unsafe. Do not silence the finding with a plain comment or by
widening the rule.

## Rule binding

The measurement flags every Rust unsafe site, an unsafe block, unsafe function,
unsafe impl, unsafe trait or static mut item and every unsafe keyword inside a
macro_rules! body or a macro invocation's arguments except one opening a
`#[unsafe(..)]` or `#[cfg_attr(cond, unsafe(..))]` attribute. An exception
naming this rule on the line directly above the reported line lifts the
finding; for an item that is the item's own line, so an exception above its
`#[cfg(..)]` lifts nothing. No other comment or doc section is read. Rerun the
reported lint check and relevant behavior checks. Do not weaken the rule or its
selectors merely to pass. Apply complexity-discipline; stop when the finding is
resolved and the unsafe is either removed or lifted by an approved exception.
