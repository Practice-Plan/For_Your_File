// i18n 守卫：① 各语言键集合必须与 en 完全一致；② 代码中不允许出现 CJK 文案字面量（注释除外）。
import fs from 'node:fs'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..')
const LANGS = ['en', 'zh', 'fr', 'ru', 'ar']
const BASE = 'en'

// 允许出现 CJK 的位置：语言资源本身、语言自称、文档与脚本
const CJK_ALLOW = [
  'src/i18n.ts', // nativeName: '中文' 是语言自称，属于数据
  'src/locales/',
  'src-tauri/locales/',
  'launcher/locales/',
  'scripts/check-i18n.mjs',
]

function flatten(obj, prefix = '', out = new Map()) {
  for (const [key, value] of Object.entries(obj)) {
    const full = prefix ? `${prefix}.${key}` : key
    if (value && typeof value === 'object' && !Array.isArray(value)) flatten(value, full, out)
    else out.set(full, value)
  }
  return out
}

function readJson(rel) {
  // 去掉文件开头可能存在的 UTF-8 BOM（兼容重复写入的情况）
  const raw = fs.readFileSync(path.join(ROOT, rel), 'utf8').replace(/^\uFEFF+/, '')
  return JSON.parse(raw)
}

const problems = []

// ① 键集合一致性
const baseKeys = flatten(readJson(`src/locales/${BASE}/translation.json`))
for (const lang of LANGS.filter((l) => l !== BASE)) {
  const keys = flatten(readJson(`src/locales/${lang}/translation.json`))
  const missing = [...baseKeys.keys()].filter((k) => !keys.has(k))
  const extra = [...keys.keys()].filter((k) => !baseKeys.has(k))
  if (missing.length) problems.push(`${lang} 缺少 ${missing.length} 个键：\n  ${missing.join('\n  ')}`)
  if (extra.length) problems.push(`${lang} 多出 ${extra.length} 个键：\n  ${extra.join('\n  ')}`)
}

// ② 代码中的 CJK 字面量（先剥离注释，注释允许用中文解释含义）
const SCAN = ['src', 'src-tauri/src', 'launcher']
const CJK = /[\u4e00-\u9fff]/
const EXT = new Set(['.ts', '.tsx', '.rs', '.go'])

function stripComments(text) {
  return text
    .replace(/\/\*[\s\S]*?\*\//g, '')
    .replace(/(^|[^:])\/\/.*$/gm, '$1')
}

function walk(dir) {
  for (const entry of fs.readdirSync(path.join(ROOT, dir), { withFileTypes: true })) {
    const rel = path.posix.join(dir, entry.name)
    if (CJK_ALLOW.some((p) => rel.includes(p))) continue
    if (entry.isDirectory()) walk(rel)
    else if (EXT.has(path.extname(entry.name))) {
      const lines = stripComments(fs.readFileSync(path.join(ROOT, rel), 'utf8')).split('\n')
      lines.forEach((line, i) => {
        if (CJK.test(line)) problems.push(`${rel}:${i + 1} 发现硬编码中文：${line.trim()}`)
      })
    }
  }
}
SCAN.forEach(walk)

if (problems.length) {
  console.error(`i18n 检查失败（${problems.length} 项）：\n`)
  problems.forEach((p) => console.error(`- ${p}\n`))
  process.exit(1)
}
console.log('i18n 检查通过')
