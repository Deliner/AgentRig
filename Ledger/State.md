# State

## Focus

P197 (B204): после успешного `feature-merge` риг удаляет всё, что Git игнорирует, кроме рабочего состояния рига,
закреплённого рантайма и путей из списка сохраняемых. VAC 1: пункт P197 выбран в Plan.

## Workspace

Branch: feature/merge-cleanup, Revision: c8e36f8 (master при старте ветки).

## Progress

VAC 1: строка P197 в Plan.md, карточка Plan/197.md, запись о выборе пользователя.

## Verification

Коммит VAC 1 проверяет гейт pre-commit (память и lint).

## Blockers

Нет.

## Next action

VAC 2: в рантайме — список `keep` в `git.yaml`, очистка игнорируемого после слияния, удаление журналов завершённых
запусков, тесты AC1–AC6.
