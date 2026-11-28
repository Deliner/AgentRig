---
name: name-the-states
description: Replace an Option<bool>, an Option<Option<..>> or several bool fields of a struct with enums that name each state, or record why an external format fixes the type.
---

# Name each state of an optional flag or a nested option

`Option<bool>` holds three states, `None`, `Some(false)` and `Some(true)`, and
`Option<Option<T>>` holds absent, empty and present; neither names them. A reader
has to guess what `None` means, and `unwrap_or(false)` or `flatten()` silently
merges two of the states.

1. Read where the value is produced and consumed, and establish what each state
   means, such as not asked yet, refused and granted, or keep, clear and set.
2. Declare an enum with one variant per state, such as
   `enum Consent { Unasked, Refused, Granted }` or
   `enum Patch<T> { Keep, Clear, Set(T) }`, and replace the type and every
   construction and match.
3. Where a consumer merged two states, decide the merge explicitly in a `match`
   arm or a named method, so that it is visible.
4. When a data type mirrors an external format, such as a JSON field that may be
   missing or null, deserialize into the enum where the format allows it;
   otherwise record the reasoning as a decision with edit-decisions and put
   its exception `// DECISION: DNNN RULE`, RULE being the rule ID the finding
   names, alone on the line directly above the reported line. List it in the
   decision as `[exception RULE](FILE)`; the user approves a new exception
   through propose-protected-change.
5. Rerun the reported lint check and the affected behavior checks.

## Clippy profile

The rig's Clippy profile denies `clippy::option_option` and
`clippy::struct_excessive_bools` in the library, binaries and examples of the
configured Cargo manifest; tests are outside the profile. The lint check
reports each Clippy message as an error naming this skill.

- `clippy::option_option` reports the `Option<Option<..>>` that the rule below
  leaves to Clippy; the steps above repair it.
- `clippy::struct_excessive_bools` reports a struct with more `bool` fields
  than the `max-struct-bools` setting of clippy.toml, which the rig sets to
  one. Flags of one struct usually encode modes or the stages of a lifecycle:
  replace flags that exclude each other with one enum whose variants name the
  combinations that occur, such as `enum Phase { Pending, Running, Done }`,
  and give a flag that varies on its own a two-variant enum, such as
  `enum Visibility { Shown, Hidden }`.

When an external format fixes the type, record the reasoning as a decision
with edit-decisions and put `#[expect(clippy::option_option)]` or
`#[expect(clippy::struct_excessive_bools)]` on the smallest item that holds
it, with its exception `// DECISION: DNNN allow-justification` alone on the
line directly above the attribute. List it in the decision as
`[exception allow-justification](FILE)`; the user approves a new exception
through propose-protected-change, as justify-allow describes. Do not weaken
the profile or the setting merely to pass.

## Rule binding

The measurement reports every Rust `Option<bool>` type, and every outermost
`Option<Option<..>>` type where the Clippy lint `option_option` does not check
it: a type alias, a `let` annotation, a closure, a `fn` pointer, a `dyn` or
`impl` trait, a bound or generic default, a turbofish, an impl header, an
associated type of a trait impl, and a member of an extern block. The
parameters, return type and `const` type of a trait impl member are left to the
trait declaration or the impl header, where that type is chosen. Clippy reports
the nested form in other signatures, fields, `const` and `static`; the repair is
the same. Test code is out of scope. An exception naming this rule,
`ambiguous-option` by default, on the line directly above the reported line
lifts the finding; a bare `DECISION:` or `INVARIANT:` reference lifts nothing.
Do not weaken the rule or its selectors merely to pass. Apply
complexity-discipline; stop when every state has a name or an approved
exception lifts the finding.
