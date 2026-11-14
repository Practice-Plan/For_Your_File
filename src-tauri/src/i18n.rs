//! 后端本地化：所有面向用户的文案都必须经此查表，禁止在命令里写死语言。

use serde_json::Value;
use std::sync::Mutex;

const EN: &str = include_str!("../locales/en.json");
const ZH: &str = include_str!("../locales/zh.json");
const FR: &str = include_str!("../locales/fr.json");
const RU: &str = include_str!("../locales/ru.json");
const AR: &str = include_str!("../locales/ar.json");

pub const SUPPORTED: [&str; 5] = ["en", "zh", "fr", "ru", "ar"];

/// 当前界面语言，由前端在切换语言时写入。
pub struct AppLanguage(pub Mutex<String>);

impl AppLanguage {
    pub fn new(lang: &str) -> Self {
        Self(Mutex::new(normalize(lang)))
    }

    pub fn current(&self) -> String {
        self.0
            .lock()
            .map(|guard| guard.clone())
            .unwrap_or_else(|_| "en".to_string())
    }
}

pub fn normalize(lang: &str) -> String {
    let code = lang.split(['-', '_']).next().unwrap_or("en").to_lowercase();
    if SUPPORTED.contains(&code.as_str()) {
        code
    } else {
        "en".to_string()
    }
}

fn table(lang: &str) -> Value {
    let raw = match normalize(lang).as_str() {
        "zh" => ZH,
        "fr" => FR,
        "ru" => RU,
        "ar" => AR,
        _ => EN,
    };
    serde_json::from_str(raw)
        .unwrap_or_else(|_| serde_json::from_str(EN).expect("en.json must be valid JSON"))
}

fn lookup<'a>(root: &'a Value, key: &str) -> Option<&'a str> {
    let mut node = root;
    for part in key.split('.') {
        node = node.get(part)?;
    }
    node.as_str()
}

/// 查表并替换 `{{name}}` 占位符；缺失时回退英文，再退化为 key 本身。
pub fn t(lang: &str, key: &str, args: &[(&str, &str)]) -> String {
    let localized = table(lang);
    let fallback = table("en");
    let text = lookup(&localized, key)
        .or_else(|| lookup(&fallback, key))
        .unwrap_or(key);
    let mut out = text.to_string();
    for (name, value) in args {
        out = out.replace(&format!("{{{{{}}}}}", name), value);
    }
    out
}

/// 读取持久化的界面语言，未设置时回退英文。
pub fn load_language(app: &tauri::AppHandle) -> String {
    crate::ppc_linker::resolve_data_dir(app)
        .ok()
        .map(|dir| dir.join("language.txt"))
        .and_then(|path| std::fs::read_to_string(path).ok())
        .map(|text| normalize(text.trim()))
        .unwrap_or_else(|| "en".to_string())
}

/// 持久化界面语言，供下次启动时初始化托盘文案。
pub fn save_language(app: &tauri::AppHandle, lang: &str) {
    let Ok(dir) = crate::ppc_linker::resolve_data_dir(app) else {
        return;
    };
    if let Err(error) = std::fs::write(dir.join("language.txt"), normalize(lang)) {
        log::warn!("Failed to persist language: {}", error);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn falls_back_to_english_for_unknown_language() {
        assert_eq!(t("de", "tray.quit", &[]), "Quit");
    }

    #[test]
    fn interpolates_parameters() {
        let text = t("zh", "errors.components.reservedId", &[("id", "fyf.x")]);
        assert!(text.contains("fyf.x"));
        assert!(!text.contains("{{id}}"));
    }

    #[test]
    fn normalizes_region_codes() {
        assert_eq!(normalize("zh-CN"), "zh");
        assert_eq!(normalize("fr-CA"), "fr");
        assert_eq!(normalize("de-DE"), "en");
    }
}
