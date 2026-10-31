# State

## Focus

**Набор II** — переносимые на любое приложение, agent-native улучшения качества (10 кластеров
C1–C10), по одной карточке за ветку, автономно. Порядок: C1 зонирование → C2 границы окружения →
C4 локальность → **C9 подавление линтов/`dbg!`** → C6 эффекты/ошибки → C3 детерминизм → C5
контракты поведения → C10 эволюция API/данных → C7 анти-дублирование → C8 анти-переусложнение.
Слитая **C1+C2 (B034/P043)** и **C4 (B035/P044)** доставлены. Сейчас — **C9 (B036/P045)**.

## Workspace

Ветка feature/hygiene (база master @ `bef58cb`). Машина — macOS. Сборка/кандидат/коммит/бустрап/
merge — через `just scratch -- '...'` с PATH (`cargo` в `~/.cargo/bin`, venv `tooling/.venv/bin`
первым). Agent-Bash — одна top-level `just`-команда без `&&`/хвостового `| tail`.

## Progress

VAC-A (регистрация карточки) готов к коммиту: заведена карточка [B036](Backlog/036.md), промоутнута
в [P045](Plan/045.md), в Plan.md добавлены строка и абзац с анализом C9. Анализ: единственное новое
статическое ядро C9 — два Rust-артефакта глушения гейта, не ловимые существующими правилами:
широкий `#[allow(...)]`/`#![allow(...)]` без ссылки-обоснования (атрибут, не комментарий) → новое
правило `allow-justification`, переиспользуя механизм justified (D065/D074); вызов `dbg!` → новое
правило `debug-artifact` (безусловно). Прочее уже обеспечено (бан комментариев D037 ловит Python
`# noqa`/`# type: ignore`, закомментированный код, `TODO`/`FIXME`; самоописание — D070; print — выбор
D060) либо норма. Реализация ещё не начата.

Догфуд-импакт (замерено `grep`): ровно один реальный `#[allow]` — `Project/command_runner/jobs/
identity/macos.rs:65` `#[allow(deprecated)]` (чинить: убрать, если сборка чистая без него, иначе
обосновать ссылкой); `#![allow]` нет; `dbg!` нет; `#[allow]` в фикстуре `architecture/resolution/
rust/tests.rs:330` — внутри строкового литерала, по AST не находка.

## Verification

VAC-A — memory-only (регистрация карточки); проверит memory-check при коммите через хук.

## Blockers

Нет.

## Next action

Закоммитить VAC-A (регистрация B036/P045) через хук. Затем:
- **VAC-B** — правило `allow-justification` end-to-end (enum `Kind::AllowJustification`, descriptor,
  `catalog::ALL`, rust-handler `rules`, детектор в `languages/rust.rs` на `attribute_item`/
  `inner_attribute_item` с именем `allow`, переиспользуя `justified`; `.agentrig/lint.yaml`; навык
  `justify-allow` в `Project/shared/skills/references/` + запись в `assets.rs` SKILLS; D080; тесты в
  `languages/tests/mod.rs`). Догфуд: починить macos.rs:65. `just candidate lint` → 0 находок.
- **VAC-C** — правило `debug-artifact` (`dbg!`) аналогично: детектор на `macro_invocation`, навык
  `remove-debug`, D081, тесты. Догфуд: 0 находок.
- Ротация B036/P045 в Archive; пин stable.txt на gate-принятую голову + `just bootstrap` (D051);
  `just feature-merge` (не пушить, ветку сохранить). Отчёт на русском.

Свободные ID: D080, D081, P046, B037.
