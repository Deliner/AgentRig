# State

## Focus

Пользователь 2026-11-10 поручил: «Сохрани показанные карточки, заведи пункт вот по этому по линту, который ты
написал, и все карточки, и вообще все карточки в бэклоге, которые есть, автономно реализуй сам. У меня ничего не
спрашиваю, я на все даю подтверждение». Все карточки Backlog B086–B101 выбраны в Plan как P094–P109, выполнены и
слиты. Риг проверялся на voxel-rust и под него не подгонялся.

## Workspace

Branch: `feature/final-pass-state`, от master.
Revision: `589f972aff00aa32745207144a69aefb1ffd1bbe` (слияние P109); этот снимок закоммичен поверх.

Установленный рантайм `6c13aa2` (пин P109): `.agentrig/bin/agentrig` запускает `bin/macos-arm64/agentrig`. macOS:
коммиты и `feature-merge` с cargo через `just scratch -- 'export PATH=/Users/admin/.cargo/bin:$PATH && …'`.
Ветки прежних поставок сохранены, `feature/init-probe` не трогать. Клоны `voxel-b101`, `voxel-orig` и `voxel-p109`
в scratchpad одноразовые, коммитов в них нет.

## Progress

- P094–P109 завершены и слиты в master; Plan и Backlog пусты.
- Финальный проход voxel-rust на ветке `feature/agentrig-migration` закоммичен через её хуки рантаймом `6c13aa2`:
  `dee3088` — скрипт запуска и рантайм macOS arm64 через `runtime-add`, tools `bin/**`, паники в телах макросов
  `exports!` и `records!`; `0eee45e` — 13 тестовых модулей в `tests/`, правило test-placement, бюджет
  `src/core/terrain` 7, одна ссылка D008 и D009; `787d89b` — `pub(in crate::native)` в host, цели без `host::`,
  исключения lock-файлов из шаблона. `ABI_REVISION` voxel-rust стал 518149772, core.ts перегенерирован.
- В voxel-rust `#[allow(clippy::expect_used)]` не заменён на `#[expect]`: AgentRig распознаёт оба.

## Verification

- P109: `test_runtime.py` 4 passed, гейт коммита и `feature-merge` 239 passed.
- voxel-rust `dee3088`, `0eee45e` и `787d89b`: в каждом коммите хук прошёл lint, memory, tests, clippy,
  build-checks и native-tests; у двух последних tests 132 и native-tests 160. build-checks Android требуют
  `JAVA_HOME=/opt/homebrew/opt/openjdk@21`.
- Настоящий Linux x86_64 не проверялся.

## Blockers

voxel-rust в main не слит: на main лежит один рантайм Linux x86_64 старой раскладки, и на Mac `git switch main`
запускает его и отказывает. По D150 слияние делает пользователь с Linux после `runtime-add` там. Приёмка GPU, Web и
Android voxel-rust за пользователем.

## Next action

Ждать пользователя: слияние voxel-rust с Linux и приёмка GPU, Web и Android. Свободные ID: D151+, I022+, P110+,
B102+ (B066 и D111 зарезервированы).
