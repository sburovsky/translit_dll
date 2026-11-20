# OpenSpec Instructions

Инструкции для AI-ассистентов, работающих в этом репозитории. OpenSpec — это
подход к ведению изменений через спецификации (spec-driven development).

## Структура каталога

```
openspec/
├── project.md          # контекст проекта (назначение, стек, соглашения, ограничения)
├── AGENTS.md           # этот файл
├── config.yaml         # schema: spec-driven + опциональные правила
├── specs/              # неизменяемые «правила истины» — текущее поведение системы
│   └── <capability>/
│       └── spec.md
└── changes/            # предлагаемые изменения (дельта-спецификации)
    ├── <change-id>/
    │   ├── proposal.md
    │   ├── tasks.md
    │   └── specs/<capability>/spec.md   # дельта: ADDED/MODIFIED/REMOVED
    └── archive/        # применённые и архивированные изменения
```

## Принципы

- `specs/` = текущее, уже реализованное поведение (источник истины).
- `changes/` = предлагаемое, ещё не реализованное поведение.
- Любое изменение поведения (новая возможность, правка логики) оформляется
  как change proposal. Код не меняется до согласования proposal.
- После реализации и синхронизации изменения переносятся из `changes/` в
  `specs/` и архивируются.

## Формат спецификации (spec.md)

Базовые спецификации (`specs/<capability>/spec.md`) используют:

```markdown
# <Capability> Specification

## Purpose
...

## Requirements
### Requirement: <имя>
Система SHALL <поведение>.

#### Scenario: <имя>
- **WHEN** <условие>
- **THEN** <ожидаемый результат>
```

Дельта-спецификации в `changes/<id>/specs/<capability>/spec.md` используют те же
блоки `### Requirement:` / `#### Scenario:`, но объединяются заголовками секций:

- `## ADDED Requirements` — новые требования;
- `## MODIFIED Requirements` — изменённые (приводятся блоки целиком);
- `## REMOVED Requirements` — удалённые (пишутся только заголовки `### Requirement:`).

## Workflow

### 1. Создание изменения (propose)
1. Прочитай `project.md` и подходящие `specs/**`, чтобы понять текущее поведение.
2. Создай `changes/<change-id>/` с `proposal.md` (почему/что/scope), `tasks.md`
   (шаги реализации) и дельта-`spec.md`.
3. Название `<change-id>` — краткий kebab-case глагольный id (например,
   `add-ukrainian-system`, `fix-driver-table`).
4. Согласуй proposal с пользователем, прежде чем менять код.

### 2. Реализация (apply)
1. Выполни задачи из `tasks.md`.
2. Сверь итог с дельта-спецификацией и с существующими `specs/**`.
3. В Rust-проекте после правок прогони:
   `cargo fmt` → `cargo clippy --all-targets -- -D warnings` → `cargo test`.

### 3. Завершение (archive)
1. Перенеси дельта-требования в базовые `specs/<capability>/spec.md`.
2. Перемести каталог изменения в `changes/archive/`.
3. Убедись, что `specs/` не содержит «ADDED/MODIFIED/REMOVED» заголовков, а
   `changes/` не содержит применённых изменений.

## Доступный инструментарий

CLI `openspec` в окружении не установлен. Для работы со спецификациями доступны
встроенные навыки (skills): `openspec-propose`, `openspec-apply-change`,
`openspec-archive-change`, `openspec-explore`, `openspec-sync-specs`,
`openspec-update-change`. Используй их при необходимости.

## Соглашения этого репозитория

- Артефакты спецификаций пиши на русском языке. Структурные ключевые слова
  (`Requirement`, `Scenario`, `WHEN`, `THEN`, `SHALL`, `ADDED/MODIFIED/REMOVED`)
  остаются на английском (это ключевые слова OpenSpec).
- Изменение набора зависимостей, структуры каталогов или отклонение от
  `.clinerules/20-rust.md` фиксируй как договорённость в `project.md`.