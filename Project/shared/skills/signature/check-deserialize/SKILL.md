---
name: check-deserialize
description: Route a Rust struct's derived Deserialize through the check its constructor enforces, usually with serde(try_from = ...) from a raw shape, so data from a file or the network cannot skip the invariant.
---

# Deserialize through the constructor's check

A constructor that refuses a bad parameter states a validity condition of the
type. A derived `Deserialize` builds the same struct field by field without
calling it, so a value read from a file, the network or a configuration skips
the check, and code that trusts the condition fails later, often with a panic
far from the input.

1. Confirm what the refusal guards. A range, a non-empty text or an order
   between fields is a condition of the stored value and must hold for every
   instance. A check of the environment, such as a path that must exist now,
   is not; move it to the operation that needs it, so the constructor stops
   refusing for it.
2. Keep one checking entry point. Give the data a raw shape that derives
   `Deserialize` with the same fields, and let serde convert it through the
   check:

   ```rust
   #[derive(Deserialize)]
   #[serde(try_from = "RawInterval")]
   pub struct Interval { start: u32, end: u32 }

   #[derive(Deserialize)]
   struct RawInterval { start: u32, end: u32 }

   impl TryFrom<RawInterval> for Interval {
       type Error = IntervalError;
       fn try_from(raw: RawInterval) -> Result<Self, Self::Error> {
           Self::new(raw.start, raw.end)
       }
   }
   ```

   The error type needs `Display` for serde to report it.
3. When the condition belongs to one field, give that field a type that checks
   itself with its own constructor and `serde(try_from = ...)`; every struct
   holding it is then safe, and the outer constructor no longer refuses. A
   field attribute `deserialize_with` that calls the check also routes it.
4. Write `impl Deserialize` by hand only when the data shape differs from both,
   and call the same constructor from it. Remove `Deserialize` when nothing
   reads the type from data.
5. Rerun the reported lint check and the affected behavior checks; data the
   constructor refuses must now fail to deserialize.

## Rule binding

The measurement flags a Rust struct deriving `Deserialize`, directly or as a
`derive` inside `cfg_attr`, when the same file holds a checking constructor: a
function without `self` in the struct's own `impl`, `impl TryFrom` or
`impl FromStr`, returning a `Result` or `Option` of `Self`, that refuses on a
parameter tested directly or through a local bound by `let` from it and stores it in a named or positional field without serde
`deserialize_with` or `with`. Serde `try_from` or `from` on the struct, a manual
`impl Deserialize` for it and test code leave no finding, and no reference
justifies one. A constructor in another file, a check inside a helper function
and enums stay unseen. Do not weaken the rule or its selectors, or hide the
check merely to pass. Apply complexity-discipline; stop when deserialization
goes through the same check as construction.
