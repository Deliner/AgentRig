# State

## Focus

**Набор II** — переносимые на любое приложение, agent-native улучшения качества (10 кластеров
C1–C10), по одной карточке за ветку, автономно. Порядок: C1 зонирование → C2 границы окружения →
C4 локальность → **C9 подавление линтов/`dbg!`** → C6 эффекты/ошибки → C3 детерминизм → C5
контракты поведения → C10 эволюция API/данных → C7 анти-дублирование → C8 анти-переусложнение.
Слитая **C1+C2 (B034/P043)** и **C4 (B035/P044)** доставлены. Сейчас — **C9 (B036/P045)**.

## Workspace

Ветка feature/hygiene (база master @ `bef58cb`), HEAD @ `a464830` (VAC-B1). Машина — macOS.
Сборка/кандидат/коммит/бутстрап — через `just scratch -- '...'` с восстановленным PATH (`cargo` в
`~/.cargo/bin`); Agent-Bash — одна top-level `just`-команда без `&&`/хвостового `| tail`.

Механизм рантайма (важно, прежняя запись «per-HEAD» была НЕВЕРНА): хук/`config-check`/оркестрация
гейта идут через `tooling/worker/run`, который читает `tooling/distribution/stable.txt` и исполняет
УСТАНОВЛЕННЫЙ бинарь `.cache/development/<stable-hash>/`; НЕ per-HEAD. Проверки `lint`/build/clippy/
rustfmt/rust-test и `init` в I020 используют КАНДИДАТА (сборка из рабочего дерева через
`tooling/worker/build`). Поэтому новый `kind` нельзя назвать в живом `.agentrig/lint.yaml`, пока его
не знает stable-бинарь → правило идёт в ДВА VAC: **B1** = enum/descriptor/detector/registry/тесты
(+ решение, т.к. memory-чекер требует marker→decision в том же коммите), затем мид-ветка бустрап
stable.txt на голову B1, затем **B2** = `.agentrig/lint.yaml` + `catalog::ALL` + навык + `assets.rs`
+ линки решения + догфуд-правка. `catalog::ALL` держится вне B1: иначе init отдаёт kind, которого
нет в живом lint.yaml → I020 падает.

УРОК (дедлок): НИКОГДА не переставлять stable.txt на ещё не установленную ревизию — agent-хук
(`worker/run hook`) падает fail-closed и блокирует ВСЕ Bash/Edit/Write (Read не заблокирован).
Бутстрап делать АТОМАРНО: запись stable.txt + `bash tooling/distribution/bootstrap.sh` в одном
`just scratch`, из уже-рабочего состояния (хук отрабатывает на старом установленном stable ДО
команды, а после команды новый stable уже установлен). bootstrap собирает из `git archive <rev>`,
т.е. из КОММИТА, не из рабочего дерева.

## Progress

VAC-A (`cbb0ca9`) и VAC-B1 (`a464830`) закоммичены. stable.txt (рабочее дерево, не закоммичено) =
`a464830`, установлен (знает `allow-justification`).

**VAC-B2 доставлен в рабочем дереве (этот VAC, ещё не закоммичен):** навык `justify-allow`
(`Project/shared/skills/references/justify-allow/SKILL.md` + `architecture.yaml`, flat-symlink
`Project/shared/skills/justify-allow`, инвентарь `references/architecture.yaml`, запись в
`project_setup/assets.rs` SKILLS); `Kind::AllowJustification` возвращён в `catalog::ALL`; правило
`allow-justification` (Rust, error) в `.agentrig/lint.yaml`; D080 «Applies in» дополнен
`[repository rule]`+`[skill]`. Догфуд-правка: единственный реальный `#[allow(deprecated)]`
(`command_runner/jobs/identity/macos.rs`) снабжён ссылкой `// DECISION: D066` (депрекейтед
`mach_timebase_info` без замены). Прочие `#[allow]` в коде — внутри строковых литералов/metric-строк,
по AST не находки.

## Verification

`just lint-config-check` — valid. `just candidate lint` (кандидат из рабочего дерева) — 0 находок.
Полный гейт коммита B2 (build/lint/memory/rustfmt/clippy/rust-test/ruff/mypy/test/typos/vulture) ещё
не прогонялся — будет при коммите через хук.

## Blockers

Нет. (Дедлок stable.txt снят: пользователь выполнил `! git restore`, затем атомарный бустрап
`a464830`.)

## Next action

Закоммитить VAC-B2 через хук (`just scratch -- 'export PATH=...; cd … && git add -A :/ && git commit
…'`, сообщение `P045 VAC-B2: deliver allow-justification rule, justify-allow skill, dogfood D066`;
НЕ стейджить stable.txt). Затем:
- **VAC-C** — правило `debug-artifact` (`dbg!`): новый `Kind::DebugArtifact`, детектор на
  `macro_invocation` c именем макроса `dbg` (безусловная находка), навык `remove-debug`, D081, тест;
  split B1(исходники+D081)/атомарный бустрап/B2(lint.yaml+catalog::ALL+навык+assets+линки). Догфуд: 0.
- Ротация B036/P045 в Archive (и флип `[card]` D080 на `Archive/Backlog/036.md`); финальный пин
  stable.txt на принятую гейтом голову + бустрап одним `just scratch` (D051); коммит пина;
  `just feature-merge` (не пушить, ветку сохранить). Отчёт на русском.

Свободные ID: D081, P046, B037.
