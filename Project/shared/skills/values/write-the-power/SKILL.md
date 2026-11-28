---
name: write-the-power
description: Replace a ^ between decimal number literals that reads as a power with pow or a shift, or write an intended bitwise exclusive or so that its bits show.
---

# Write a power with `pow`

In Rust `^` is the bitwise exclusive or, not exponentiation: `2 ^ 10` is 8,
not 1024. Between two number literals written in decimal, `^` is almost always
a power the author meant, and the wrong value compiles without a warning.

1. Read the value the expression should produce.
2. For a power, call `pow` on a typed base, `10_u64.pow(6)`, or shift for a
   power of two, `1_u32 << 10`.
3. For an exclusive or that is meant, write the operands in binary or
   hexadecimal, `0b1010 ^ 0b0110`, or use named constants, so that the bits
   show.
4. Rerun the reported lint check and the affected behavior checks.

No exception keeps a decimal `^`: one of the forms above states every intent,
so do not suppress the lint.

## Rule binding

The rig's Clippy profile denies `clippy::suspicious_xor_used_as_pow` in the
library, binaries and examples of the configured Cargo manifest; tests are
outside the profile. Clippy reports a `^` between two number literals unless
one of them is written in another base than decimal, and the lint check reports
each message as an error naming this skill. Do not weaken the profile merely
to pass. Apply complexity-discipline; stop when the expression states the
power or the bitwise operation it means.
