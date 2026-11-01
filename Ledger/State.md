# State

## Focus

**Набор II** — переносимые на любое приложение, agent-native улучшения качества (10 кластеров
C1–C10), по одной карточке за ветку, автономно. Порядок: C1 → C2 → C4 → C9 → C6 → **C3 детерминизм**
→ C5 контракты → C10 эволюция API → C7 анти-дублирование → C8 анти-переусложнение.
Доставлены **C1+C2 (P043)**, **C4 (P044)**, **C9 (P045)**, **C6 (P046)**. Сейчас — **C3 (B038/P047)**.

## Workspace

Ветка feature/determinism (база master @ `081d5e6` — merge-коммит C6). Машина — macOS.
Сборка/кандидат/коммит/бутстрап/git-мутации — через `just scratch -- '...'` с восстановленным PATH
(`cargo` в `~/.cargo/bin`); Agent-Bash — одна top-level `just`-команда без `&&`. Поиск — `just read --
grep ...` (без пайпов, globʼы в кавычках).

Механизм рантайма: хук/`config-check`/оркестрация гейта идут через `tooling/worker/run` →
УСТАНОВЛЕННЫЙ бинарь `.cache/development/<stable-hash>/` (из `tooling/distribution/stable.txt`, сейчас
`50c5ed79…` — голова архива C6). `lint`/build/clippy/rustfmt/rust-test и `init` (I020) используют
КАНДИДАТА. C3 — расширение ПОВЕДЕНИЯ существующего правила `directory-architecture` (не новый вид):
`lint.yaml` не меняется, I020 без изменений, НЕТ B1/B2-сплита и мид-ветка бутстрапа; финальный пин
D051 — на голову ветки как обычно.

## Progress

C3 ядро (по complexity-discipline, заземлено на D078): чтение стенных часов (`Instant::now`/
`SystemTime::now`) в собственных файлах `zone: pure`-каталога — находка, расширением проверки зоны.
Факты реализации: `architecture/mod.rs` — `ENVIRONMENT=[std::env,std::fs,std::process]`,
`environment(path)` (namespace на границе), `EnvironmentUse{line,path}`; `runner/analysis.rs:214`
классифицирует resolved external и пушит в `environment`-вектор; `policy.rs::check_zones` репортит их
для `zone: pure`-каталогов. Резолвер (`resolution/rust/paths.rs::import`/`external_path`) дописывает
суффикс `::now` → external = `std::time::Instant::now`/`std::time::SystemTime::now` (точное сопоставление
отличает чтение часов от `Duration`/`UNIX_EPOCH`).

**VAC-A (готов к коммиту, память)**: B038 заведена + промоутнута в P047; индексы Backlog/Plan; проза
промоушена и анализ в Plan; State. Ветка создана.

**VAC-B (предстоит, код)**: в `architecture/mod.rs` — `CLOCK=[std::time::Instant::now,
std::time::SystemTime::now]` + предикат `clock()`; `analysis.rs` — `environment()||clock()` в пуш;
`policy.rs::check_zones` — сообщение по категории (env оставить как есть, добавить сообщение про часы);
тесты (предикат `clock`, зона с clock-use) в `architecture/tests.rs`; карточка D083 (расширяет D078) +
индекс + линки. Дифференциация сообщений — чтобы не менять committed-поведение D078. Догфуд: `pure`-
модуль репозитория (`structural_lint/architecture/resolution`) часов не читает → зелёный.

## Verification

C6 влит в master (`081d5e6`), гейт всей ветки зелёный, пин stable `50c5ed7`. C3 — ещё не
реализовано/проверено. VAC-A проверяется коммит-гейтом (memory-check).

## Blockers

Нет.

## Next action

1. Закоммитить **VAC-A** (промоушен C3, память) — сообщение
   `P047 VAC-A: register B038 (C3 determinism), promote, record C3 analysis`.
2. **VAC-B**: реализовать `clock()` + расширить `check_zones`, тесты, карточка D083 + линки; догфуд
   (`just candidate lint` → 0 находок часов; `just config-check`); закоммитить через гейт.
3. Финал: ротация B038/P047 в Archive (флип D083 `[card]`), пин stable.txt на голову + бустрап D051,
   `just feature-merge` (ветку оставить, НЕ пушить). Затем C5. Свободные ID: P048, B039, D084+.
