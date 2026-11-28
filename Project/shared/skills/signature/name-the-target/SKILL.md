---
name: name-the-target
description: Write the target type of a parse, into, try_into, sum, product, collect, Default::default or From::from bound by a let without a type, call a conversion written through its trait path, such as From::from, FromStr::from_str or Default::default, on its target type, move the conversion into its use, or record why a later use should choose the type.
---

# Name the type a conversion produces

`let port = text.parse()?;`, `let items = iter.collect();` and
`let name = raw.into();` leave the target type to inference. The compiler takes
it from a later use: a function parameter, a struct field, the return type or a
comparison. When that use changes, the conversion silently changes with it: a
`parse` accepts another syntax, a `collect` builds another collection that may
drop duplicates or order, an `into` runs another `From` implementation. The
line shows neither what it produces nor what it accepts. A conversion called
through its trait path, such as `From::from(raw)`, `.map(Into::into)` or
`FromStr::from_str(text)`, leaves the target to inference the same way wherever
it stands.

1. Find the type the later use requires.
2. Write it on the binding, `let items: Vec<PathBuf> = iter.collect();`, or as
   a turbofish, `text.parse::<u16>()?` or `iter.collect::<BTreeSet<_>>()`. A
   conversion that names its target, such as `PathBuf::from(raw)` or
   `path.to_path_buf()`, serves as well.
3. Call a conversion written through its trait path on the target type:
   `Target::from(value)`, `Target::try_from(value)?`,
   `text.parse::<Target>()` or `.map(Target::from)`.
4. When the value goes straight into its use, move the conversion into the
   field or argument expression, where the use itself fixes the type.
5. When the later use is meant to choose the type, as in generic code that
   forwards it, record that as a decision with edit-decisions and put its
   exception `// DECISION: DNNN RULE`, RULE being the rule ID the finding
   names, alone on the line directly above the reported line. List it in the
   decision as `[exception RULE](FILE)`; the user approves a new exception
   through propose-protected-change.
6. Rerun the reported lint check and the affected behavior checks.

## Clippy profile

The rig's Clippy profile denies `clippy::default_trait_access` in the library,
binaries and examples of the configured Cargo manifest; tests are outside the
profile. Clippy reports a `Default::default()` call of a struct, enum or union
type wherever it stands, also with a typed `let`, as a field value or as an
argument, and the lint check reports each message as an error naming this
skill. Call it on the type: `Config::default()`. The named call is the same
call, so no exception keeps the trait path; do not suppress the lint or weaken
the profile merely to pass.

## Rule binding

The measurement flags every Rust `let` of a plain name without a type
annotation whose value, through `?`, is a `.parse()`, `.into()`,
`.try_into()`, `.sum()`, `.product()` or `.collect()` call without arguments
or turbofish, a `Default::default()` call or a `From::from(..)` call. An
`.into()` on a plain parameter name of the nearest `fn` is skipped when the
parameter's type is an `impl` with an `Into` bound or a generic parameter of
that `fn`, or of the nearest `impl` or trait, bounded by `Into` in its list or
`where` clause: the bound already names the target. Test code is not measured,
and an exception naming `untyped-conversion`, the default ID, lifts the
finding. Types are not resolved, so a method of the same name that fixes its
own return type also counts.

The trait path call measurement flags every Rust call or path value
`From::from`, `Into::into`, `TryFrom::try_from`, `TryInto::try_into` or
`FromStr::from_str` whose trait is a single segment or follows `std::convert`,
`core::convert`, `std::str` or `core::str`, without type arguments, outside test
code; an exception naming `trait-path-call`, the default ID, lifts the finding.
A call inside the value of a `let` with a type is skipped, and a `From::from`
call that, through `?`, is the whole value of a `let` of a plain name without a
type is reported by the measurement above instead.

An exception lifts a finding only from the line directly above the reported
line, and only the rule it names; a bare `DECISION:` or `INVARIANT:` reference
lifts nothing. Rerun the reported lint check and relevant behavior checks. Do
not weaken the rule or its selectors merely to pass. Apply
complexity-discipline; stop when the target type is written or fixed by its
use, or an approved exception lifts the finding.
