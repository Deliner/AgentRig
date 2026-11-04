# State

## Focus

**Программа реализации находок исследовательской дуги (семь раундов) через догфудинг.**
Пользователь поручил (/goal, 2026-11-04) реализовать все находки последовательно и для каждой
убедиться, что она работает. Карточки B052–B064 (B064 — идея, не выбрана), дополнение B049 (AC2);
выбраны в Plan **P055–P068** (порядок — в Plan.md). P055 (`33477fd`) и P056 (`8c3142c`) слиты. **P057**
(B053 AC1–AC4, `allow-justification`: `expect`, `cfg_attr`, `[lints]` Cargo.toml, [D098](Decisions/098.md))
завершается этим VAC-B2; следующая карточка — P058 (B059, discarded-result: `.status()?;`/`.output()?;`).

## Workspace

Ветка `feature/allow-justification-ext` (база master `8c3142c`), наблюдаемый HEAD `d8fea52` (пин D051 на
VAC-B1 `fdbd5bb`); VAC-B2 в рабочем дереве и коммитится этим VAC. macOS: коммиты/cargo — через `just scratch`
с восстановленным PATH (`/Users/admin/.cargo/bin`), манифест — `Project/Cargo.toml`; рабочая форма догфуда —
`tooling/worker/run run candidate -- lint` внутри `just scratch`.

## Progress

- VAC-B1 `fdbd5bb`: детекторы `languages/suppression.rs` и `languages/manifest.rs`, `Inspect::{Tree, Document}`
  и обработчик `toml` в `registry.rs`, дескрипторы, тесты, D098. Пин `d8fea52` (stable.txt → `fdbd5bb`).
- VAC-B2 (этот коммит): `.toml` в правиле `allow-justification` `.agentrig/lint.yaml` (`# DECISION: D098`),
  навык `justify-allow` с формами `expect`/`cfg_attr`/`[lints]` и «`reason =` не ссылка» (+ строка назначения в
  `references/architecture.yaml`), ссылки D098 на правило и навык, ротация P057 → `Archive/Plan/057.md` с
  Delivery и B053 → `Archive/Backlog/053.md`, строки архивных индексов, примечание Plan.md.
- Не сделано: финальный пин D051 (stable.txt → голова ветки + bootstrap, отдельный коммит), `just feature-merge`
  (ветку сохранить, не пушить); далее P058–P068, по одной карточке за ветку.

## Verification

- 2026-11-04 на рабочем дереве VAC-B2: `just lint-config-check` — valid с `.rs`/`.toml`; догфуд кандидата
  `tooling/worker/run run candidate -- lint` — 0 находок (exit 0); `just lint-rule allow-justification` —
  Languages: Rust (.rs) and TOML (.toml), дескриптор называет `reason = "..."`.
- Живая проба (добавлено через shell, откат `git checkout`): `#[expect(dead_code)]` → `allow-justification
  manifest.rs:73 (expect)`, `#[cfg_attr(test, allow(unused))]` → `:75 (allow)`, `[lints.rust] unused = "allow"`
  в `Project/Cargo.toml` → `Cargo.toml:42 (unused)` (навык `justify-allow`), `#[cfg_attr(miri, ignore)]` →
  `disabled-test :77 (ignore)` (навык `justify-skip`); после отката — 0 находок. Побочное наблюдение: `expect` и
  `cfg_attr` не входят в список атрибутов, которые анализатор зависимостей (`architecture/source/rust/mod.rs`)
  считает не влияющими на раскрытие, поэтому та же проба даёт каскад `architecture` «attribute or conditional
  expansion is not analyzed» по всему крейту — существующий предел, не объём B053.
- Гейт VAC-B2 ещё не запускался (коммит — следующий шаг).

## Blockers

Нет.

## Next action

Закоммитить VAC-B2 через гейт (`git add .agentrig/lint.yaml Ledger Project/shared/skills/references`, переименования
уже в индексе), затем финальный пин D051 (`git rev-parse HEAD > tooling/distribution/stable.txt` + bootstrap,
коммит «P057 D051 pin…»), `just feature-merge` (ветку сохранить, не пушить). Далее `just feature-start` для P058
(B059). Свободные ID: D099+, I022+, P069+, B065+.
