---
name: convert-without-as
description: Replace an as conversion with From, TryFrom or a named method that shows whether the conversion can lose information, or record why the as cast is the intended conversion.
---

# Convert without `as`

`value as u16` converts silently. It cuts off the high bits of an integer that
does not fit, turns a negative number into a large unsigned one, drops the
fraction of a float and clamps it to the integer range, and changes the type
of a pointer or makes it an address. The line does not show which of these it
means, and when the source type changes later the cast keeps compiling with
another meaning.

1. Read the source and target types and decide what a value that does not fit
   means here: impossible by the types, an error, or a value to clamp.
2. For a conversion that cannot lose information, call `From`:
   `u64::from(count)`, `f64::from(ratio)`, `u32::from(letter)` for a `char`,
   `u8::from(flag)` for a `bool`.
3. For a conversion that can fail, call `TryFrom` and handle its error:
   `u16::try_from(port)?`, or a `match` that names the out-of-range case.
   `usize` and `u64` convert through `TryFrom` in both directions, since the
   width of `usize` depends on the platform. When clamping is the meaning,
   write it: `u8::try_from(level).unwrap_or(u8::MAX)` for an unsigned level.
4. For other casts call the method that names the conversion:
   `char::from_u32`, `char::from_digit`, `ptr.cast::<T>()`, `ptr.cast_mut()`,
   `ptr.cast_const()` or `ptr.addr()`. For the number of a variant of a
   fieldless enum, write a method that matches every variant.
5. When `as` is the conversion meant, such as a float converted to an integer,
   for which the standard library has no `From` or `TryFrom`, record the
   reasoning as a decision with edit-decisions and put
   `#[expect(clippy::as_conversions)]` on the smallest item that holds the
   cast, with its exception `// DECISION: DNNN allow-justification` alone on
   the line directly above the attribute. List it in the decision as
   `[exception allow-justification](FILE)`; the user approves a new exception
   through propose-protected-change, as justify-allow describes.
6. Rerun the reported lint check and the affected behavior checks.

## Rule binding

The rig's Clippy profile denies `clippy::as_conversions` in the library,
binaries and examples of the configured Cargo manifest; tests are outside the
profile. Clippy reports every `as` cast, whether or not it can lose
information, and the lint check reports each message as an error naming this
skill. Do not weaken the profile merely to pass. Apply complexity-discipline;
stop when each conversion states its meaning, or an approved exception lifts
the finding.
