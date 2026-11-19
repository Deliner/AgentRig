# State

## Focus

P162 (B164): a lint kind reports `.filter(|_| flag)` on the result of a call, so the flag is checked before the call.
This VAC activates P162.

## Workspace

- Branch: feature/outer-flag-filter
- Revision: 3680f3afacfc2b8d9416e754ce17b54e77e8716b

## Progress

- P161 is merged; P162 is active in Plan.

## Verification

- Not run yet for P162.

## Blockers

None.

## Next action

Add the inert `outer-flag-filter` kind with its detector, tests and decision, then pin it and enable it with its skill.
