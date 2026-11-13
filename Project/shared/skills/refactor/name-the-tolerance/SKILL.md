---
name: name-the-tolerance
description: Replace f32::EPSILON or f64::EPSILON compared as an absolute tolerance with a named tolerance scaled to the compared values and the accuracy of the computation, or record why EPSILON is the intended bound.
---

# Compare floats with a tolerance the computation justifies

`(a - b).abs() < f64::EPSILON` reads as "equal up to rounding", but `EPSILON`
is only the gap between 1.0 and the next float. Above 2 the gap between
neighbouring floats is wider than `EPSILON`, so the test turns into exact
equality; near zero it accepts values that differ by orders of magnitude.
Rounding error also grows with every operation, so even values near 1.0 rarely
stay within one `EPSILON` of each other after a computation.

1. Establish what the comparison decides and the range, units and accuracy of
   the compared values and of the operations that produced them.
2. Replace `EPSILON` with a named constant or parameter that states the
   tolerance, such as `const POSITION_TOLERANCE: f64 = 1e-9;`, scaled to the
   magnitude when it varies:
   `(a - b).abs() <= TOLERANCE * a.abs().max(b.abs())`.
3. When the code asks for an exact value, such as zero or a whole number,
   compare with that value or with the call that states it, such as
   `v.fract() == 0.0`.
4. When `EPSILON` is the intended bound, such as an algorithm specified in
   units of the machine epsilon around 1.0, record why as an owned
   `DECISION:`/`INVARIANT:` reference on the line directly above the comparison.
5. Rerun the reported lint check and the affected behavior checks.

## Rule binding

The measurement flags every Rust path ending in `f32::EPSILON` or
`f64::EPSILON`, alone, in parentheses or under unary minus, that is a direct
operand of `<`, `<=`, `>`, `>=` or `==`, outside test code, without a
`DECISION:`/`INVARIANT:` reference on the line directly above the comparison.
A tolerance computed from `EPSILON`, such as `EPSILON * scale`, a named
constant and a comparison with `!=` are not flagged. Do not weaken the rule or
its selectors merely to pass. Apply complexity-discipline; stop when the
comparison uses a tolerance the computation justifies or an owned reference
justifies `EPSILON`.
