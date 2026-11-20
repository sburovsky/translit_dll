# Transliteration Specification

## Purpose

Описывает поведение внешней компоненты 1С `TranslitAddin` — транслитерацию
русского текста (кириллица → латиница) по одной из трёх стандартных систем.
Спецификация фиксирует существующее (реализованное) поведение `src/lib.rs`.

## Requirements

### Requirement: Transliterate method

Система SHALL предоставлять метод компоненты `Transliterate` (русское имя
`Транслитерировать`) со сигнатурой:

```
Транслитерировать(ИсходнаяСтрока, СистемаТранслитерации = "") -> Строка
```

- `ИсходнаяСтрока` (Строка) — обязательный параметр, исходный текст на кириллице.
- `СистемаТранслитерации` (Строка) — необязательный параметр, код системы.
  Пустое значение означает ГОСТ 7.79-2000 (система Б).
- Возвращает строку на латинице.

#### Scenario: Synchronous and asynchronous invocation

- **WHEN** метод объявлен через `#[add_in_func(name = "Transliterate", name_ru = "Транслитерировать")]`
  с `#[returns(Str, result)]`
- **THEN** доступны синхронный (`ОбъектВК.Транслитерировать(...)`) и асинхронный
  (`Ждать ОбъектВК.ТранслитерироватьАсинх(...)`) вызовы, результат асинхронного
  вызова в свойстве `.Значение`.

#### Scenario: Return type errors become 1C exceptions

- **WHEN** метод компоненты возвращает `AddinResult = Result<String, Box<dyn Error>>`
- **THEN** ошибка преобразуется в исключение 1С.

### Requirement: System selection by code

Система SHALL определять систему транслитерации по коду `СистемаТранслитерации`
без учёта регистра и пробелов по краям. Пустой код приравнивается к `GOST`.
Неизвестный код генерирует ошибку.

Поддерживаемые системы и синонимы кодов:

| Код (основной) | Синонимы | Система |
|---|---|---|
| `GOST` | пустая строка, `ГОСТ`, `GOST779B` | ГОСТ 7.79-2000, система Б (по умолчанию) |
| `PASSPORT` | `ICAO`, `ЗАГРАНПАСПОРТ` | Загранпаспорт РФ 2013 (ICAO Doc 9303) |
| `DRIVER` | `ВУ`, `ВОДИТЕЛЬСКОЕ` | Водительское удостоверение (Приказ МВД № 995) |

#### Scenario: Empty code defaults to GOST

- **WHEN** передаётся пустой код системы
- **THEN** используется ГОСТ 7.79-2000 (система Б).

#### Scenario: Case and whitespace insensitivity

- **WHEN** код передан в ином регистре (`passport`, `гост`) или с пробелами по
  краям (` Driver `)
- **THEN** код сопоставляется корректно (нормализация — trim + uppercase).

#### Scenario: Unknown code raises an error

- **WHEN** передан неизвестный код системы (например, `UNKNOWN`)
- **THEN** возвращается ошибка, текст которой перечисляет допустимые коды
  (`GOST`, `PASSPORT`, `DRIVER`); в 1С она становится исключением.

### Requirement: Character preservation

Система SHALL передавать без изменений символы, отсутствующие в таблице
транслитерации: латиницу, цифры, знаки препинания и пробелы, сохраняя регистр.

#### Scenario: Latin, digits and punctuation pass through

- **WHEN** исходная строка содержит смешанный текст (`Дом 15 Abc`)
- **THEN** латиница и цифры остаются без изменений (`Dom 15 Abc` при PASSPORT).

#### Scenario: Case is preserved

- **WHEN** исходная строка — `ПРИВЕТ` или `привет`
- **THEN** результат сохраняет регистр (`PRIVET` / `privet`).

#### Scenario: Empty input yields empty output

- **WHEN** исходная строка пуста
- **THEN** возвращается пустая строка.

### Requirement: GOST system specifics

Система SHALL транслитерировать по таблице ГОСТ 7.79-2000 Б (`gost779b_ru()`),
включая особые соответствия: `ь` → `` ` ``, `ъ` → ``` `` ```, `ы` → ``y` ``,
`э` → ``e` ``, `х` → `x`, `ц` → `c`, `ё` → `yo`.

#### Scenario: GOST phrase transliteration

- **WHEN** транслитерируется фраза «Съешь же ещё этих мягких французских булок» кодом `GOST`
- **THEN** результат — ``S``esh` zhe eshhyo e`tix myagkix francuzskix bulok``.

#### Scenario: GOST soft and hard signs

- **WHEN** транслитерируется `соль` кодом `GOST`
- **THEN** результат — ``sol` `` (ь передаётся как обратный апостроф).

### Requirement: Other system outputs

Система SHALL различать результат по системам PASSPORT и DRIVER в соответствии
с их таблицами (`iternational_passport_2013_ru()`, `order_n_995_ru()`).

#### Scenario: Passport vs Driver phrase

- **WHEN** транслитерируется фраза «Съешь же ещё этих мягких французских булок»
  кодами `PASSPORT` и `DRIVER`
- **THEN** результат — `Sieesh zhe eshche etikh miagkikh frantsuzskikh bulok`.

#### Scenario: Letter ё differs by system

- **WHEN** транслитерируется `Ёж ёж`
- **THEN** при `GOST` результат `Yozh yozh`, при `PASSPORT` — `Ezh ezh`.

### Requirement: Dual-architecture packaging

Система SHALL собирать компоненту под Windows x86 (i686) и x64 и упаковывать в
единый ZIP-архив для загрузки в макет 1С.

#### Scenario: Build and pack artifacts

- **WHEN** выполнена команда `cargo make pack`
- **THEN** в каталоге `out/` создаются `translit_x32.dll`, `translit_x64.dll`,
  `Manifest.xml` и `translit.zip` (архив содержит обе DLL и манифест).

#### Scenario: Manifest declares both architectures

- **WHEN** читается `Manifest.xml`
- **THEN** `bundle` имеет `name="translit_addin"` и объявляет компоненты
  `arch="i386"` и `arch="x86_64"` типа `native`.