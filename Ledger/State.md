# State

## Focus

**Программа реализации находок исследовательской дуги (семь раундов) через догфудинг.**
Пользователь поручил (/goal, 2026-11-04) реализовать все находки последовательно и для каждой
убедиться, что она работает. Карточки B052–B064 (B064 — идея, не выбрана), дополнение B049 (AC2);
выбраны в Plan **P055–P068**. P055–P067 слиты (последний merge `0e9fb8b`).
**P068** (B063, нормы N1/N2 и вердикты дуги): VAC-A (AC1, AC2) — этот коммит: навыки
`type-driven-discipline` и `fail-closed-enforcement` в группе `Project/shared/skills/discipline`
(туда же перенесён `testing-discipline`; плоские симлинки сохраняют прежние пути), поставка
через `assets.rs`, фраза N1 в AGENTS.md репозитория и в поставляемом, рекомендательный маршрут
в live и поставляемом hooks.yaml. Далее VAC-B: решение D110 с вердиктами, строки N1/N2 в
Norms.md, ротация P068/B063; затем пин D051 и `just feature-merge`.

## Workspace

Ветка `feature/norms-verdict` (база master `0e9fb8b`), наблюдаемый HEAD `0e9fb8b` (коммитов на
ветке нет); stable.txt → `a8fbebf`. macOS: коммиты/cargo — через `just scratch` с восстановленным
PATH (`/Users/admin/.cargo/bin`), манифест — `Project/Cargo.toml`; профиль clippy гейта —
`tooling/worker/run run clippy`; догфуд — `tooling/worker/run run candidate -- lint`; pytest
одного файла — `tooling/worker/run run test -- <файл>`; проба хука —
`tooling/worker/run hook --root <корень>` с JSON PreToolUse на stdin.

## Progress

- VAC-A (этот коммит, AC1/AC2): группа `discipline/` (architecture.yaml с тремя листами) —
  лимит `directory-size` корня навыков (15) не допускал двух новых директорий;
  `testing-discipline` перенесён в группу (`git mv`), симлинки `testing-discipline`,
  `type-driven-discipline`, `fail-closed-enforcement` в корне навыков сохраняют пути
  `.agents/skills/<имя>/SKILL.md`. `assets.rs`: две записи SKILLS + новый путь
  testing-discipline; `instructions()` и AGENTS.md: «Apply type-driven-discipline when a value
  carries an invariant, a mode or a lifecycle». Маршрут без `block` на `Project/agent_hooks/**`,
  `Project/change_delivery/**` → `fail-closed-enforcement/SKILL.md` (`events: [edit]`);
  в `template.rs` такой же маршрут на `{source}/**` (I020: поставка == live), цикл маршрутов
  памяти извлечён в `memory_routes` (лимит 40 строк).
- Не сделано: VAC-B (D110, Norms N1/N2, ротация Plan/068 → Archive, Backlog/063 → Archive,
  заметка в Plan.md), пин D051, `just feature-merge`.

## Verification

- 2026-11-05, рабочее дерево VAC-A: `cargo fmt --check` чисто; `run clippy` exit 0;
  `--all-targets` чисто; догфуд `candidate -- lint` — 0 находок (инвентарь, `directory-size` 15);
  `candidate -- memory-check` exit 0; `config-check` — конфигурация валидна; `cargo test --lib`
  128 passed; pytest `test_delivery.py` (I020: поставка == live по навыкам и маршрутам) +
  `project_setup/tests` + `agent_hooks` — 58 passed. Проба хука: Edit `Project/agent_hooks/guard.rs`
  → Required refactor-large-directory + «Guidance: … fail-closed-enforcement/SKILL.md …
  hooks.routes[8]»; Edit `Project/project_setup/template.rs` → только Required, без упоминания
  навыка. Гейт коммита VAC-A — следующий шаг.

## Blockers

Нет.

## Next action

Закоммитить VAC-A через гейт («P068 VAC-A: ship type-driven and fail-closed norm skills (B063
AC1, AC2)»); затем VAC-B: `Ledger/Decisions/110.md` + строка индекса (вердикты по списку карточки
B063 AC3; Applies-in: карточка, оба навыка, AGENTS.md, `assets.rs` и `template.rs` с маркерами
`// DECISION: D110`, `hooks.yaml`), строки N1/N2 в `Ledger/Norms.md`, Delivery в Plan/068 и
ротация в Archive; затем пин D051 и `just feature-merge` (ветку сохранить).
Свободные ID: D110+, I022+, P069+, B065+.
