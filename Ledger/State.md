# State

## Focus

Пользователь 2026-11-10 поручил: «Сохрани показанные карточки, заведи пункт вот по этому по линту, который ты
написал, и все карточки, и вообще все карточки в бэклоге, которые есть, автономно реализуй сам. У меня ничего не
спрашиваю, я на все даю подтверждение». Карточки B091, B094–B101 и правка B089 сохранены в `4518103`. Все карточки
Backlog B086–B101 выбраны в Plan как P094–P109 и выполняются автономно по одной за ветку; решения, которые карточки
оставляли пользователю, исполнитель принимает сам и записывает. Риг дорабатывается для себя через докфудинг и
проверяется на voxel-rust; под voxel-rust он не подгоняется.

## Workspace

Branch: `feature/revision-gate` (P094, B093)
Revision: `cca748c` (VAC A; завершающий VAC с ротацией ещё не закоммичен)

Установленный рантайм `9ba4fad`. macOS: коммиты и `feature-merge` с cargo через
`just scratch -- 'export PATH=/Users/admin/.cargo/bin:$PATH && cd /Users/admin/projects/AgentRig && …'`.
Ветки прежних поставок сохранены, `feature/init-probe` не трогать.
voxel-rust: ветка `feature/agentrig-migration`, коммит `71440d0`, рабочее дерево чистое, в main не слита, push нет.
В memory/State.md voxel-rust число тестовых модулей 21 неверно: живой счёт 13 файлов `*_tests.rs`.

## Progress

- Выбор P094–P109 слит в master `e9fc137`.
- P094 VAC A: проверка `tools` под `check --revision` берёт файлы из дерева ревизии и Git исходного репозитория;
  `feature-merge` после отказанного `git switch` возвращает индекс и дерево к HEAD.

## Verification

- Установленный рантайм `9ba4fad`: `just check --revision HEAD --only tools` — «not a git repository», код 2.
- Кандидат: тот же запуск на AgentRig и на voxel-rust — PASS [tools].
- Новый тест отказанного переключения падал до правки со смешанным деревом (` M .agentrig/hooks/reference-transaction`).
- Гейт коммита `cca748c`: все проверки PASS, rust-test 11 и test 136 passed.
- В рабочем дереве: Delivery P094 и ротация P094/B093 в Archive.

## Blockers

Нет.

## Next action

Закоммитить ротацию; пин D051 (`git rev-parse HEAD > tooling/distribution/stable.txt && just bootstrap`), коммит пина,
`just feature-merge`; затем P095 (B098).
Свободные ID: D139+, I022+, P110+, B102+ (B066 и D111 зарезервированы).
