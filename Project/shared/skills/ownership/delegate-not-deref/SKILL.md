---
name: delegate-not-deref
description: Replace Deref or DerefMut of a wrapper with methods that delegate only what callers need, so that the inner type's methods cannot break the wrapper's invariant, or record why the wrapper must deref.
---

# Expose only what the wrapper means

A wrapper such as `struct SortedVec(Vec<u32>)` or `struct UserId(String)` exists
to keep an invariant or an identity. `Deref` to the inner type hands out every
method of that type: `DerefMut` lets a caller `push` out of order, and `Deref`
lets string operations treat an id as text, so the wrapper stops meaning
anything.

1. Read the wrapper and its callers, and establish which inner methods callers
   actually use and which would break the invariant.
2. Remove the `Deref` and `DerefMut` impls.
3. Write the methods callers need on the wrapper, each delegating to the inner
   field, such as `fn len(&self) -> usize { self.0.len() }`; a mutating method
   keeps the invariant.
4. When callers need read access to the whole inner value, add a named method
   such as `fn as_slice(&self) -> &[u32]`, so that the escape is visible.
5. When the type is a smart pointer or a guard whose purpose is to stand for the
   pointee, record the reasoning as an owned `DECISION:`/`INVARIANT:` reference
   on the line directly above the impl.
6. Rerun the reported lint check and the affected behavior checks.

## Rule binding

The measurement reports every Rust impl of `Deref` or `DerefMut` for a struct of
the same file whose `Target` is not a type parameter of that struct. Test code
and an impl with a `DECISION:` or `INVARIANT:` reference on the line directly
above it are out of scope. Do not weaken the rule or its selectors merely to
pass. Apply complexity-discipline; stop when callers use delegating methods or
the deref is justified by an owned reference.
