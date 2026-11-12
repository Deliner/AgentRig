---
name: name-the-target
description: Write the target type of a parse, into, try_into, sum, product, collect, Default::default or From::from bound by a let without a type, move the conversion into its use, or record why a later use should choose the type.
---

# Name the type a conversion produces

`let port = text.parse()?;`, `let items = iter.collect();` and
`let name = raw.into();` leave the target type to inference. The compiler takes
it from a later use: a function parameter, a struct field, the return type or a
comparison. When that use changes, the conversion silently changes with it: a
`parse` accepts another syntax, a `collect` builds another collection that may
drop duplicates or order, an `into` runs another `From` implementation. The
line shows neither what it produces nor what it accepts.

1. Find the type the later use requires.
2. Write it on the binding, `let items: Vec<PathBuf> = iter.collect();`, or as
   a turbofish, `text.parse::<u16>()?` or `iter.collect::<BTreeSet<_>>()`. A
   conversion that names its target, such as `PathBuf::from(raw)` or
   `path.to_path_buf()`, serves as well.
3. When the value goes straight into its use, move the conversion into the
   field or argument expression, where the use itself fixes the type.
4. When the later use is meant to choose the type, as in generic code that
   forwards it, record that as an owned `DECISION:`/`INVARIANT:` reference on
   the line directly above the `let`.
5. Rerun the reported lint check and the affected behavior checks.

## Rule binding

The measurement flags every Rust `let` of a plain name without a type
annotation whose value, through `?`, is a `.parse()`, `.into()`,
`.try_into()`, `.sum()`, `.product()` or `.collect()` call without arguments
or turbofish, a `Default::default()` call or a `From::from(..)` call. An
`.into()` on a plain parameter name of the nearest `fn` is skipped when the
parameter's type is an `impl` with an `Into` bound or a generic parameter of
that `fn`, or of the nearest `impl` or trait, bounded by `Into` in its list or
`where` clause: the bound already names the target. Test code and a `let` with
a `DECISION:` or `INVARIANT:` reference on the line directly above it are not
measured. Types are not resolved, so a method of the same name that fixes its
own return type also counts. Rerun the reported lint check and relevant
behavior checks. Do not weaken the rule or its selectors merely to pass. Apply
complexity-discipline; stop when the target type is written or fixed by its
use, or the inferred type is justified by an owned reference.
