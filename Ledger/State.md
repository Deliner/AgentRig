# State

## Focus

P197 (B204): после успешного `feature-merge` риг удаляет всё, что Git игнорирует, кроме рабочего состояния рига,
закреплённого рантайма и путей из `git.keep`. VAC 2: очистка в рантайме с тестами AC1–AC6, решение D233, README.

## Workspace

Branch: feature/merge-cleanup, Revision: 82629dd (VAC 1, выбор P197). Правки VAC 2 не закоммичены.

## Progress

- `git.keep` (шаблоны, `Config::keep_matcher`, проверка при загрузке), `Repository::ignored`, `fs::remove_dir`.
- `shared/vcs/removal.rs`: пофайловое удаление без перехода по ссылкам, каталог только пустым, `.gitignore` только
  с каталогом; возвращает удалённое с размерами и ошибки.
- Jobs: `finished` (завершённые запуски), `directory` (каталог запусков) вместо копий `runtime.join("jobs")`.
- `change_delivery/git/sweep.rs`: план (сохраняемые пути состояния, ревизия из `HEAD:<pin>`, `git.keep`), отчёт
  «removed ignored files, freeing …», ошибки плана ничего не удаляют; ошибки — stderr и код 1.
- Тесты: `change_delivery/git/tests/sweep.rs` (AC2–AC6, fail-closed), `project_setup/tests/delivery.rs` (AC1),
  `merge_cleans_only_owned_task_runs` проверяет отчёт остановки, фикстура гейта хранит `.runtime/checks.json` в `keep`.
- D232 занят (инкрементальный release), решение записано как D233; README `project_setup` описывает очистку и `keep`.

## Verification

На рабочем дереве: `just check --only clippy`, `--only rustfmt` — PASS; `just run test --
change_delivery:: command_runner:: project_config:: shared::vcs project_memory:: agent_hooks:: project_setup::` —
255 passed. После маркеров D233, решения и сокращения README: `--only lint` и `--only memory` — PASS.

## Blockers

Нет.

## Next action

Закоммитить VAC 2 через хук, затем `just propose-pin HEAD`, закрепить рантайм, архивировать P197/B204, слить ветку.
