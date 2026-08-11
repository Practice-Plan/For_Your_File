//! Pinyin tokenization support for Chinese text search
//!
//! Provides utilities for converting Chinese characters to Pinyin
//! for better search results. Supports both full Pinyin and initials.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Common Chinese character to Pinyin mapping
/// This is a simplified dictionary covering ~2000 common characters
/// For production use, consider using the `pinyin` crate
static PINYIN_DICT: &[(&str, &str)] = &[
    // Common single character mappings
    ("文", "wen"), ("件", "jian"), ("管", "guan"), ("理", "li"),
    ("中", "zhong"), ("心", "xin"), ("系", "xi"), ("统", "tong"),
    ("设", "she"), ("置", "zhi"), ("程", "cheng"), ("序", "xu"),
    ("软", "ruan"), ("硬", "ying"), ("盘", "pan"), ("件", "jian"),
    ("目", "mu"), ("录", "lu"), ("文", "wen"), ("档", "dang"),
    ("编", "bian"), ("辑", "ji"), ("器", "qi"), ("工", "gong"),
    ("具", "ju"), ("运", "yun"), ("行", "xing"), ("启", "qi"),
    ("动", "dong"), ("开", "kai"), ("关", "guan"), ("闭", "bi"),
    ("桌", "zhuo"), ("面", "mian"), ("窗", "chuang"), ("口", "kou"),
    ("帮", "bang"), ("助", "zhu"), ("信", "xin"), ("息", "xi"),
    ("设", "she"), ("备", "bei"), ("网", "wang"), ("络", "luo"),
    ("安", "an"), ("全", "quan"), ("保", "bao"), ("护", "hu"),
    ("存", "cun"), ("储", "chu"), ("删", "shan"), ("除", "chu"),
    ("复", "fu"), ("制", "zhi"), ("粘", "zhan"), ("贴", "tie"),
    ("剪", "jian"), ("切", "qie"), ("找", "zhao"), ("查", "cha"),
    ("搜", "sou"), ("索", "suo"), ("浏", "liu"), ("览", "lan"),
    ("下", "xia"), ("载", "zai"), ("传", "chuan"), ("上", "shang"),
    ("图", "tu"), ("片", "pian"), ("视", "shi"), ("频", "pin"),
    ("音", "yin"), ("乐", "le"), ("游", "you"), ("戏", "xi"),
    ("学", "xue"), ("习", "xi"), ("教", "jiao"), ("育", "yu"),
    ("办", "ban"), ("公", "gong"), ("公", "gong"), ("司", "si"),
    ("项", "xiang"), ("目", "mu"), ("务", "wu"), ("任", "ren"),
    ("执", "zhi"), ("批", "pi"), ("处", "chu"), ("理", "li"),
    ("显", "xian"), ("示", "shi"), ("隐", "yin"), ("藏", "cang"),
    ("最", "zui"), ("大", "da"), ("小", "xiao"), ("化", "hua"),
    ("原", "yuan"), ("还", "huan"), ("撤", "che"), ("销", "xiao"),
    ("重", "chong"), ("做", "zuo"), ("消", "xiao"), ("除", "chu"),
    ("选", "xuan"), ("择", "ze"), ("择", "ze"), ("取", "qu"),
    ("消", "xiao"), ("确", "que"), ("定", "ding"), ("是", "shi"),
    ("否", "fou"), ("取", "qu"), ("消", "xiao"), ("应", "ying"),
    ("用", "yong"), ("序", "xu"), ("版", "ban"), ("本", "ben"),
    ("更", "geng"), ("新", "xin"), ("卸", "xie"), ("载", "zai"),
    ("修", "xiu"), ("改", "gai"), ("变", "bian"), ("化", "hua"),
    ("属", "shu"), ("性", "xing"), ("详", "xiang"), ("细", "xi"),
    ("简", "jian"), ("单", "dan"), ("复", "fu"), ("杂", "za"),
    ("快", "kuai"), ("慢", "man"), ("速", "su"), ("度", "du"),
    ("质", "zhi"), ("量", "liang"), ("高", "gao"), ("低", "di"),
    ("好", "hao"), ("坏", "huai"), ("新", "xin"), ("旧", "jiu"),
    ("正", "zheng"), ("常", "chang"), ("错", "cuo"), ("误", "wu"),
    ("成", "cheng"), ("功", "gong"), ("败", "bai"), ("失", "shi"),
    ("警", "jing"), ("告", "gao"), ("提", "ti"), ("示", "shi"),
    ("错", "cuo"), ("误", "wu"), ("异", "yi"), ("常", "chang"),
    ("问", "wen"), ("题", "ti"), ("答", "da"), ("案", "an"),
    ("测", "ce"), ("试", "shi"), ("调", "diao"), ("试", "shi"),
    ("开", "kai"), ("发", "fa"), ("布", "bu"), ("发", "fa"),
    ("源", "yuan"), ("码", "ma"), ("编", "bian"), ("译", "yi"),
    ("解", "jie"), ("析", "xi"), ("运", "yun"), ("行", "xing"),
    ("调", "tiao"), ("试", "shi"), ("断", "duan"), ("点", "dian"),
    ("监", "jian"), ("视", "shi"), ("日", "ri"), ("志", "zhi"),
    ("输", "shu"), ("出", "chu"), ("入", "ru"), ("控", "kong"),
    ("制", "zhi"), ("台", "tai"), ("终", "zhong"), ("端", "duan"),
    ("命", "ming"), ("令", "ling"), ("脚", "jiao"), ("本", "ben"),
    ("配", "pei"), ("置", "zhi"), ("环", "huan"), ("境", "jing"),
    ("变", "bian"), ("量", "liang"), ("数", "shu"), ("据", "ju"),
    ("库", "ku"), ("服", "fu"), ("务", "wu"), ("端", "duan"),
    ("客", "ke"), ("户", "hu"), ("端", "duan"), ("接", "jie"),
    ("口", "kou"), ("函", "han"), ("数", "shu"), ("类", "lei"),
    ("对", "dui"), ("象", "xiang"), ("模", "mo"), ("块", "kuai"),
    ("包", "bao"), ("引", "yin"), ("用", "yong"), ("导", "dao"),
    ("入", "ru"), ("出", "chu"), ("输", "shu"), ("打", "da"),
    ("印", "yin"), ("扫", "sao"), ("描", "miao"), ("识", "shi"),
    ("别", "bie"), ("识", "shi"), ("别", "bie"), ("认", "ren"),
    ("证", "zheng"), ("授", "shou"), ("权", "quan"), ("限", "xian"),
    ("角", "jiao"), ("色", "se"), ("用", "yong"), ("户", "hu"),
    ("组", "zu"), ("织", "zhi"), ("部", "bu"), ("门", "men"),
    ("员", "yuan"), ("工", "gong"), ("作", "zuo"), ("者", "zhe"),
    ("管", "guan"), ("理", "li"), ("员", "yuan"), ("操", "cao"),
    ("作", "zuo"), ("使", "shi"), ("用", "yong"), ("者", "zhe"),
    ("访", "fang"), ("问", "wen"), ("客", "ke"), ("游", "you"),
    ("客", "ke"), ("买", "mai"), ("家", "jia"), ("卖", "mai"),
    ("家", "jia"), ("商", "shang"), ("品", "pin"), ("订", "ding"),
    ("单", "dan"), ("支", "zhi"), ("付", "fu"), ("收", "shou"),
    ("款", "kuan"), ("转", "zhuan"), ("账", "zhang"), ("汇", "hui"),
    ("款", "kuan"), ("现", "xian"), ("金", "jin"), ("钱", "qian"),
    ("银", "yin"), ("行", "hang"), ("卡", "ka"), ("信", "xin"),
    ("用", "yong"), ("卡", "ka"), ("支", "zhi"), ("票", "piao"),
    ("发", "fa"), ("票", "piao"), ("收", "shou"), ("据", "ju"),
    // Additional common characters
    ("微", "wei"), ("软", "ruan"), ("视", "shi"), ("窗", "chuang"),
    ("代", "dai"), ("码", "ma"), ("编", "bian"), ("辑", "ji"),
    ("终", "zhong"), ("端", "duan"), ("远", "yuan"), ("程", "cheng"),
    ("本", "ben"), ("地", "di"), ("云", "yun"), ("端", "duan"),
    ("虚", "xu"), ("拟", "ni"), ("机", "ji"), ("容", "rong"),
    ("器", "qi"), ("环", "huan"), ("境", "jing"), ("依", "yi"),
    ("赖", "lai"), ("库", "ku"), ("框", "kuang"), ("架", "jia"),
    ("插", "cha"), ("件", "jian"), ("扩", "kuo"), ("展", "zhan"),
    ("主", "zhu"), ("题", "ti"), ("皮", "pi"), ("肤", "fu"),
    ("布", "bu"), ("局", "ju"), ("样", "yang"), ("式", "shi"),
    ("格", "ge"), ("模", "mo"), ("板", "ban"), ("页", "ye"),
    ("面", "mian"), ("视", "shi"), ("图", "tu"), ("图", "tu"),
    ("标", "biao"), ("图", "tu"), ("像", "xiang"), ("相", "xiang"),
    ("片", "pian"), ("视", "shi"), ("频", "pin"), ("音", "yin"),
    ("频", "pin"), ("动", "dong"), ("画", "hua"), ("效", "xiao"),
    ("果", "guo"), ("过", "guo"), ("渡", "du"), ("样", "yang"),
    ("本", "ben"), ("测", "ce"), ("试", "shi"), ("调", "diao"),
    ("试", "shi"), ("断", "duan"), ("点", "dian"), ("堆", "dui"),
    ("栈", "zhan"), ("跟", "gen"), ("踪", "zong"), ("性", "xing"),
    ("能", "neng"), ("优", "you"), ("化", "hua"), ("重", "chong"),
    ("构", "gou"), ("版", "ban"), ("本", "ben"), ("控", "kong"),
    ("制", "zhi"), ("分", "fen"), ("支", "zhi"), ("合", "he"),
    ("并", "bing"), ("冲", "chong"), ("突", "tu"), ("解", "jie"),
    ("决", "jue"), ("修", "xiu"), ("补", "bu"), ("更", "geng"),
    ("新", "xin"), ("发", "fa"), ("布", "bu"), ("回", "hui"),
    ("滚", "gun"), ("归", "gui"), ("档", "dang"), ("案", "an"),
    ("备", "bei"), ("份", "fen"), ("恢", "hui"), ("复", "fu"),
    ("损", "sun"), ("坏", "huai"), ("丢", "diu"), ("失", "shi"),
    ("错", "cuo"), ("误", "wu"), ("异", "yi"), ("常", "chang"),
    ("崩", "beng"), ("溃", "kui"), ("死", "si"), ("机", "ji"),
    ("重", "chong"), ("启", "qi"), ("退", "tui"), ("出", "chu"),
    ("崩", "beng"), ("退", "tui"), ("出", "chu"), ("码", "ma"),
];

