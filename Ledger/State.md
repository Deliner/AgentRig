# State

## Focus

Набор улучшений качества (кластеры 1–8), по одной карточке. Доставлены B027/P035, B029/P036,
B026/P037, B028/P038 (все в master). Текущая — [P039](Plan/039.md) ([B030](Backlog/030.md)):
структура как карта функционала. План по под-правилам: 5.2 (качество `purpose`) — расширением
правила `directory-architecture`; 5.1 (семьи вместе) и 5.3 (одна возможность = один файл) —
спайк, вероятно норма (D059). Следующие карточки набора — B031–B033.

## Workspace

Ветка: feature/structure-map (от master `0ed526f`, где слит P038), HEAD `165dfd9` (VAC-B, 5.2). Машина — macOS. Рабочее дерево: VAC-B (5.2) готов, не закоммичен —
`architecture/policy.rs` (+ набор `PLACEHOLDER_PURPOSES`, вторая `ensure` с маркером D070),
`architecture/tests.rs` (+3 кейса-болванки, +позитивный тест на фразу с дженерик-словом), карточка
D070, индекс/проза D070 в Decisions.md. stable.txt = `3ad2838` (установлен бустрапом на прошлой
ветке; будет перезаписан финальным пином этой ветки). Сборка/clippy/fmt/тесты/коммит/бустрап —
через `just scratch -- '...'` с PATH (`cargo` в `~/.cargo/bin`, venv `tooling/.venv/bin` первым
для Python-гейта); `just run`/гейт не видят cargo в PATH без этого.

## Progress

VAC-A (`c62149b`): plan-only промоушен B030→P039. VAC-B (5.2, в рабочем дереве): валидация
`purpose` в `directory-architecture` ужесточена — отвергается `purpose`, чьё всё значимое
содержимое (trimmed, без крайних не-буквенно-цифровых, lowercase) пусто или входит в фикс-набор
болванок/дженерик-токенов (`todo`,`module`,`...` и т.п.). Это строже существующего правила, без
нового `Kind`, без `catalog::ALL`, без нового навыка (ремонт — тот же `refactor-large-directory`);
I020 не меняется, доставка через бинарь + пин. Все ~95 наших `purpose` — содержательные (догфуд
зелёный). VAC-C (в рабочем дереве): норма 5.1/5.3 оформлена решением D071 (символьный резолвинг
с дырами glob/re-export/prelude и новый подсистемный объём — D059; 5.3 нераспознаваемо без
семантики фреймворка) — карточка + индекс/проза в Decisions.md. Нового кода/правила/навыка нет.

## Verification

VAC-B: `cargo test --lib architecture::` — 40 passed (вкл. новый
`contracts_accept_a_real_purpose_containing_a_generic_word` и расширенный
`contracts_reject_...`). Кандидат-линт (`just lint`) по репозиторию — exit 0 (0 находок болванок).
master после слияния P038 зелёный (86 Rust + 345 Python PASS). Полный гейт VAC-B — при коммите.

## Blockers

Нет.

## Next action

1) Закоммитить VAC-C (D071, норма 5.1/5.3) через хук. 2) Архив P039/B030 (Plan/039→Archive с
Delivery, Backlog/030→Archive, перенести card-ссылки D070 и D071 на `Archive/Backlog/030.md`,
индексы Plan.md/Backlog.md/Archive/*). 3) Финальный пин (D051, двухкоммитная схема P036):
stable.txt = голова архивного коммита + `just bootstrap`, отдельный коммит `pin <head>`.
4) `just feature-merge`. Не пушить, ветку сохранить. Отчёт на русском на границе merge; затем
следующая карточка (B031–B033).
