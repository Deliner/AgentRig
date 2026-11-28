---
name: box-the-large-array
description: Move a large local array to the heap or into a static item, so that it cannot overflow the stack of a thread, or record the stack size that holds it.
---

# Keep a large array off the stack

A local array lives on the stack of the thread that runs the function. A
large one, such as `[0_u8; 1 << 20]`, can overflow that stack, which aborts the
process without an error to handle. The size that still fits differs between
the main thread, spawned threads and platforms, so the code can pass on one
machine and abort on another.

1. Read whether the array changes and how long it must live.
2. For a buffer, allocate it on the heap: `vec![0_u8; LEN]`, or
   `vec![0_u8; LEN].into_boxed_slice()` when its length never changes.
   `Box::new([0_u8; LEN])` can still build the array on the stack first, so it
   does not help.
3. For a table that never changes, make it a `static` item, which does not
   live on the stack.
4. When the array must stay local, such as in a hot loop on a thread spawned
   with a stack size that holds it, record the reasoning and that size as a
   decision with edit-decisions and put `#[expect(clippy::large_stack_arrays)]`
   on the smallest item that holds the array, with its exception
   `// DECISION: DNNN allow-justification` alone on the line directly above
   the attribute. List it in the decision as
   `[exception allow-justification](FILE)`; the user approves a new exception
   through propose-protected-change, as justify-allow describes.
5. Rerun the reported lint check and the affected behavior checks.

## Rule binding

The rig's Clippy profile denies `clippy::large_stack_arrays` in the library,
binaries and examples of the configured Cargo manifest; tests are outside the
profile. Clippy reports a local array larger than its `array-size-threshold`,
and the lint check reports each message as an error naming this skill. Do not
raise the threshold or weaken the profile merely to pass. Apply
complexity-discipline; stop when the array lives on the heap or in a static
item, or an approved exception lifts the finding.
