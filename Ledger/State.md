# State

## Focus

Пользователь 2026-11-07 выбрал для P089 разбиение `Kind` на семейные перечисления («Ок, принимаем 1 пункт. Делай все
оставшиеся карточки»). Выбрано **P087 → P090**; P087, P088 и P090 слиты в master, последний merge `5f02cdc`.
**P089** (B080, `unchecked-deserialize`) активен, см. [P089](Plan/089.md). Текущий VAC C: разбор имени вида через
семейства по [D134](Decisions/134.md), потому что рантайм мид-пина не принял инертный вид в `lint.yaml`.

## Workspace

Branch: `feature/unchecked-deserialize`
Revision: `2166104997d04c88288c4da936385462d381d969` (VAC C; коммит второго мид-пина следует)

В рабочем дереве также лежат незакоммиченные части фазы 2: навык `signature/check-deserialize`, его symlink и
описания в `architecture.yaml` групп навыков. В коммит VAC C они не входят. Ветки `feature/clippy-delivery`,
`feature/init-probe` и прежние сохранены. Установленный рантайм — `2166104`. macOS: коммиты, слияние и вызовы с
cargo через `just scratch -- 'export PATH=/Users/admin/.cargo/bin:$PATH && cd /Users/admin/projects/AgentRig && …'`.

## Progress

- VAC A `fcc4f1e`: `Kind` разбит на пять семейств по D133, ссылки решений перенесены; гейт PASS.
- VAC B `84aba7f`: инертный вид `Signature::UncheckedDeserialize`, детектор в `languages/rust/derived_trait.rs`,
  восемь тестов, D131; гейт PASS: rust-test 137, test 226. Мид-пин `ec9eebe`.
- Фаза 2 упёрлась в D133: установленный `84aba7f` разбирает имя только среди `catalog::ALL`, и хук заблокировал
  правки после добавления правила в `lint.yaml`. Правило убрано, `just config-check` проходит.
- VAC C: семейства выводят `Deserialize`, `Kind` разбирает имя через них в `catalog/mod.rs`, сообщение об ошибке
  прежнее; D134 и строка индекса со строкой о замене D133. VAC C `2166104`, гейт PASS: test 226.
- Второй мид-пин: `stable.txt` — `2166104`, `just bootstrap` код 0; установленный рантайм выводит
  `lint-rule unchecked-deserialize`.
- Осталось: коммит второго мид-пина; затем фаза 2: правило в `.agentrig/lint.yaml`, `catalog::ALL`, реестр,
  навык, `assets.rs`, список решённых в `test_config_check.py`, проверка AC1–AC4, ротация, пин, `just feature-merge`.

## Verification

- VAC C: `cargo test` 163 passed; `just run clippy` код 0. Кандидат: `lint-rule unchecked-deserialize --example`
  выводит правило; `lint-rule nope` выдаёт «unknown variant `nope`, expected one of …». Установленный `84aba7f` на
  `lint-rule unchecked-deserialize` выдаёт «unknown variant».

## Blockers

Нет.

## Next action

Закоммитить второй мид-пин, затем фаза 2 P089 одним VAC.
Свободные ID: D135+, I022+, P091+, B083+ (B066 и D111 зарезервированы).
