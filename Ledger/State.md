# State

## Focus

P171 B176 entry-use and P172 B178 entry-bypass, delivered autonomously under consent. This VAC moves the pin to
the head whose resolver no longer reports a private module reached through its parent's reexport, so the
installed lint can judge the rig with the entry rules on.

## Workspace

Branch: feature/entry-boundaries
Revision: 452a44bb12826bce4bb9b23c707237d521472c29

## Progress

- The rule enabling change, rig lint.yaml rules, init catalog, the Python fixture and the D227 and D228 links,
  waits in the stash vac22-enable-entry-rules until this pin is committed.

## Verification

- The installed lint at the old pin dae94a4 reported 13 entry-bypass findings that the candidate does not report.
- propose-pin HEAD under consent: the branch gate passed, the runtime 452a44b is installed and stable.txt written.

## Blockers

- The checks.yaml impact-map proposal in .cache/runtime/proposal waits for the user approve.

## Next action

Commit the pin, pop the stash, rerun the installed lint and commit the enabled entry rules.
