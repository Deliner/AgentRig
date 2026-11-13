# State

## Focus

Пользователь 2026-11-10 задал стандарт рига, записан в AGENTS.md и D151. P110–P135 выполнены и перенесены в Archive.
B107 о сверке имён узлов с грамматикой сохранена, но в работу не взята: она требует дополнительного согласования,
пользователь вернётся к ней сам. B112 ждёт доступа к Linux-машине. B109 и B126 сохранены, в работу не выбраны.

2026-11-13 пользователь согласился с разбором списка V `/Users/admin/projects/junk/rust-ast-rules/list-5-candidates.md`
и поручил завести карточки, последовательно их сделать, обязательно провести замеры и затем разметить в файле списка,
что принято, что нет и почему. P135 по B131 выполнена; следующая P136 по B132, свои правила рига.

## Workspace

Branch: `feature/list5-clippy`, от master `89992fa`, где влита P134.
Revision: `9cd3cefffb673b30a57205798ae415fb1576718b`

Установленный рантайм `9cd3cef`, он же в `stable.txt`. macOS: cargo, кандидат, коммиты и `feature-merge` через
`just scratch -- 'export PATH=/Users/admin/.cargo/bin:$PATH && …'`. Ветки прежних поставок сохранены и отправлены в
`origin`, `feature/init-probe` не трогать. Скрипты замера списка IV восстановлены из истории сессии в
`scratchpad/p135/old`; скрипты и копии P135 лежат в `scratchpad/p135`.

## Progress

- P135: VAC 1–4 `3207ad6`, `b068079`, `3425bdb`, `9cd3cef`; D180 включает `default_trait_access`,
  `suspicious_xor_used_as_pow`, `host_endian_bytes`, `large_stack_arrays` и `maybe_infinite_iter`, а
  `unreadable_literal` не прошёл замер, 12 из 24. VAC 5 закрепляет `9cd3cef` по D051 и архивирует P135 с B131.
- В rustc 1.100 `-A warnings` глушит и lint, включённый после него: замеры Clippy идут с одним `-W` и отбором по
  коду lint-а.
- V-4, V-26, V-21 и V-22 уже ловят правила рига `disabled-test`, `disabled-raii`, `comments` и
  `unsafe-justification`.

## Verification

Хук коммита VAC 4 прошёл: build, lint, memory, rustfmt, clippy, rust-test 496, ruff, mypy, typos и vulture. `just
bootstrap` установил `9cd3cef`. VAC 5 меняет память и `stable.txt`.

## Blockers

Нет.

## Next action

Влить ветку через `just feature-merge`, затем начать P136 веткой `feature/list5-rules` от master: протокол, инертные
виды правил, замер на риге, voxel и корпусе r8. Свободные ID: D181+, I022+, P137+, B133+.
