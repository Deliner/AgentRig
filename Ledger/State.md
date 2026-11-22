# State

## Focus

P198 (B205): риг без режима ratchet. VAC 2 удаляет ratchet из рантайма, `init`, защиты, гейта, хука, документации
и навыка; решение D235.

## Workspace

Branch: feature/lint-ratchet-removal
Revision: 0ef0631585c1f7586a6e4138d1ddbdd172961f44

## Progress

VAC 1 `0ef0631`: P198 выбран. VAC 2 в рабочем дереве: удалены команда `lint-baseline`, модуль baseline, его тесты и
`Repository::renames`; защита и гейт без baseline; `init` и мастер называют число ошибок без предложения принять их;
тесты AC1–AC4 в `layout.rs`, `portable.rs`, `approval.rs`, `protected.rs`; D235 и ссылки индекса.

## Verification

PASS: `memory`, `build`, `clippy`, `candidate-lint`, `lint`. Тесты `project_setup::`, `agent_hooks::`,
`change_delivery::git::tests::approval`, `structural_lint::`: 465 прошли; после одобренного пользователем
`.agentrig/just/lint.just` без `lint-baseline` прошёл и `init_matches_the_rig_setup_except_listed_differences`.

## Blockers

Нет.

## Next action

Закоммитить VAC 2 с одобренным `lint.just`, затем `just propose-pin HEAD`.