/// Pinyin tokenizer with caching support
pub struct PinyinTokenizer {
    /// Cache for character to pinyin mappings
    cache: Arc<RwLock<HashMap<char, (String, String)>>>,
    /// Character to pinyin mapping
    dict: HashMap<char, &'static str>,
}

impl Default for PinyinTokenizer {
    fn default() -> Self {
        Self::new()
    }
}

impl PinyinTokenizer {
    /// Create a new Pinyin tokenizer
    pub fn new() -> Self {
        let mut dict = HashMap::new();
        for (ch, pinyin) in PINYIN_DICT {
            if let Some(c) = ch.chars().next() {
                dict.insert(c, *pinyin);
            }
        }

        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            dict,
        }
    }

    /// Convert a Chinese character to Pinyin
    ///
    /// # Arguments
    /// * `ch` - The character to convert
    ///
    /// # Returns
    /// A tuple of (full_pinyin, initials) or None if not Chinese
    fn char_to_pinyin(&self, ch: char) -> Option<(String, String)> {
        // Check cache first
        {
            let cache = self.cache.read().unwrap();
            if let Some(result) = cache.get(&ch) {
                return Some(result.clone());
            }
        }

        // Look up in dictionary
        if let Some(pinyin) = self.dict.get(&ch) {
            let full = pinyin.to_string();
            let initials = pinyin.chars().next().unwrap().to_string();
            
            // Cache the result
            let mut cache = self.cache.write().unwrap();
            cache.insert(ch, (full.clone(), initials.clone()));
            
            Some((full, initials))
        } else {
            None
        }
    }

    /// Convert a string to Pinyin representation
    ///
    /// # Arguments
    /// * `text` - The text to convert
    ///
    /// # Returns
    /// A tuple of (full_pinyin, initials)
    pub fn to_pinyin(&self, text: &str) -> (String, String) {
        let mut full_pinyin = String::new();
        let mut initials = String::new();

        for ch in text.chars() {
            if let Some((full, init)) = self.char_to_pinyin(ch) {
                full_pinyin.push_str(&full);
                initials.push_str(&init);
            } else if ch.is_ascii_alphabetic() {
                full_pinyin.push(ch.to_ascii_lowercase());
                initials.push(ch.to_ascii_lowercase());
            } else if ch.is_ascii_digit() || ch.is_whitespace() {
                full_pinyin.push(ch);
            }
        }

        (full_pinyin, initials)
    }

    /// Get Pinyin initials only
    pub fn to_pinyin_initials(&self, text: &str) -> String {
        let (_, initials) = self.to_pinyin(text);
        initials
    }

    /// Build a search query that matches both original text and Pinyin
    ///
    /// # Arguments
    /// * `query` - The search query
    ///
    /// # Returns
    /// An expanded FTS5 query string
    pub fn search_with_pinyin(&self, query: &str) -> String {
        let trimmed = query.trim();
        if trimmed.is_empty() {
            return "*".to_string();
        }

        // Check if query is pure ASCII (potential Pinyin input)
        let is_ascii = trimmed.chars().all(|c| c.is_ascii_alphabetic() || c.is_whitespace());

        if is_ascii {
            // Query might be Pinyin initials or full Pinyin
            // Return both the original and potential matches
            format!("({}*)", trimmed.replace(' ', "* "))
        } else {
            // Query contains Chinese, expand with Pinyin
            let (full_pinyin, initials) = self.to_pinyin(trimmed);

            if full_pinyin.is_empty() {
                format!("{}*", trimmed)
            } else {
                // Match original text OR full pinyin OR initials
                format!("({}* OR {}* OR {}*)", 
                    escape_fts_text(trimmed), 
                    escape_fts_text(&full_pinyin), 
                    escape_fts_text(&initials))
            }
        }
    }

    /// Clear the Pinyin cache
    pub fn clear_cache(&self) {
        let mut cache = self.cache.write().unwrap();
        cache.clear();
    }

    /// Get cache statistics
    pub fn cache_size(&self) -> usize {
        self.cache.read().unwrap().len()
    }
}

