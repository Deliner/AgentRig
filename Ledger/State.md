# State

## Focus

Пользователь 2026-11-10 задал стандарт рига, записан в AGENTS.md и D151. По нему показаны карточки B102–B105, и
пользователь поручил: «Все карточки подтверждаю, заводи и делай автономно». Выбрано P110 → P113; P110–P112 слиты,
P113 (B105) завершается этим VAC. Пользователь одобрил карточку B106 и поручил запланировать её следом: выбрана P114
после P113. B107 о сверке имён узлов с грамматикой сохранена по поручению пользователя, но в работу не взята: она
требует дополнительного согласования, пользователь вернётся к ней сам.

## Workspace

Branch: `feature/structural-bans`, от master `3508e36`.
Revision: `4fe150758fdb00e9563b8f50f30ff625fb2d652e` (пин середины P113 поверх фазы 1 `4122a4f`); этот снимок входит в
коммит фазы 2 поверх неё.

Установленный рантайм `4122a4f` (пин середины P113). macOS: cargo, кандидат, коммиты и `feature-merge` через
`just scratch -- 'export PATH=/Users/admin/.cargo/bin:$PATH && …'`. Ветки прежних поставок сохранены,
`feature/init-probe` не трогать.

## Progress

- P110, P111 и P112 слиты в master; последний merge `3508e36`, пин P112 `a5a6af6`.
- voxel-rust: итоги ноутбука для P022 записаны коммитом `0eafc8e` и слиты в main `37de3ff`; P022 там paused.
- P113: VAC A `4a5c4b0`, фаза 1 `4122a4f` с D155, пин середины `4fe1507`. Фаза 2 в этом коммите: `unlabeled-exit`,
  `length-subtraction`, `implicit-order` и `unit-lock` включены в `lint.yaml`, `catalog::ALL`, реестр Rust и `init`;
  навыки label-the-exit, subtract-checked, pin-the-order и lock-the-data; находки рига исправлены, пять меток циклов и
  `split_last` в `paths.rs`. P113 и B105 перенесены в Archive, Delivery с таблицей замера и причинами отказа в
  `Archive/Plan/113.md`.

## Verification

- Фаза 2 до коммита: `just candidate lint` без находок, rust-test 178 PASS, Python-тесты config, project_setup и
  structural_lint 144 PASS, Clippy и rustfmt PASS, `config-check` PASS. Проба `init` кандидата: четыре правила в
  `lint.yaml`, четыре навыка в `.agentrig/skills`, `config-check` PASS, пробный файл даёт по находке каждого вида.

## Blockers

Нет.

## Next action

Финальный пин D051: `git rev-parse HEAD > tooling/distribution/stable.txt && just bootstrap`, коммит пина с хешем в
Delivery P113, затем `just feature-merge`. После слияния разметить списки запретов `list-2-bans.md` и `list-3-bans.md`
итогами замера и начать P114 своей веткой от master. Свободные ID: D156+, I022+, P115+, B108+ (B066 и D111
зарезервированы).
