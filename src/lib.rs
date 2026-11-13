use native_api_1c::{native_api_1c_core::ffi::connection::Connection, native_api_1c_macro::AddIn};
use std::sync::Arc;
use translit::{
    CharsMapping, ToLatin, Transliterator, gost779b_ru, iternational_passport_2013_ru,
    order_n_995_ru,
};

/// Коды ГОСТ 7.79-2000, система Б (система по умолчанию, в т.ч. для пустого кода).
const GOST_CODES: [&str; 4] = ["", "GOST", "ГОСТ", "GOST779B"];
/// Коды системы загранпаспорта РФ 2013 (ICAO Doc 9303).
const PASSPORT_CODES: [&str; 3] = ["PASSPORT", "ICAO", "ЗАГРАНПАСПОРТ"];
/// Коды системы водительского удостоверения (Приказ МВД № 995).
const DRIVER_CODES: [&str; 3] = ["DRIVER", "ВУ", "ВОДИТЕЛЬСКОЕ"];

/// Результат метода компоненты: ошибка превращается в исключение 1С.
type AddinResult = Result<String, Box<dyn std::error::Error>>;

/// Система транслитерации русского текста.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TranslitSystem {
    /// ГОСТ 7.79-2000, система Б.
    Gost,
    /// Загранпаспорт РФ 2013 (ICAO).
    Passport,
    /// Водительское удостоверение (Приказ МВД № 995).
    Driver,
}

impl TranslitSystem {
    /// Определяет систему по коду: без учёта регистра и пробелов по краям.
    ///
    /// Пустой код соответствует ГОСТ 7.79-2000 Б.
    /// Для неизвестного кода возвращается ошибка со списком допустимых кодов.
    pub fn from_code(code: &str) -> Result<Self, String> {
        let normalized = code.trim().to_uppercase();
        let code_ref = normalized.as_str();
        if GOST_CODES.contains(&code_ref) {
            Ok(Self::Gost)
        } else if PASSPORT_CODES.contains(&code_ref) {
            Ok(Self::Passport)
        } else if DRIVER_CODES.contains(&code_ref) {
            Ok(Self::Driver)
        } else {
            Err(format!(
                "Неизвестная система транслитерации \"{code}\". Допустимые значения: \
                 GOST (ГОСТ, GOST779B, пустая строка), PASSPORT (ICAO, ЗАГРАНПАСПОРТ), \
                 DRIVER (ВУ, ВОДИТЕЛЬСКОЕ)"
            ))
        }
    }

    fn mapping(self) -> CharsMapping {
        match self {
            Self::Gost => gost779b_ru(),
            Self::Passport => iternational_passport_2013_ru(),
            Self::Driver => order_n_995_ru(),
        }
    }
}

/// Транслитерирует кириллическую строку в латиницу по коду системы.
///
/// Символы, отсутствующие в таблице системы, передаются без изменений.
pub fn transliterate(source: &str, system_code: &str) -> Result<String, String> {
    let system = TranslitSystem::from_code(system_code)?;
    Ok(Transliterator::new(system.mapping()).to_latin(source))
}

/// Внешняя компонента 1С для транслитерации.
#[derive(AddIn)]
pub struct TranslitAddin {
    #[add_in_con] // соединение с 1С для вызова внешних событий
    connection: Arc<Option<&'static Connection>>,

    #[add_in_func(name = "Transliterate", name_ru = "Транслитерировать")]
    #[arg(Str)] // исходная строка на кириллице
    #[arg(Str, default = "")] // код системы транслитерации, пусто — ГОСТ 7.79-2000 Б
    #[returns(Str, result)]
    transliterate: fn(&Self, String, String) -> AddinResult,
}

impl TranslitAddin {
    /// Создаёт экземпляр компоненты.
    pub fn new() -> Self {
        Self {
            connection: Arc::new(None),
            transliterate: Self::transliterate,
        }
    }

    /// Обработчик метода 1С `Transliterate`.
    pub fn transliterate(&self, source: String, system_code: String) -> AddinResult {
        Ok(transliterate(&source, &system_code)?)
    }
}

impl Default for TranslitAddin {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PHRASE: &str = "Съешь же ещё этих мягких французских булок";

    #[test]
    fn gost_phrase() {
        assert_eq!(
            transliterate(PHRASE, "GOST"),
            Ok("S``esh` zhe eshhyo e`tix myagkix francuzskix bulok".to_string())
        );
    }

    #[test]
    fn passport_phrase() {
        assert_eq!(
            transliterate(PHRASE, "PASSPORT"),
            Ok("Sieesh zhe eshche etikh miagkikh frantsuzskikh bulok".to_string())
        );
    }

    #[test]
    fn driver_phrase() {
        assert_eq!(
            transliterate(PHRASE, "DRIVER"),
            Ok("Sieesh zhe eshche etikh miagkikh frantsuzskikh bulok".to_string())
        );
    }

    #[test]
    fn default_and_synonyms_equal_gost() {
        let expected = transliterate(PHRASE, "GOST");
        for code in ["", "ГОСТ", "гост", "GOST779B"] {
            assert_eq!(transliterate(PHRASE, code), expected, "код: {code:?}");
        }
    }

    #[test]
    fn codes_are_case_and_space_insensitive() {
        assert_eq!(
            TranslitSystem::from_code("passport"),
            Ok(TranslitSystem::Passport)
        );
        assert_eq!(
            TranslitSystem::from_code(" Driver "),
            Ok(TranslitSystem::Driver)
        );
        assert_eq!(
            TranslitSystem::from_code("icao"),
            Ok(TranslitSystem::Passport)
        );
        assert_eq!(TranslitSystem::from_code("ву"), Ok(TranslitSystem::Driver));
    }

    #[test]
    fn unknown_code_is_error() {
        let result = transliterate(PHRASE, "UNKNOWN");
        assert!(result.is_err_and(|e| e.contains("PASSPORT")));
    }

    #[test]
    fn empty_source() {
        assert_eq!(transliterate("", ""), Ok(String::new()));
    }

    #[test]
    fn mixed_text_keeps_latin_and_digits() {
        assert_eq!(
            transliterate("Дом 15 Abc", "PASSPORT"),
            Ok("Dom 15 Abc".to_string())
        );
    }

    #[test]
    fn case_is_preserved() {
        assert_eq!(transliterate("ПРИВЕТ", ""), Ok("PRIVET".to_string()));
        assert_eq!(transliterate("привет", ""), Ok("privet".to_string()));
    }

    #[test]
    fn yo_and_signs() {
        assert_eq!(transliterate("Ёж ёж", "GOST"), Ok("Yozh yozh".to_string()));
        assert_eq!(
            transliterate("Ёж ёж", "PASSPORT"),
            Ok("Ezh ezh".to_string())
        );
        assert_eq!(transliterate("объём", "PASSPORT"), Ok("obieem".to_string()));
        assert_eq!(transliterate("соль", "PASSPORT"), Ok("sol".to_string()));
        assert_eq!(transliterate("соль", "GOST"), Ok("sol`".to_string()));
    }

    #[test]
    fn addin_method_handles_codes() {
        let addin = TranslitAddin::new();
        assert!(addin.transliterate("тест".into(), "XXX".into()).is_err());
        assert!(
            addin
                .transliterate("тест".into(), String::new())
                .is_ok_and(|s| s == "test")
        );
    }
}
