# State

## Focus

На 2026-10-28 доставка трёх согласованных исходов аудита докфудинга на ветке
feature/dogfood-init завершена: B019/P028 (правило `architecture` и маршрут
структуры в поставке `init`), B020/P029 (три markdown-правила и навык
`condense-markdown`) и B021/P030 (статический страж согласованности поставки и
живой конфигурации). P028 и P029 закоммичены (40dbc30, cc42049); P030 реализован,
ещё не закоммичен. B019–B021 и P028–P030 перенесены в Archive.

## Workspace

Branch: feature/dogfood-init

Revision: cc42049 (HEAD, P029; VAC P030 ещё не закоммичен)

Машина — macOS (Darwin 25.5.0). Страж живёт в Project/shared/testing/test_delivery.py
как инвариант I020 с оракулом `test_dogfooding_capabilities_stay_universal`:
манифест `WORKER_LOCAL_CHECKS` (решение D053), helper `capabilities` читает обе
конфигурации, `divergences` сравнивает universal-измерения. Карта затронутых тестов
(.agentrig/checks.yaml) направляет страж на `.agentrig/**`. Сборки/тесты — через
`just scratch -- 'export PATH=/Users/admin/.cargo/bin:/Users/admin/.local/bin:
/opt/homebrew/bin:...; ...'`; кандидат — из tooling/worker/build.

## Progress

P030 реализован: инвариант I020 и оракул-тест в test_delivery.py, манифест
`WORKER_LOCAL_CHECKS` и helpers `capabilities`/`check_capability`/`divergences`,
запись маршрута `.agentrig/**` → test_delivery.py в .agentrig/checks.yaml, привязка
оракула в .agentrig/oracles.yaml, решение D053 (Decisions.md, Decisions/053.md) и
инвариант I020 (Invariants.md, Invariants/020.md). Карточки B019–B021 и P028–P030
перенесены в Archive/Backlog и Archive/Plan (ссылки на решения в B019/B021
углублены до `../../Decisions`). Изменения VAC P030 ещё не закоммичены.

## Verification

На кандидате: pytest test_delivery.py::test_dogfooding_capabilities_stay_universal
(1 passed), полный `lint` (exit 0), ruff/ruff-format/mypy по файлу стража чисто,
config-check (0). memory-check до переноса падал на ссылке Archive/Backlog/021.md;
после переноса карточек требуется повторный прогон перед коммитом. Полный гейт — на
коммите VAC P030.

## Blockers

Нет.

## Next action

Прогнать `memory-check` на кандидате для подтверждения ссылок и привязок после
переноса, затем закоммитить VAC P030 через hook (ожидается стоп D047 на первом VAC
— edit-state применён, повтор проходит). Далее по D051: записать принятый гейтом
head в tooling/distribution/stable.txt, выполнить bootstrap, закоммитить пин,
затем `just feature-merge` (ветку сохранить). Доложить пользователю.