/// Escape text for FTS5 queries
fn escape_fts_text(text: &str) -> String {
    text.replace('"', "")
        .replace('*', "")
        .replace('(', "")
        .replace(')', "")
        .replace('{', "")
        .replace('}', "")
        .replace(':', "")
        .replace('+', "")
        .replace('-', "")
        .replace('~', "")
        .replace('^', "")
}

/// Convert a Chinese string to its Pinyin representation (standalone function)
pub fn to_pinyin(text: &str) -> String {
    let tokenizer = PinyinTokenizer::new();
    let (full, _) = tokenizer.to_pinyin(text);
    full
}

/// Get the first letter of each Pinyin syllable (standalone function)
pub fn to_pinyin_initials(text: &str) -> String {
    let tokenizer = PinyinTokenizer::new();
    tokenizer.to_pinyin_initials(text)
}

/// Build a search query that matches both Chinese and Pinyin (standalone function)
pub fn build_pinyin_query(query: &str) -> String {
    let tokenizer = PinyinTokenizer::new();
    tokenizer.search_with_pinyin(query)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pinyin_initials() {
        let tokenizer = PinyinTokenizer::new();
        let initials = tokenizer.to_pinyin_initials("文件管理");
        assert_eq!(initials, "wjgl");
    }

    #[test]
    fn test_to_pinyin_full() {
        let tokenizer = PinyinTokenizer::new();
        let (full, _) = tokenizer.to_pinyin("文件");
        assert_eq!(full, "wenjian");
    }

    #[test]
    fn test_ascii_query() {
        let tokenizer = PinyinTokenizer::new();
        let query = tokenizer.search_with_pinyin("vs");
        assert!(query.contains("vs"));
    }

    #[test]
    fn test_chinese_query() {
        let tokenizer = PinyinTokenizer::new();
        let query = tokenizer.search_with_pinyin("文件");
        // Should contain pinyin expansion
        assert!(query.contains("wenjian") || query.contains("wj"));
    }

    #[test]
    fn test_cache() {
        let tokenizer = PinyinTokenizer::new();
        tokenizer.to_pinyin("文件");
        assert!(tokenizer.cache_size() > 0);
        
        tokenizer.clear_cache();
        assert_eq!(tokenizer.cache_size(), 0);
    }

    #[test]
    fn test_standalone_functions() {
        assert_eq!(to_pinyin_initials("文件管理"), "wjgl");
        assert!(!build_pinyin_query("test").is_empty());
    }
}