---
name: type-driven-discipline
description: Close a named invariant, mode or lifecycle in a type checked once at the boundary, so consumers cannot re-check, forget or violate it.
---

# Make the type carry the invariant

A type earns its definition by closing a named invariant: a condition that
would otherwise be re-checked, forgotten or silently violated downstream. Each
rule below names the invariant it closes. Where no invariant exists, the plain
value is correct and the type would be ceremony.

1. A value with a validity condition is a newtype with a private field and one
   entry: `TryFrom`, `FromStr` or a `parse`/`new` returning `Result`. The
   constructor is the only place that checks; consumers take the type and never
   re-validate. The same predicate re-checked across functions is the symptom.
2. Parse once at the boundary, then branch on an `enum`. A string, integer or
   set of flags that stands for a closed set of modes becomes an `enum` as soon
   as a second site compares it; `match` it exhaustively, without a `_` arm, so
   a new variant fails to compile at every site that must care. Vocabularies of
   the outside world (wire protocols, foreign registries, file formats) are
   decoded once as data, not spread through `match` arms.
3. Required fields arrive through a constructor. Do not reach a valid object
   through `Default` plus setters or through `Option<T>` fields that are
   "always `Some`" after setup. Staged construction is lawful while no invalid
   object is live: a builder ending in one validating `build`, or a lifecycle
   whose each step returns the next type.
4. Encode an operation order in types only where it forbids a dangerous
   operation: `fn step(self) -> Next` consumes the earlier state so the misuse
   cannot be written. Ordinary sequencing needs no typestate.
5. Keep `pub` fields for a record without an invariant between its fields. A
   field that a constructor validates, or that another field depends on, is
   private behind an accessor; a record of independent values stays open.

## Related forms

- One vocabulary, one definition: a `match` that re-lists the strings a parser
  accepts, or a message that enumerates the variants, drifts from its source.
  Derive or generate the second site from the first.
- A static table whose order matters is asserted sorted by
  `const _: () = assert!(..)` or a test, not by convention.
- Snapshot and restore go through `Clone`, `mem::replace` or `mem::take` of the
  whole value, never field-by-field copies that miss the field added later.
- Filesystem paths are `&Path`/`PathBuf`, not strings; durations are
  `Duration`, not bare integers.
- A chain of `else if x == LITERAL` over one value, and a `match` on an integer
  mirroring enum discriminants, are an enum in disguise: decode once with
  explicit discriminants and match the enum.
- Two literal tables that invert each other (`name -> variant`,
  `variant -> name`) are one derive or one exhaustive `match`, so the sets
  cannot diverge.
- A tuple variant carrying several primitives is a struct variant with named
  fields.
- Comparing variants by ordinal (`a as u8 <= b as u8`) is a derived `Ord` with
  the documented order, or a method naming the relation.
- A `macro_rules!` that hides control flow or generates items carries a
  `// DECISION:` reference naming why a function or generic does not suffice.
- A shim that only forwards is removed unless it isolates a real boundary
  (platform, visibility, trait object).

## Enforcement

These are review norms. The syntactic engine cannot see a type's invariant,
a constructor's relation to its fields or the second site of a vocabulary: the
string-as-enum probe reached 62.5 % precision against the 90 % rule bar, and
the newtype and typestate forms need the type resolution the engine omits. The
decidable slices already ship as rules: `boolean-parameter`, `tuple-as-record`
and `global-state`. The rest apply in review, where a finding is repaired here.
Apply complexity-discipline: a type that closes no named invariant is not
required.
