# State

## Focus

Пользователь 2026-11-10 решил по наблюдениям из перевода voxel-rust: тестовые файлы Rust проекты приводят к
соглашению AgentRig, это B091; architecture.yaml не считается в лимите записей каталога; `init` сразу исключает
lock-файлы из file-size; заводятся карточки о рёбрах через трейты, о panic-path в телах `macro_rules!` с
исследованием, о job-cleanup, о двойных маркерах DECISION и о сообщении «never imported»; «`src/**` не выбирает сам
`src`» описывается в README. Черновики карточек показываются в чате и сохраняются только после согласия.
Риг дорабатывается для себя через докфудинг и проверяется на voxel-rust; под voxel-rust он не подгоняется.

## Workspace

Branch: `feature/readme-directory-globs`
Revision: `cb5343f` (master; VAC с README и этим снимком ещё не закоммичен)

Установленный рантайм `5ad3707`. macOS: коммиты и `feature-merge` с cargo через
`just scratch -- 'export PATH=/Users/admin/.cargo/bin:$PATH && cd /Users/admin/projects/AgentRig && …'`.
Ветки `feature/backlog-voxel-observations`, `feature/coverage-include`, `feature/rust-expansion`,
`feature/option-option`, `feature/init-probe` и прежние сохранены.
voxel-rust: ветка `feature/agentrig-migration`, коммит `71440d0`, рабочее дерево чистое, в main не слита, push нет.

## Progress

- B086–B090, B092 и B093 сохранены и слиты в master `cb5343f`. B091 зарезервирована, её надо переписать под
  соглашение AgentRig; строка «Зависит от B091» в B089 тогда меняется с согласия пользователя.
- Project/README.md: абзац о шаблонах путей объясняет, что `src/**` не выбирает сам `src`. В рабочем дереве.
- Двойные маркеры выяснены: panic-path требует маркер над вызовом `.expect()`, allow-justification над
  `#[allow(clippy::expect_used)]`, а общий профиль Clippy запрещает `expect_used`. Одна паника требует двух ссылок.
- Пункт о трейтах пришёл из слепой зоны собственного оракула зависимых, а не из находки рига; предел рига записан в
  D059.

## Verification

- Изменение README ещё не проходило гейт коммита.

## Blockers

voxel-rust: слияние в main ждёт решения пользователя о платформе рантайма, B086. Не запускать там `feature-merge`,
`feature-start` и `git switch main`.

## Next action

Закоммитить README, записать голову ветки в stable.txt с bootstrap по D051, закоммитить пин и выполнить
`just feature-merge`. Затем показать пользователю черновики карточек.
Свободные ID: D139+, I022+, P094+, B094+ (B066, B091 и D111 зарезервированы).
