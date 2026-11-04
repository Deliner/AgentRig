# State

## Focus

**Программа реализации находок исследовательской дуги (семь раундов) через догфудинг.**
Пользователь поручил (/goal, 2026-11-04) реализовать все находки последовательно и для каждой
убедиться, что она работает. Карточки B052–B064 (B064 — идея, не выбрана), дополнение B049 (AC2);
выбраны в Plan **P055–P068** (порядок — в Plan.md). P055 (`33477fd`), P056 (`8c3142c`), P057 (`fbb0a06`),
P058 (`3d23588`), P059 (`a2dc929`), P060 (`f33d418`) слиты. **P061** (B055, `kind: recursive-deletion`,
[D102](Decisions/102.md)) завершается этим VAC-B2; следующая карточка — P062 (B046, расширение
`unsafe-site-justification`).

## Workspace

Ветка `feature/recursive-deletion` (база master `f33d418`), наблюдаемый HEAD `aced8ec` (мид-бранч пин на VAC-B1
`30fd2a9`); весь VAC-B2 в рабочем дереве (переносы уже в индексе) и коммитится этим VAC. macOS: коммиты/cargo —
через `just scratch` с восстановленным PATH (`/Users/admin/.cargo/bin`), манифест — `Project/Cargo.toml`; догфуд —
`tooling/worker/run run candidate -- lint` внутри `just scratch`.

## Progress

- VAC-B2 (этот коммит): `catalog::ALL` и правила Rust в `languages/registry.rs` += `RecursiveDeletion`; правило
  `recursive-deletion` в `.agentrig/lint.yaml` (исключения `**/tests/**`, `**/tests.rs`); новая группа навыков
  `skills/ownership` (`own-the-resource` и `own-the-state` перенесены из `references`, добавлен
  `contain-the-deletion`; плоские ссылки, `assets.rs`, контракты `ownership`/`references`/`skills`, ссылки
  D092/D096); обоснования `// DECISION: D102` над двумя сайтами `command_runner/scratch.rs`; ротация P061 →
  `Archive/Plan/061.md` с Delivery, B055 → `Archive/Backlog/055.md`, строки индексов, примечание Plan.md, ссылки
  владельцев в строке D102.
- Не сделано: финальный пин D051 (stable.txt → голова ветки + bootstrap, отдельный коммит), `just feature-merge`
  (ветку сохранить, не пушить); далее P062–P068, по одной карточке за ветку.

## Verification

- 2026-11-04 на рабочем дереве: `cargo fmt --check` OK, `cargo clippy --all-targets -D warnings` OK, `cargo test
  --lib` — 122 passed; `just lint-config-check` OK, `just config-check` OK; догфуд кандидата — 0 находок (exit 0);
  `lint-rule recursive-deletion` (кандидат) описывает вид (file, навык `contain-the-deletion`, Rust); живая проба
  `fn probe` с `std::fs::remove_dir_all(root)` в `shared/vcs/git.rs` → `ERROR [recursive-deletion] … git.rs:247
  (remove_dir_all)` с навыком `contain-the-deletion`; после отката — 0 находок.
- Гейт этого VAC ещё не запускался (коммит — следующий шаг).

## Blockers

Нет.

## Next action

Закоммитить VAC-B2 через гейт (`git add Ledger Project .agentrig`), затем финальный пин D051
(`git rev-parse HEAD > tooling/distribution/stable.txt` + bootstrap, коммит «P061 D051 pin…»), `just feature-merge`
(ветку сохранить, не пушить). Далее `just feature-start` для P062 (B046). Свободные ID: D103+, I022+, P069+, B065+.
