---
name: derive-the-variants
description: Derive a list of every variant of an enum from the enum's declaration instead of writing it by hand, or record why the list is a selection that only happens to name every variant.
---

# Derive the list of variants from the enum

`const ALL: [Mode; 3] = [Mode::Fast, Mode::Slow, Mode::Idle];` is complete only
until `Mode` gains a variant. The compiler accepts the stale list, and every
loop, lookup or validation over it silently skips the new variant.

1. Establish what the list means: every variant of the enum, or a selection
   chosen by meaning, such as a display order or the variants a format
   supports, that names every variant only today.
2. When the project already depends on `strum`, derive the list:
   `#[derive(strum::VariantArray)]` gives `Mode::VARIANTS`, and
   `#[derive(strum::EnumIter)]` gives `Mode::iter()`. Add no dependency for
   one list.
3. Otherwise declare the enum and its list in one `macro_rules!` whose input
   names each variant once, such as
   `($($variant:ident),+ $(,)?) => { enum Mode { $($variant),+ } impl Mode { const ALL: &'static [Self] = &[$(Self::$variant),+]; } }`,
   so a variant added to the declaration joins the list.
4. When the enum also has variants with data, let the macro take the unit
   variants as its list and the other variants as they are, such as
   `($($unit:ident),+; $($other:tt)*) => { enum Color { $($unit,)+ $($other)* } ... }`.
5. When the list is a selection by meaning, record why as an owned
   `DECISION:`/`INVARIANT:` reference on the line directly above the array.
6. Rerun the reported lint check and the affected behavior checks.

## Rule binding

The measurement flags every Rust array literal, alone or under `&`, of three or
more paths `E::V`, or `Self::V` inside an impl of `E`, where `E` is an enum
declared in the same file and the paths name all of its unit variants in
declaration order, outside test code, without a `DECISION:`/`INVARIANT:`
reference on the line directly above the array. A partial, reordered or
repeated list, a list of fewer than three paths and an enum declared in another
file are not flagged; an enum with data variants is flagged when the list names
all of its unit variants. Do not weaken the rule or its selectors merely to
pass. Apply complexity-discipline; stop when the list follows from the enum's
declaration or an owned reference justifies the selection.
