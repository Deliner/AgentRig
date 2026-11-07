---
name: redact-secret
description: Keep a secret field of a Rust struct out of a derived Debug by wrapping it in a type whose Debug prints a placeholder, or by writing Debug by hand without the value.
---

# Redact the secret from Debug

A derived `Debug` prints every field in full. Wherever the struct reaches
`{:?}`, a log or tracing macro, a failed `assert!`, `unwrap` or `expect`, or a
panic message, the secret lands in logs, terminals and crash reports read by
more people than the secret was meant for.

1. Confirm what the field holds. A real password, token, key or card code is
   a secret. A field that only shares the name, such as a lexer `token`, gets
   a name that says what it holds; a real secret is never renamed to pass.
2. Prefer a type that owns the redaction: a redacting type the project already
   depends on, such as `secrecy::SecretString`, or a small newtype whose
   `Debug` writes a fixed placeholder and whose value is reached only through
   an explicit method at the one place that needs it:

   ```rust
   pub struct Password(String);
   impl fmt::Debug for Password {
       fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
           formatter.write_str("Password(<redacted>)")
       }
   }
   ```

   Every struct holding the type then stays redacted without its own code.
3. When no reusable type is wanted, remove `Debug` from the derive and write it
   by hand: `debug_struct` with every other field, then `finish_non_exhaustive`
   or a placeholder for the secret. Keep `Debug` itself when consumers print
   the struct.
4. Rerun the reported lint check and the affected behavior checks.

## Rule binding

The measurement flags every named field of a Rust struct deriving `Debug`,
directly or as a `derive` inside `cfg_attr`, whose name is exactly
`password`, `passwd`, `secret`, `token`, `api_key`, `apikey`, `private_key`,
`access_token`, `refresh_token`, `client_secret`, `jwt` or `cvv` and whose type
is `String` under any path, `&str`, `Vec<u8>` or an `Option` of one of them.
Any other field type, a manual `Debug`, tuple structs, enum variants and test
code are out of scope, and no reference justifies a finding. Rerun the
reported lint check and relevant behavior checks. Do not weaken the rule or
its selectors, or rename a real secret, merely to pass. Apply
complexity-discipline; stop when the secret no longer reaches a derived
`Debug`.
