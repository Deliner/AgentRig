# State

## Focus

Пользователь 2026-11-10 задал стандарт рига, записан в AGENTS.md и D151. По нему показаны карточки B102–B105, и
пользователь поручил: «Все карточки подтверждаю, заводи и делай автономно». Выбрано P110 → P113; P110–P112 слиты,
P113 (B105) завершается этим VAC. Пользователь одобрил карточку B106 и поручил запланировать её следом: выбрана P114
после P113. B107 о сверке имён узлов с грамматикой сохранена по поручению пользователя, но в работу не взята: она
требует дополнительного согласования, пользователь вернётся к ней сам.

## Workspace

Branch: `feature/structural-bans`, от master `3508e36`.
Revision: `d97417e81e5378ba1f939ef8b5bc2ba11e65db03` (фаза 2 P113); этот снимок входит в коммит финального пина поверх
неё.

Установленный рантайм `d97417e` (финальный пин P113). macOS: cargo, кандидат, коммиты и `feature-merge` через
`just scratch -- 'export PATH=/Users/admin/.cargo/bin:$PATH && …'`. Ветки прежних поставок сохранены,
`feature/init-probe` не трогать.

## Progress

- P110, P111 и P112 слиты в master; последний merge `3508e36`, пин P112 `a5a6af6`.
- voxel-rust: итоги ноутбука для P022 записаны коммитом `0eafc8e` и слиты в main `37de3ff`; P022 там paused.
- P113: VAC A `4a5c4b0`, фаза 1 `4122a4f` с D155, пин середины `4fe1507`, фаза 2 `d97417e`: `unlabeled-exit`,
  `length-subtraction`, `implicit-order` и `unit-lock` включены в `lint.yaml`, `catalog::ALL`, реестр Rust и `init`;
  навыки label-the-exit, subtract-checked, pin-the-order и lock-the-data; находки рига исправлены, пять меток циклов и
  `split_last` в `paths.rs`. P113 и B105 перенесены в Archive, Delivery с таблицей замера и причинами отказа в
  `Archive/Plan/113.md`.

## Verification

- Гейт коммита `d97417e` PASS: rust-test 178, test 408, Clippy, rustfmt, lint, memory, ruff, mypy, typos, vulture.
  Проба `init` кандидата: четыре правила в `lint.yaml`, четыре навыка в `.agentrig/skills`, `config-check` PASS,
  пробный файл даёт по находке каждого вида. `just bootstrap` установил `d97417e`; его `lint-rules` перечисляет
  четыре вида, его `lint` рига без находок.

## Blockers

Нет.

## Next action

`just feature-merge` ветки `feature/structural-bans`. После слияния разметить списки запретов `list-2-bans.md` и `list-3-bans.md`
итогами замера и начать P114 своей веткой от master. Свободные ID: D156+, I022+, P115+, B108+ (B066 и D111
зарезервированы).
