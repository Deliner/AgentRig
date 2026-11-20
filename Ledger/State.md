# State

## Focus

P168 (B179, D223): archive the delivered plan item and its card before the final pin and feature-merge.

## Workspace

Branch: feature/tests-directory
Revision: dc491bcccdbf91aadfa4790ac53dc9ec0fedc145

## Progress

- VAC 5 archives P168 and B179: index rows and cards move to Ledger/Archive, the Plan card gains Delivery with
  AC1–AC4 evidence, and the D223 row links to the archived card and delivery.
- `just propose-pin HEAD` prepared the pin dc491bcccdbf91aadfa4790ac53dc9ec0fedc145 for the user's approval.

## Verification

- `just check --only memory`: PASS on this archive change.
- Candidate lint on a temporary project with only `test-placement` reported exactly `jobs/tests.rs` and
  `jobs/test_cleanup.py`.
- The gate passed 594 tests on dc491bc.

## Blockers

The pin waits for the user to type `! just approve`.

## Next action

After the approval commit State and tooling/distribution/stable.txt, then run `just feature-merge` and start P169.
