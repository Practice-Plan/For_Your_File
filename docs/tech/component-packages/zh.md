# 组件包创作指南

从 v0.0.5 起，For_Your_File 的功能以**组件包（Component Package）**的形式组织。一个组件包是一个纯 JSON 文档，用来声明一组功能的配置；宿主程序读取并解释这些配置，**不会加载或执行任何第三方代码**，因此导入他人创作的组件包是安全的。

本文档同时作为应用内「创作指南」的内容来源（`get_component_creation_guide` 命令直接返回本文件）。

---

## 1. 文件结构

组件包就是一个扩展名为 `.json` 的文件：

```json
{
  "schema": 1,
  "id": "com.example.my-pack",
  "name": "我的组件包",
  "version": "1.0.0",
  "description": "一句话说明这个组件包做什么",
  "author": "你的名字",
  "kind": "theme",
  "components": [
    {
      "id": "appearance",
      "kind": "theme",
      "title": "我的配色",
      "config": { "accent": "violet" }
    }
  ]
}
```

## 2. 字段说明

| 字段 | 必填 | 说明 |
| --- | --- | --- |
| `schema` | 是 | 格式版本，当前固定为 `1` |
| `id` | 是 | 唯一标识。3–64 个字符，首字符为字母或数字，其余可用 `a-z`、`0-9`、`.`、`-`、`_`。建议使用 `com.作者.名称` 形式 |
| `name` | 是 | 显示名称，1–100 个字符 |
| `version` | 是 | 点分数字，如 `1.0.0`（1–4 段，每段只能是数字） |
| `description` | 否 | 描述文本 |
| `author` | 否 | 作者信息 |
| `kind` | 否 | 包类型，见下节；留空视为通用包 |
| `components` | 否 | 组件数组，最多 32 个 |

### 组件对象

| 字段 | 必填 | 说明 |
| --- | --- | --- |
| `id` | 是 | 组件标识，1–64 个字符 |
| `kind` | 是 | 组件类型，用于宿主选择解释方式 |
| `title` | 是 | 组件标题（界面展示用） |
| `config` | 否 | 任意 JSON 对象，序列化后不得超过 256 KiB |

## 3. 包类型（kind）

| kind | 用途 |
| --- | --- |
| `theme` | 外观：强调色、字号、动效等显示偏好 |
| `search` | 搜索行为：排序策略、每页数量、多语言开关 |
| `workflow` | 快捷操作：把已有命令组合成动作按钮 |
| `bundle` | 组合包：同时携带多种类型的组件 |

## 4. 配置约定

宿主按组件 `kind` 读取 `config`。当前已识别的键如下（未知键会被忽略，保留给未来版本）：

### 4.1 `theme`

```json
{ "accent": "blue", "font_scale": "normal", "reduce_motion": false }
```

| 键 | 取值 | 说明 |
| --- | --- | --- |
| `accent` | `blue` `violet` `emerald` `amber` `rose` | 强调色 |
| `font_scale` | `small` `normal` `large` | 界面字号 |
| `reduce_motion` | `true` / `false` | 减弱动画效果 |

### 4.2 `search`

```json
{ "sort": "relevance", "page_size": 50, "multilingual": true }
```

| 键 | 取值 | 说明 |
| --- | --- | --- |
| `sort` | `relevance` `mostUsed` `recentlyOpened` `alphabetical` | 默认排序 |
| `page_size` | 10–200 的整数 | 每页结果数 |
| `multilingual` | `true` / `false` | 是否对非 ASCII 关键词启用子串匹配（CJK、西里尔文、阿拉伯文等） |

### 4.3 `workflow`

```json
{
  "actions": [
    { "id": "rebuild-index", "title": "重建搜索索引", "command": "rebuild_fts_index" }
  ]
}
```

`command` 必须是宿主已注册的命令名（例如 `open_database_preview`、`rebuild_fts_index`）。未知命令不会被执行，只会以禁用状态展示。

## 5. 导入、导出与内置包

- **导入**：设置 → 组件包 → 导入，选择 `.json` 文件。导入时会校验 schema、id、版本、kind、组件数量和配置体积；不合法会被拒绝并说明原因。
- **导出**：任意包（含内置包）都可以导出为 JSON。导入的包可原样再次导入；内置包导出后仅作为创作模板——重新导入前请先修改 `id`（内置包 id 为保留值）。
- **内置默认包**：应用内置三个包，属于二进制的一部分：
  - `fyf.appearance.classic` — 经典蓝色外观
  - `fyf.search.balanced` — 相关度优先、每页 50 条、多语言匹配
  - `fyf.workflow.quicklaunch` — 维护类快捷动作
  内置包可以启用 / 停用，但**不能删除或覆盖**（其 id 为保留值）。
- **存储位置**：导入的包保存在应用数据目录的 `components/packages/<id>.json`，启用状态保存在 `components/state.json`。

## 6. 安全约束

1. 组件包是**声明式**的：不包含脚本、可执行文件或远程资源引用，宿主永远不会执行包内的字符串。
2. 单个文件最大 1 MiB，单个组件配置最大 256 KiB，组件数量最多 32 个。
3. 包 id 必须唯一；内置包 id 不可被占用。
4. 未知字段与未知 `kind` 会被忽略或以只读方式展示，不会导致应用崩溃。

## 7. 最小示例：自定义紫色主题

```json
{
  "schema": 1,
  "id": "com.example.violet-theme",
  "name": "紫色夜间",
  "version": "1.0.0",
  "description": "紫色强调色 + 减弱动效",
  "author": "example",
  "kind": "theme",
  "components": [
    {
      "id": "appearance",
      "kind": "theme",
      "title": "Violet",
      "config": { "accent": "violet", "font_scale": "normal", "reduce_motion": true }
    }
  ]
}
```

保存为 `violet-theme.json`，在应用内导入即可。
