# State

## Focus

Пользователь 2026-11-10 задал стандарт рига, записан в AGENTS.md и D151. По нему показаны карточки B102–B105, и
пользователь поручил: «Все карточки подтверждаю, заводи и делай автономно». P110–P113 слиты в master. Пользователь
одобрил карточку B106 и поручил запланировать её следом: P114 активна. B107 о сверке имён узлов с грамматикой
сохранена по поручению пользователя, но в работу не взята: она требует дополнительного согласования, пользователь
вернётся к ней сам.

2026-11-11 пользователь поручил после P114 автономно перенести тесты рига с Python на Rust: быстрее, только через
публичные интерфейсы, тесты внутри проверяемых фич; где хватает Rust-теста, Python-тест не нужен. Ок дан заранее на
всё, что для этого нужно.

## Workspace

Branch: `feature/structural-bans-2`, от master `ba71c19`.
Revision: `ff47847cc3609d853361cfaa947e64ce71eb1cbb` (старт P114); этот снимок входит в VAC фазы 1 поверх неё.

Установленный рантайм `d97417e` (финальный пин P113). macOS: cargo, кандидат, коммиты и `feature-merge` через
`just scratch -- 'export PATH=/Users/admin/.cargo/bin:$PATH && …'`; Python-тесты кандидата с
`WORKER_BINARY=.cache/worker/release/agentrig`. Ветки прежних поставок сохранены, `feature/init-probe` не трогать.

## Progress

- P110–P113 слиты в master, последний merge `ba71c19`, пин P113 `f7bdadf`.
- P114 фаза 1: замер по протоколу выполнен, решение D156 записано. Поставляются 14 видов и параметр `junctions` у
  `boolean-expression` (II-7), параметр `pairs` у `paired-calls`; 7 форм отвергнуты. Виды есть в коде с описаниями и
  тестами, но не включены: их нет в `lint.yaml`, `catalog::ALL` и реестре, пример `boolean-expression` ещё без
  `junctions: single`. Тесты языков разложены по `control_flow/`, `reference/`, `signature/`, `ownership/`.
- Замер II-7 в поставляемом объёме, Rust и Python вместе с тестами, дал те же 16 мест рига и 5 мест voxel.
- voxel-rust: P022 там paused, итоги ноутбука слиты в main `37de3ff`.

## Verification

- Для содержимого этого VAC: Rust lib-тесты 201 из 201, Clippy чистый, `just candidate lint` без ошибок, Python-тесты
  `Project/structural_lint` и `Project/project_setup/tests` 271 из 271, ruff, ruff-format и mypy чистые.

## Blockers

Нет.

## Next action

Закоммитить фазу 1, записать её голову в stable.txt с bootstrap и закоммитить пин. Затем фаза 2: включить виды и
`junctions: single`, навыки, `TEST_CODE_EXCLUDED`, ремонт находок рига, проба init, повторный замер, ротация, пин и
`feature-merge`. Свободные ID: D157+, I022+, P115+, B108+ (B066 и D111 зарезервированы).
