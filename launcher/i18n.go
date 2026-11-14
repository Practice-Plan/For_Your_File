package main

// Launcher localization: all user-visible text comes from the embedded JSON
// resources in locales/. The language follows the Windows UI language and
// falls back to English when unsupported.

import (
	"embed"
	"encoding/json"
	"strings"
	"sync"
	"syscall"
)

//go:embed locales/*.json
var localeFS embed.FS

var (
	localeOnce sync.Once
	localeData map[string]map[string]string
	localeLang string
)

// lcidLang maps Windows primary language IDs to our resource languages.
var lcidLang = map[uint16]string{
	0x04: "zh",
	0x0c: "fr",
	0x19: "ru",
	0x01: "ar",
}

// systemLanguage returns the closest supported language for the current
// Windows UI language.
func systemLanguage() string {
	getLang := syscall.NewLazyDLL("kernel32.dll").NewProc("GetUserDefaultUILanguage")
	lcid, _, _ := getLang.Call()
	if lang, ok := lcidLang[uint16(lcid)&0x3ff]; ok {
		return lang
	}
	return "en"
}

func loadLocales() {
	localeData = make(map[string]map[string]string)
	for _, lang := range []string{"en", "zh", "fr", "ru", "ar"} {
		raw, err := localeFS.ReadFile("locales/" + lang + ".json")
		if err != nil {
			continue
		}
		var table map[string]string
		if json.Unmarshal(raw, &table) == nil {
			localeData[lang] = table
		}
	}
	localeLang = systemLanguage()
	if _, ok := localeData[localeLang]; !ok {
		localeLang = "en"
	}
}

// tr returns a localized string; missing entries fall back to English, then to
// the key itself. Arguments are key/value pairs for {{placeholder}} expansion.
func tr(key string, args ...string) string {
	localeOnce.Do(loadLocales)
	text := localeData[localeLang][key]
	if text == "" {
		text = localeData["en"][key]
	}
	if text == "" {
		return key
	}
	for i := 0; i+1 < len(args); i += 2 {
		text = strings.ReplaceAll(text, "{{"+args[i]+"}}", args[i+1])
	}
	return text
}
