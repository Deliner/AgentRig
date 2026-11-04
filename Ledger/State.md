# State

## Focus

**Программа реализации находок исследовательской дуги (семь раундов) через догфудинг.**
Пользователь поручил (/goal, 2026-11-04) реализовать все находки последовательно и для каждой
убедиться, что она работает. Карточки B052–B064 (B064 — идея, не выбрана), дополнение B049 (AC2);
выбраны в Plan **P055–P068** (порядок — в Plan.md). P055 (`33477fd`), P056 (`8c3142c`), P057 (`fbb0a06`),
P058 (`3d23588`), P059 (`a2dc929`) слиты. **P060** (B054, расширение `tuple-as-record`, [D101](Decisions/101.md))
завершается этим единственным VAC; следующая карточка — P061 (B055, recursive-deletion, новый `kind:` → двухфазно).

## Workspace

Ветка `feature/tuple-as-record-ext` (база master `a2dc929`), наблюдаемый HEAD `a2dc929` (коммитов в ветке ещё нет);
весь VAC в рабочем дереве (переносы карточек в индексе) и коммитится этим VAC. macOS: коммиты/cargo — через
`just scratch` с восстановленным PATH (`/Users/admin/.cargo/bin`), манифест — `Project/Cargo.toml`; догфуд —
`tooling/worker/run run candidate -- lint` внутри `just scratch`.

## Progress

- VAC (этот коммит): `record_signature` в `languages/rust.rs` для функций, сигнатур и замыканий — верхнеуровневый
  кортеж (D088, только именованные функции) или кортеж из трёх и более элементов со скаляром под
  `generic_type`/`array_type` на любой глубине (`annotations`, `nested_record`, `contained_record`, `unnamed_record`,
  `scalar`); дескриптор `TUPLEASRECORD` и навык `signature/name-the-record` описывают вложенную форму; тесты
  `tuple-as-record` вынесены в `languages/tests/tuple_as_record.rs` (находка `file-size` 511 > 500 в `markers.rs`;
  `mod.rs`, `architecture.yaml`); [D101](Decisions/101.md) + строка индекса, ссылки `tests` D088/D101 на новый
  файл; ротация P060 → `Archive/Plan/060.md` с Delivery, B054 → `Archive/Backlog/054.md`, строки индексов,
  примечание Plan.md.
- Не сделано: коммит через гейт, финальный пин D051 (stable.txt → голова ветки + bootstrap, отдельный коммит),
  `just feature-merge` (ветку сохранить, не пушить); далее P061–P068, по одной карточке за ветку.

## Verification

- 2026-11-04 на рабочем дереве: `cargo fmt --check` OK, `cargo clippy --all-targets -D warnings` OK, `cargo test
  --lib` — 121 passed; догфуд кандидата — 0 находок (exit 0); `lint-rule tuple-as-record` (кандидат) описывает
  расширенную метрику; живая проба `fn probe(rows: Vec<(String, u64, bool)>) {}` в `shared/vcs/git.rs` →
  `ERROR [tuple-as-record] … git.rs:247 (signature)` с навыком `name-the-record`; после отката — 0 находок.
- Гейт этого VAC ещё не запускался (коммит — следующий шаг); `lint-config-check`/`config-check` не запускались
  (конфигурация не менялась).

## Blockers

Нет.

## Next action

Закоммитить VAC через гейт (`git add Ledger Project`), затем финальный пин D051 (`git rev-parse HEAD >
tooling/distribution/stable.txt` + bootstrap, коммит «P060 D051 pin…»), `just feature-merge` (ветку сохранить,
не пушить). Далее `just feature-start` для P061 (B055). Свободные ID: D102+, I022+, P069+, B065+.
