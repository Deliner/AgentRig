# State

## Focus

**Набор II** — переносимые на любое приложение, agent-native улучшения качества (10 кластеров
C1–C10), по одной карточке за ветку, автономно. Порядок: C1 зонирование → C2 границы окружения →
C4 локальность → C9 подавление линтов/`dbg!` → **C6 эффекты/ошибки** → C3 детерминизм → C5
контракты поведения → C10 эволюция API/данных → C7 анти-дублирование → C8 анти-переусложнение.
Слиты и доставлены **C1+C2 (B034/P043)**, **C4 (B035/P044)** и **C9 (B036/P045)**. Следующий — C6.

## Workspace

Ветка feature/hygiene (база master @ `bef58cb`), HEAD @ `dbcd342` (VAC-C2) + ротационный VAC.
Машина — macOS. Сборка/кандидат/коммит/бутстрап — через `just scratch -- '...'` с восстановленным
PATH (`cargo` в `~/.cargo/bin`); Agent-Bash — одна top-level `just`-команда без `&&`.

Механизм рантайма: хук/`config-check`/оркестрация гейта идут через `tooling/worker/run` →
УСТАНОВЛЕННЫЙ бинарь `.cache/development/<stable-hash>/` (из `tooling/distribution/stable.txt`); НЕ
per-HEAD. `lint`/build/clippy/rustfmt/rust-test и `init` (I020) используют КАНДИДАТА. stable
забутстраплен на `843a29e` (знает `debug-artifact`).

УРОК (порядок доставки): живое правило в `.agentrig/lint.yaml` нельзя добавлять раньше, чем создан
навык, на который оно ссылается: `config-check` падает (`paths.lint: No such file`), а agent-хук
fail-closed блокирует все Bash/Edit/Write. Порядок: навык + `catalog::ALL` + `assets.rs` сначала,
живое правило — последним. УРОК (дедлок): бутстрап только АТОМАРНО (запись stable.txt + `bash
tooling/distribution/bootstrap.sh` в одном `just scratch`), никогда не ставить stable на
неустановленную ревизию.

## Progress

C9 ДОСТАВЛЕНО полностью (два ядра). `allow-justification`: VAC-B1 `a464830`, VAC-B2 `c851755`.
`debug-artifact` (`dbg!`, безусловно): VAC-C1 `843a29e`, VAC-C2 `dbcd342`. Оба вида в `catalog::ALL`,
навыки `justify-allow`/`remove-debug` в `assets.rs` + symlink; правила в `.agentrig/lint.yaml`; линки
D080/D081. Ротация B036/P045 в Archive сделана в рабочем дереве (карточки перенесены, индексы Plan/
Backlog очищены, Archive/Plan.md + Archive/Backlog.md дополнены, `[card]` D080/D081 → `Archive/
Backlog/036.md`, прозаическая строка 2026-11-01 в Plan.md). Ротационный VAC ещё не закоммичен.

## Verification

Коммит-гейт VAC-C2 `dbcd342` зелёный: build/lint/memory/rustfmt/clippy/rust-test 96/ruff-format/ruff/
mypy/pytest 306 (вкл. test_delivery/I020)/typos/vulture — все PASS. `just candidate lint` — 0 находок.
Memory-check ротации прогонится при коммите ротационного VAC.

## Blockers

Нет.

## Next action

1. Закоммитить ротационный VAC (memory-only: Plan/Backlog/Archive/Decisions/State + перенос карточек),
   сообщение `P045: archive B036/P045 (C9 delivered)`. НЕ стейджить stable.txt.
2. D051: атомарный `just scratch` — stable.txt = голова ротационного VAC + `bash
   tooling/distribution/bootstrap.sh`; затем коммит пина stable.txt.
3. `just feature-merge` (не пушить, ветку сохранить). Отчёт на русском.
4. Далее Набор II: C6 → C3 → C5 → C10 → C7 → C8. Свободные ID: P046, B037, D082.
