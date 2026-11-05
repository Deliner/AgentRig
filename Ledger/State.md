# State

## Focus

**Программа реализации находок исследовательской дуги (семь раундов) через догфудинг.**
Пользователь поручил (/goal, 2026-11-04) реализовать все находки последовательно и для каждой
убедиться, что она работает. Карточки B052–B064 (B064 — идея, не выбрана), дополнение B049 (AC2);
выбраны в Plan **P055–P068**. P055–P067 слиты (последний merge `0e9fb8b`).
**P068** (B063): VAC-A `4c05c39` (AC1, AC2 — навыки N1/N2, фраза в AGENTS.md, advisory-маршрут);
VAC-B (AC3, AC4: решение D110 с вердиктами дуги, строки N1/N2 в Norms.md, маркеры D110 в
`assets.rs`/`template.rs`, ротация P068/B063 в Archive) завершается этим коммитом; далее пин D051
и `just feature-merge` — программа P055–P068 завершена; затем итоговый отчёт пользователю.

## Workspace

Ветка `feature/norms-verdict` (база master `0e9fb8b`), наблюдаемый HEAD `4c05c39` (VAC-A);
stable.txt → `a8fbebf`. macOS: коммиты/cargo — через `just scratch` с восстановленным PATH
(`/Users/admin/.cargo/bin`), манифест — `Project/Cargo.toml`; профиль clippy гейта —
`tooling/worker/run run clippy`; догфуд — `tooling/worker/run run candidate -- lint`; pytest
одного файла — `tooling/worker/run run test -- <файл>`; проба хука —
`tooling/worker/run hook --root <корень>` с JSON PreToolUse на stdin.

## Progress

- VAC-A (`4c05c39`, AC1/AC2): группа `Project/shared/skills/discipline/` (туда перенесён
  `testing-discipline`; плоские симлинки `testing-discipline`, `type-driven-discipline`,
  `fail-closed-enforcement` в корне навыков сохраняют пути `.agents/skills/<имя>/SKILL.md`;
  лимит `directory-size` корня 15 сохранён); `assets.rs`: две записи SKILLS + новый путь
  testing-discipline; `instructions()` и AGENTS.md: «Apply type-driven-discipline when a value
  carries an invariant, a mode or a lifecycle»; маршрут без `block` на `Project/agent_hooks/**`,
  `Project/change_delivery/**` → `fail-closed-enforcement/SKILL.md`; в `template.rs` такой же
  маршрут на `{source}/**` (I020), цикл маршрутов памяти извлечён в `memory_routes`.
- VAC-B (этот коммит, AC3/AC4): `Ledger/Decisions/110.md` + строка индекса (card →
  Archive/Backlog/063, оба навыка, AGENTS.md, `assets.rs` и `template.rs` с `// DECISION: D110`,
  `hooks.yaml`; related D093/D054/D059/D035/D097); `Ledger/Norms.md` — строки N1 и N2 (D110,
  навыки, I021). Ротация: Plan/068 → Archive/Plan/068 (Delivery с приёмкой AC1–AC4),
  Backlog/063 → Archive/Backlog/063 (без изменений), строки индексов Archive/Plan.md и
  Archive/Backlog.md, строки P068/B063 удалены из Plan.md/Backlog.md (заметка в Plan.md
  добавлена; Plan пуст — допустимо).
- Не сделано: пин D051 (меняется `Project`), `just feature-merge`.

## Verification

- 2026-11-05, VAC-A: гейт `4c05c39` прошёл (128 rust-test, 225 pytest). На том же дереве: fmt,
  clippy (и `--all-targets`) чисто, догфуд 0, memory-check 0, config valid, `cargo test --lib`
  128 passed, pytest `test_delivery.py` (I020) + `project_setup/tests` + `agent_hooks` — 58 passed;
  проба хука: Edit `Project/agent_hooks/guard.rs` → Guidance `fail-closed-enforcement/SKILL.md`
  (hooks.routes[8]) рядом с Required refactor-large-directory; Edit `project_setup/template.rs` →
  без упоминания навыка.
- 2026-11-05, рабочее дерево VAC-B: `candidate -- memory-check` exit 0 (D110 связан с маркерами
  и ссылками, ротация корректна); догфуд `candidate -- lint` — 0 находок; `cargo fmt --check`
  чисто; `run clippy` exit 0; `config-check` — конфигурация валидна. Гейт коммита VAC-B —
  следующий шаг.

## Blockers

Нет.

## Next action

Закоммитить VAC-B через гейт («P068 VAC-B: record D110 verdicts and norms N1/N2 (B063 AC3,
AC4)»); затем пин D051 (`git rev-parse HEAD > tooling/distribution/stable.txt` + bootstrap,
отдельный коммит) и `just feature-merge` (ветку сохранить); затем итоговый отчёт пользователю
по находкам дуги. Свободные ID: D111+, I022+, P069+, B065+.
