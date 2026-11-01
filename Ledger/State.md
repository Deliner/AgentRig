# State

## Focus

**Набор II** — переносимые на любое приложение, agent-native улучшения качества (10 кластеров
C1–C10), по одной карточке за ветку, автономно. Порядок: C1 зонирование → C2 границы окружения →
C4 локальность → **C9 подавление линтов/`dbg!`** → C6 эффекты/ошибки → C3 детерминизм → C5
контракты поведения → C10 эволюция API/данных → C7 анти-дублирование → C8 анти-переусложнение.
Слитая **C1+C2 (B034/P043)** и **C4 (B035/P044)** доставлены. Сейчас — **C9 (B036/P045)**.

## Workspace

Ветка feature/hygiene (база master @ `bef58cb`), HEAD @ `c851755` (VAC-B2). Машина — macOS.
Сборка/кандидат/коммит/бутстрап — через `just scratch -- '...'` с восстановленным PATH (`cargo` в
`~/.cargo/bin`); Agent-Bash — одна top-level `just`-команда без `&&`/хвостового `| tail`.

Механизм рантайма: хук/`config-check`/оркестрация гейта идут через `tooling/worker/run`, который
читает `tooling/distribution/stable.txt` и исполняет УСТАНОВЛЕННЫЙ бинарь `.cache/development/
<stable-hash>/`; НЕ per-HEAD. Проверки `lint`/build/clippy/rustfmt/rust-test и `init` в I020
используют КАНДИДАТА (сборка из рабочего дерева через `tooling/worker/build`). Новый `kind` нельзя
назвать в живом `.agentrig/lint.yaml`, пока его не знает stable-бинарь → правило идёт в ДВА VAC:
**B1** = enum/descriptor/detector/registry/тесты (+ решение, memory-чекер требует marker→decision в
том же коммите), затем мид-ветка бустрап stable.txt на голову B1, затем **B2** = `.agentrig/
lint.yaml` + `catalog::ALL` + навык + `assets.rs` + линки решения + догфуд-правка. `catalog::ALL`
держится вне B1: иначе init отдаёт kind, которого нет в живом lint.yaml → I020 падает.

УРОК (дедлок): НИКОГДА не переставлять stable.txt на ещё не установленную ревизию — agent-хук
падает fail-closed и блокирует ВСЕ Bash/Edit/Write (Read нет). Бутстрап делать АТОМАРНО: запись
stable.txt + `bash tooling/distribution/bootstrap.sh` в одном `just scratch`, из уже-рабочего
состояния. bootstrap собирает из `git archive <rev>`, т.е. из КОММИТА, не из рабочего дерева.

## Progress

C9 ядро 1 (`allow-justification`) ДОСТАВЛЕНО: VAC-B1 `a464830` (enum/детектор/тест+D080), затем
бустрап stable.txt=`a464830`, затем VAC-B2 `c851755` (lint.yaml-правило + `catalog::ALL` + навык
`justify-allow` + `assets.rs` + линки D080 + догфуд `macos.rs` → `// DECISION: D066` над
`#[allow(deprecated)]`). Полный гейт зелёный; `just candidate lint` — 0 находок.

C9 ядро 2 (`debug-artifact`, `dbg!`) — **VAC-C-source готов в рабочем дереве, ещё не закоммичен**:
`Kind::DebugArtifact` (kind.rs), descriptor `DEBUGARTIFACT`+arm (descriptor.rs), rust-handler rules
(registry.rs), детектор `debug_macro` на `macro_invocation` c именем макроса `dbg` безусловно
(rust.rs), тест `rust_flags_dbg_macro_invocations`→`[2,6]` (вынесен в новый `tests/markers.rs`; `mod.rs` превысил file-size 500 → сплит disabled-test/allow/
debug-тестов в `markers.rs`, привязки D074/D080/D081 `[tests]` → `markers.rs`), решение D081
(карточка + индекс-строка + абзац). `catalog::ALL`/lint.yaml/навык — НЕ тронуты (это VAC-C-delivery).
stable.txt (рабочее дерево, не закоммичено) = `a464830`.

## Verification

`cargo +1.98.1 test --lib structural_lint::languages::tests` — 15/15 ok (вкл. новый `dbg`-тест и
сохранённые allow/ignore/unsafe). Полный гейт VAC-C-source ещё не прогонялся — будет при коммите.

## Blockers

Нет.

## Next action

Закоммитить VAC-C-source через хук (НЕ стейджить stable.txt), сообщение `P045 VAC-C1: add
debug-artifact rule kind and measurement; record D081`. Затем:
- Атомарный бустрап stable.txt на голову VAC-C-source (один `just scratch`), чтобы stable-бинарь
  узнал `debug-artifact`.
- **VAC-C-delivery** (`P045 VAC-C2`): правило `debug-artifact` в `.agentrig/lint.yaml`,
  `Kind::DebugArtifact` в `catalog::ALL`, навык `remove-debug` (`references/remove-debug/` + symlink
  + инвентарь + `assets.rs`), линки D081 (`[repository rule]`+`[skill]`), State. Догфуд: 0 находок.
- Ротация B036/P045 в Archive (+ флип `[card]` D080 и D081 на `Archive/Backlog/036.md`); финальный
  пин stable.txt на принятую гейтом голову + бустрап одним `just scratch` (D051); коммит пина;
  `just feature-merge` (не пушить, ветку сохранить). Отчёт на русском.

Свободные ID: P046, B037, D082.
