# State

## Focus

**Набор II** — переносимые на любое приложение, agent-native улучшения качества (10 кластеров
C1–C10), по одной карточке за ветку, автономно. Порядок: C1 зонирование → C2 границы окружения →
C4 локальность → **C9 подавление линтов/`dbg!`** → C6 эффекты/ошибки → C3 детерминизм → C5
контракты поведения → C10 эволюция API/данных → C7 анти-дублирование → C8 анти-переусложнение.
Слитая **C1+C2 (B034/P043)** и **C4 (B035/P044)** доставлены. Сейчас — **C9 (B036/P045)**.

## Workspace

Ветка feature/hygiene (база master @ `bef58cb`), HEAD @ `843a29e` (VAC-C-source). Машина — macOS.
Сборка/кандидат/коммит/бутстрап — через `just scratch -- '...'` с восстановленным PATH (`cargo` в
`~/.cargo/bin`); Agent-Bash — одна top-level `just`-команда без `&&`/хвостового `| tail`.

Механизм рантайма: хук/`config-check`/оркестрация гейта идут через `tooling/worker/run`, который
читает `tooling/distribution/stable.txt` и исполняет УСТАНОВЛЕННЫЙ бинарь `.cache/development/
<stable-hash>/`; НЕ per-HEAD. Проверки `lint`/build/clippy/rustfmt/rust-test и `init` в I020
используют КАНДИДАТА (сборка из рабочего дерева через `tooling/worker/build`). stable сейчас
забутстраплен на `843a29e` — stable-бинарь знает `debug-artifact`.

УРОК (порядок доставки): живое правило в `.agentrig/lint.yaml` нельзя добавлять раньше, чем создан
навык, на который оно ссылается: `config-check` падает с `paths.lint: No such file`, а agent-хук
fail-closed блокирует все Bash/Edit/Write (кроме config-файлов, `cat` их и `just config-check`).
Порядок: навык + `catalog::ALL` + `assets.rs` сначала, живое правило в `lint.yaml` — последним.
УРОК (дедлок): НИКОГДА не переставлять stable.txt на ещё не установленную ревизию; бутстрап делать
АТОМАРНО (запись stable.txt + `bash tooling/distribution/bootstrap.sh` в одном `just scratch`).

## Progress

C9 ядро 1 (`allow-justification`) ДОСТАВЛЕНО: VAC-B1 `a464830`, VAC-B2 `c851755`.

C9 ядро 2 (`debug-artifact`, `dbg!`): VAC-C-source `843a29e` (enum/descriptor/registry/детектор
`debug_macro` безусловно + тест `[2,6]` в `markers.rs` + D081). Бустрап stable.txt=`843a29e` сделан.
**VAC-C-delivery готов в рабочем дереве, ещё не закоммичен**: правило `debug-artifact` в
`.agentrig/lint.yaml` (Rust `.rs`, error, навык `remove-debug`), `Kind::DebugArtifact` в
`catalog::ALL`, навык `remove-debug` (`references/remove-debug/SKILL.md`+`architecture.yaml` +
symlink `shared/skills/remove-debug` + строка в `references/architecture.yaml` + `assets.rs` SKILLS),
линки D081 (`[repository rule]`+`[skill]`). stable.txt (рабочее дерево) = `843a29e` (не стейджить).

## Verification

`just config-check` — valid; `just lint-config-check` — valid; `just candidate lint` — 0 находок
(в репозитории нет `dbg!`). Полный гейт VAC-C-delivery ещё не прогонялся — будет при коммите (вкл.
I020 `test_delivery.py`: наборы rule-kinds и skills выросли на один и сбалансированы).

## Blockers

Нет.

## Next action

Закоммитить VAC-C-delivery через хук (НЕ стейджить stable.txt), сообщение `P045 VAC-C2: deliver
debug-artifact rule, remove-debug skill`. Затем финал C9:
- Ротация B036/P045 в Archive (+ флип `[card]` D080 и D081 на `Archive/Backlog/036.md`).
- Финальный пин stable.txt на принятую гейтом голову + бустрап одним `just scratch` (D051); коммит
  пина; `just feature-merge` (не пушить, ветку сохранить). Отчёт на русском.
- Далее Набор II: C6 → C3 → C5 → C10 → C7 → C8. Свободные ID: P046, B037, D082.
