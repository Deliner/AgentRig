# State

## Focus

Пользователь 2026-11-10 задал стандарт рига, записан в AGENTS.md и D151. По нему показаны карточки B102–B105, и
пользователь поручил: «Все карточки подтверждаю, заводи и делай автономно». Выбрано P110 → P113; P110–P112 слиты,
идёт P113 (B105, 12 форм). Пользователь одобрил карточку B106 и поручил запланировать её следом: выбрана P114 после
P113. B107 о сверке имён узлов с грамматикой сохранена по поручению пользователя, но в работу не взята: она требует
дополнительного согласования, пользователь вернётся к ней сам.

## Workspace

Branch: `feature/structural-bans`, от master `3508e36`.
Revision: `4122a4fb60a8f32a1c94764a67252c8fa97daa03` (фаза 1 P113); этот снимок входит в коммит пина поверх неё.

Установленный рантайм `4122a4f` (пин середины P113). macOS: cargo, кандидат, коммиты и `feature-merge` через
`just scratch -- 'export PATH=/Users/admin/.cargo/bin:$PATH && …'`. Ветки прежних поставок сохранены,
`feature/init-probe` не трогать.

## Progress

- P110, P111 и P112 слиты в master; последний merge `3508e36`, пин P112 `a5a6af6`.
- voxel-rust: итоги ноутбука для P022 записаны коммитом `0eafc8e` и слиты в main `37de3ff`; P022 там paused.
- P113 VAC A `4a5c4b0`: детекторы Rust разнесены по семействам правил без смены поведения.
- P113 фаза 1 `4122a4f`, D155; её голова записана в `stable.txt` и установлена. Замер кандидатом на замороженных копиях voxel `src/` и рига `Project/` по
  протоколу, зафиксированному до замера; верные находки из всех, тестовый код исключён:

| Форма | Вид | voxel | риг | Итог |
| --- | --- | --- | --- | --- |
| III-15 | unlabeled-exit | 1 из 1 | 5 из 5 | поставляется |
| III-16 | length-subtraction | 2 из 2 | 1 из 1 | поставляется |
| III-11 | implicit-order | 0 | 0 | не измеримо, поставляется |
| III-23 | unit-lock | 0 | 0 | не измеримо, поставляется |
| III-28 | spawn в discarded-result | 0 | 0 | не измеримо, поставляется |
| II-29 | static mut | - | - | владелец unsafe-justification, D103 |
| II-13 | struct-update | не больше 44 из 50 | 5, не размечены | отклонено |
| II-20 | string-error | не больше 23 из 30 | 0 | отклонено |
| II-30 | unknown-fields | 0 | 4 из 8 | отклонено |
| III-10 | statement-cfg | не больше 2 из 8 | 0 из 4 | отклонено |
| III-18 | sentinel-value | 0 из 4 | 0 | отклонено |
| III-19 | разбор в silent-error | 1 из 1 | 0 из 1 | отклонено |

- Причины отказа: II-13 наборы опций wgpu и сообщения protobuf; II-20 ошибки только печатаются генератором схемы и
  профилировщиком; II-30 частичный разбор внешнего `Cargo.toml`; III-10 ветка называет модуль только под тем же
  `cfg`; III-18 коды возврата FFI в wasm; III-19 намеренное насыщение `i32::try_from(pid)` в `jobs/group.rs`.
- Фаза 1 инертна: виды есть в `rules/`, детекторы в `languages/rust/control_flow/`, `signature/implicit_order.rs`,
  `ownership/unit_lock.rs`, но их нет в `lint.yaml`, `catalog::ALL` и реестре. Расширение spawn живёт сразу, так как
  `discarded-result` уже настроен; навык handle-discarded дополнен. Тесты discarded-result перенесены из
  `tests/markers.rs` в `tests/discarded_result.rs`, ссылки D090, D099 и D114 переведены. Бюджет зависимых `rules`
  11 → 12 из-за нового каталога семейства control_flow.

## Verification

- Фаза 1: гейт коммита `4122a4f` PASS, rust-test 178 PASS, Clippy и rustfmt PASS, `just candidate lint` без находок,
  `config-check` и `lint-config-check` PASS; `just bootstrap` установил `4122a4f`. Реестр без изменений; `lint-rules` кандидата отличается от прежнего только метрикой
  `discarded-result`. На копиях находки рига и voxel со всеми правилами рига совпадают с прежними, новых spawn нет.

## Blockers

Нет.

## Next action

Фаза 2: четыре вида в `lint.yaml`, `catalog::ALL`, реестр, `project_setup/assets.rs` и `config/tests/test_config_check.py`;
навыки label-the-exit, subtract-checked, pin-the-order и lock-the-data; исправить находки рига; ротация P113 и B105
в Archive с Delivery по таблице выше; финальный пин и `just feature-merge`. Затем разметка списков запретов и P114
своей веткой от master. Свободные ID: D156+, I022+, P115+, B108+ (B066 и D111 зарезервированы).
