---
name: use-the-current-form
description: Replace a Rust form the project's toolchain supersedes with its direct standard form, a strip_prefix or strip_suffix falling back to its receiver with trim_prefix or trim_suffix, Infallible with ! and a Vec collected from a map over 0..n with Vec::from_fn, or record why the old form stays.
---

# Write the form the toolchain now provides

Older Rust spelled some intents through a combination of calls:
`x.strip_prefix(p).unwrap_or(x)` removes an optional prefix, `Infallible`
names an error that cannot occur and `(0..n).map(f).collect::<Vec<_>>()`
builds a vector from its indices. The toolchain the project pins states each
intent directly in one call or name. The direct form cannot fall back to a value
other than the receiver and reads the same in every codebase, so an agent or a
reviewer recognizes it at once.

1. Replace the form the finding names:
   - `x.strip_prefix(p).unwrap_or(x)`, also with `&x`, by `x.trim_prefix(p)`,
     and the `strip_suffix` form by `x.trim_suffix(p)`; both exist on `str`,
     slices and `Path`;
   - `Infallible` by `!`; a `Result<T, !>` then unpacks with
     `let Ok(value) = result;`, and the `Infallible` import goes;
   - `(0..n).map(f).collect::<Vec<_>>()`, or the same collect bound by
     `let v: Vec<_>`, by `Vec::from_fn(n, f)`, whose `f` takes each index as
     `usize`.
2. When the old form must stay, such as indices of another integer type than
   `usize`, record why as an owned `DECISION:`/`INVARIANT:` reference on the
   line directly above it.
3. Rerun the reported lint check and the affected behavior checks.

## Rule binding

The measurement flags every Rust `.unwrap_or` call whose single argument, or
the operand of `&` in it, has the text of the receiver of the `.strip_prefix` or
`.strip_suffix` call it is chained to, every `Infallible` type name, also as the
last segment of a path, and every `.collect()` of a `.map(f)` over a
parenthesized `0..n` or `0usize..n` whose turbofish or `let` type is `Vec`, test
code included, outside macro arguments, without a `DECISION:`/`INVARIANT:`
reference on the line directly above it. Types are not resolved, so a method of
the same name on another type counts. A fallback other than the receiver, an
inclusive range, another start and an inferred or other collection are not
flagged. Do not weaken the rule or its selectors merely to pass. Apply
complexity-discipline; stop when the current form replaces the old one or an
owned reference justifies keeping it.
