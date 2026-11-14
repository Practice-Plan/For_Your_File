//! 组件包创作指南：按语言返回，缺失回退英文。

const EN: &str = include_str!("../../docs/tech/component-packages/en.md");
const ZH: &str = include_str!("../../docs/tech/component-packages/zh.md");
const FR: &str = include_str!("../../docs/tech/component-packages/fr.md");
const RU: &str = include_str!("../../docs/tech/component-packages/ru.md");
const AR: &str = include_str!("../../docs/tech/component-packages/ar.md");

/// 返回指定语言的创作指南；不支持的语言回退英文。
pub fn for_language(lang: &str) -> String {
    let text = match crate::i18n::normalize(lang).as_str() {
        "zh" => ZH,
        "fr" => FR,
        "ru" => RU,
        "ar" => AR,
        _ => EN,
    };
    text.to_string()
}
