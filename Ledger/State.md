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

**VAC-A закоммичен** (`ef62152`, память, гейт зелёный): B038 заведена + промоутнута в P047; индексы
Backlog/Plan; проза промоушена и анализ в Plan; State. Ветка создана.

**VAC-B (код готов, dogfood-зелёный, НЕ закоммичен)**: `architecture/mod.rs` — `CLOCK=[std::time::
Instant::now, std::time::SystemTime::now]` + предикат `clock()`; `analysis.rs` — `environment()||clock()`
в пуш `EnvironmentUse`; `policy.rs` — свободная fn `zone_message()` по категории (env-строка байт-
идентична D078, добавлено «must not read the wall clock»); `rules/architecture.rs` — описание контракта
расширено (без маркера, не линкуется, как D078); 3 теста (`clock_reads_match_only_the_now_constructors`,
`zone_pure_directories_reject_wall_clock_reads_in_their_own_files`,
`clock_constructors_resolve_to_their_full_std_path`); карточка `Decisions/083.md` (расширяет D078) +
индекс-строка + линки на 5 файлов с маркером. Дифференциация сообщений — чтобы не менять committed-
поведение D078.

## Verification

C6 влит в master (`081d5e6`), гейт всей ветки зелёный, пин stable `50c5ed7`. VAC-A прошёл коммит-гейт
(`ef62152`). VAC-B: `cargo test --manifest-path Project/Cargo.toml` — 3 новых теста зелёные + регрессия
D078 (сообщение окружения не изменилось); `just candidate lint` → 0 находок (единственный `pure`-каталог
репозитория `structural_lint/architecture/resolution` часов не читает). Коммит-гейт VAC-B — предстоит.

## Blockers

Нет.

## Next action

1. Закоммитить **VAC-B** через гейт — код (`architecture/mod.rs`, `analysis.rs`, `policy.rs`,
   `rules/architecture.rs`), 3 теста, `Decisions/083.md`, индекс-строка Decisions, State. Сообщение
   `P047 VAC-B: deliver pure-zone wall-clock rule; record D083`.
2. Финал C3: ротация B038→`Archive/Backlog/038.md` + строка, P047→`Archive/Plan/047.md` (`## Delivery`)
   + строка, проза завершения в Plan, `git rm` живых `Plan/047.md`+`Backlog/038.md`, флип D083 `[card]`
   → `Archive/Backlog/038.md`, refresh State; коммит `P047: archive B038/P047 (C3 delivered)`.
3. Пин D051: stable.txt = полный 40-символьный хеш головы архив-коммита + атомарный бутстрап (один
   scratch, откат при сбое); коммит `P047: pin stable.txt to <short> (D051)`.
4. `just feature-merge` (ветку оставить, НЕ пушить). Отчёт по C3 на русском. Затем C5. Свободные ID:
   P048, B039, D084+.
