# State

## Focus

P183 (B190): archive the delivered parallel parse of the memory check.

## Workspace

Branch: feature/parallel-parse
Revision: ad2e73ed766018305987f7e6971c43266eb17d1a

## Progress

VAC 3 moves P183 and B190 into Archive with Delivery: VAC 1 `29f6da0` selected the card, VAC 2 `ad2e73e` parses the
sources of the decision and reference steps on several threads and checks them in the former order.

## Verification

Full suite at `ad2e73e`: 616 passed. `just lint`: clean. Decision step 143, 146 and 134 ms against 259, 256 and
257 ms; reference step 132, 151 and 133 ms against 161, 159 and 161 ms.

## Blockers

None.

## Next action

Run `just check --only memory`, commit the archive, then run `just feature-merge`.
