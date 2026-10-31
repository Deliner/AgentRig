# State

## Focus

**Набор II** — переносимые на любое приложение, agent-native улучшения качества (10 кластеров
C1–C10), по одной карточке за ветку, автономно. Порядок: C1 зонирование → C2 границы окружения →
C4 локальность → **C9 подавление линтов/`dbg!`** → C6 эффекты/ошибки → C3 детерминизм → C5
контракты поведения → C10 эволюция API/данных → C7 анти-дублирование → C8 анти-переусложнение.
Слитая **C1+C2 (B034/P043)** и **C4 (B035/P044)** доставлены. Сейчас — **C9 (B036/P045)**.

## Workspace

Ветка feature/hygiene (база master @ `bef58cb`), HEAD @ `cbb0ca9` (VAC-A). Машина — macOS. Сборка/
кандидат/коммит/бустрап/merge — через `just scratch -- '...'` с PATH (`cargo` в `~/.cargo/bin`, venv
`tooling/.venv/bin`). Agent-Bash — одна top-level `just`-команда без `&&`/хвостового `| tail`.

Механизм (важно для порядка VAC): dev-рантайм хука/`config-check` собирается per-HEAD
(`.cache/development/<hash>/`), НЕ из stable.txt. Поэтому новый `kind` нельзя назвать в живом
`.agentrig/lint.yaml`, пока его не знает бинарь текущего HEAD → правило добавляется в ДВА VAC
(проверено на disabled-test D074): **B1** = enum/descriptor/detector/registry/тесты (двигает HEAD,
рантайм узнаёт kind), **B2** = `.agentrig/lint.yaml` + `catalog::ALL` + навык + `assets.rs` + решение
+ догфуд-правка. `catalog::ALL` держится ВНЕ B1: иначе init отдаёт kind, которого нет в живом
lint.yaml → I020 (`test_delivery.py`, доставка == живое, по candidate-бинарю) падает.

## Progress

VAC-A закоммичен (`cbb0ca9`, гейт зелёный): карточка [B036](Backlog/036.md) → [P045](Plan/045.md),
строка и абзац анализа C9 в Plan.md. Анализ: единственное новое статическое ядро C9 — два Rust-
артефакта глушения гейта: широкий `#[allow(...)]`/`#![allow(...)]` без ссылки (атрибут, не
комментарий) → правило `allow-justification`, переиспользуя `justified` (D065/D074); `dbg!` → правило
`debug-artifact` (безусловно). Прочее уже обеспечено (D037 ловит Python `# noqa`/`# type: ignore`,
закомментированный код, `TODO`/`FIXME`; самоописание — D070; print — выбор D060) либо норма.

VAC-B1 готов к коммиту (source-only): `Kind::AllowJustification` (kind.rs), descriptor
`ALLOWJUSTIFICATION` + ветка match (descriptor.rs), в rust-handler `rules` (registry.rs), детектор —
`disabled_test` переписан в общий `attribute`/`attribute_name` на `attribute_item`/
`inner_attribute_item`, имя `ignore`→DisabledTest / `allow`→AllowJustification, через `justified`
(rust.rs), тест `rust_flags_allow_without_a_reference_on_the_line_above` → `[1, 11]` (tests/mod.rs).
`catalog::ALL` и lint.yaml НЕ трогаются (их — в B2).

Догфуд-импакт (замерено `grep`): ровно один реальный `#[allow]` — `Project/command_runner/jobs/
identity/macos.rs:65` `#[allow(deprecated)]` (чинить в B2: убрать, если сборка чистая без него, иначе
обосновать ссылкой D066); `#![allow]` нет; `dbg!` нет; `#[allow]` в `architecture/resolution/rust/
tests.rs:330` — внутри строкового литерала, по AST не находка.

## Verification

VAC-B1 проверен напрямую через `cargo +1.98.1` (PATH-обход): build + clippy (с deny-флагами гейта) +
rustfmt `--check` чисто; `--lib languages::tests` — 14/14 ok, включая новый allow-тест и сохранённый
`rust_flags_ignore_...` (поведение disabled-test не сломано). Python-тесты через scratch/uv падают с
127 (cargo не виден в подпроцессе `tooling/worker/build`) — артефакт среды, не кода; авторитетный
гейт коммита видит cargo (VAC-A прошёл). I020 для B1 сбалансирован по построению.

## Blockers

Нет. (Снят lockout `config-check`: живой lint.yaml возвращён к исходному — kind добавится в B2, когда
HEAD после B1 будет знать `allow-justification`.)

## Next action

Закоммитить VAC-B1 (`add allow-justification rule kind and measurement`) через хук. Затем:
- **VAC-B2** — сделать правило живым: `.agentrig/lint.yaml` (rule `allow-justification`), вернуть
  `Kind::AllowJustification` в `catalog::ALL`, навык `justify-allow` в `Project/shared/skills/
  references/` + `assets.rs` SKILLS, D080 (+ индекс/абзац в Decisions.md), догфуд-правка macos.rs:65.
  `just candidate lint` → 0 находок; гейт зелёный.
- **VAC-C** — `debug-artifact` (`dbg!`): детектор на `macro_invocation`, навык `remove-debug`, D081,
  тесты; тоже split B1/B2, если добавляет kind в lint.yaml. Догфуд: 0 находок.
- Ротация B036/P045 в Archive; пин stable.txt на gate-принятую голову + `just bootstrap` (D051);
  `just feature-merge` (не пушить, ветку сохранить). Отчёт на русском.

Свободные ID: D080, D081, P046, B037.
