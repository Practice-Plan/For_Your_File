# Component Package Authoring Guide

Since v0.0.5, For_Your_File organizes its features as **component packages**. A component package is a plain JSON document that declares the configuration of a set of features; the host reads and interprets that configuration and **never loads or executes third-party code**, so importing packages written by others is safe.

This document is also the source of the in-app "Creation Guide" (the `get_component_creation_guide` command returns this file).

---

## 1. File structure

A component package is simply a file with the `.json` extension:

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

## 2. Fields

| Field | Required | Description |
| --- | --- | --- |
| `schema` | Yes | Format version, currently fixed at `1` |
| `id` | Yes | Unique identifier. 3–64 characters; the first must be a letter or digit, the rest may use `a-z`, `0-9`, `.`, `-`, `_`. The `com.author.name` form is recommended |
| `name` | Yes | Display name, 1–100 characters |
| `version` | Yes | Dot-separated numbers, e.g. `1.0.0` (1–4 segments, digits only) |
| `description` | No | Description text |
| `author` | No | Author information |
| `kind` | No | Package kind, see below; empty means a generic package |
| `components` | No | Component array, at most 32 |

### Component object

| Field | Required | Description |
| --- | --- | --- |
| `id` | Yes | Component identifier, 1–64 characters |
| `kind` | Yes | Component kind, used by the host to choose an interpreter |
| `title` | Yes | Component title (shown in the UI) |
| `config` | No | Arbitrary JSON object, at most 256 KiB after serialization |

## 3. Package kinds

| kind | Purpose |
| --- | --- |
| `theme` | Appearance: accent color, font scale, motion and other display preferences |
| `search` | Search behavior: sort strategy, page size, multilingual switch |
| `workflow` | Quick actions: combine existing commands into action buttons |
| `bundle` | Bundle: carries components of several kinds at once |

## 4. Configuration conventions

The host reads `config` according to the component `kind`. The currently recognized keys are listed below (unknown keys are ignored and reserved for future versions):

### 4.1 `theme`

```json
{ "accent": "blue", "font_scale": "normal", "reduce_motion": false }
```

| Key | Values | Description |
| --- | --- | --- |
| `accent` | `blue` `violet` `emerald` `amber` `rose` | Accent color |
| `font_scale` | `small` `normal` `large` | Interface font scale |
| `reduce_motion` | `true` / `false` | Reduce animation effects |

### 4.2 `search`

```json
{ "sort": "relevance", "page_size": 50, "multilingual": true }
```

| Key | Values | Description |
| --- | --- | --- |
| `sort` | `relevance` `mostUsed` `recentlyOpened` `alphabetical` | Default sort order |
| `page_size` | Integer from 10 to 200 | Results per page |
| `multilingual` | `true` / `false` | Whether to enable substring matching for non-ASCII keywords (CJK, Cyrillic, Arabic, etc.) |

### 4.3 `workflow`

```json
{
  "actions": [
    { "id": "rebuild-index", "title": "重建搜索索引", "command": "rebuild_fts_index" }
  ]
}
```

`command` must be a command name already registered by the host (for example `open_database_preview`, `rebuild_fts_index`). Unknown commands are never executed; they are only shown in a disabled state.

## 5. Import, export and built-in packages

- **Import**: Settings → Component Packages → Import, then choose a `.json` file. The schema, id, version, kind, component count and config size are validated; invalid packages are rejected with a reason.
- **Export**: Any package (including built-in ones) can be exported as JSON. Imported packages can be re-imported unchanged; an exported built-in package only serves as an authoring template — change its `id` before re-importing (built-in ids are reserved).
- **Built-in default packages**: three packages ship with the app as part of the binary:
  - `fyf.appearance.classic` — classic blue appearance
  - `fyf.search.balanced` — relevance first, 50 results per page, multilingual matching
  - `fyf.workflow.quicklaunch` — maintenance quick actions
  Built-in packages can be enabled or disabled but **cannot be deleted or overwritten** (their ids are reserved).
- **Storage**: imported packages live in `components/packages/<id>.json` inside the application data directory, and their enabled state lives in `components/state.json`.

## 6. Security constraints

1. Component packages are **declarative**: they contain no scripts, executables or remote resource references, and the host never executes strings from a package.
2. A single file is limited to 1 MiB, a single component config to 256 KiB, and a package to at most 32 components.
3. Package ids must be unique; built-in ids cannot be taken.
4. Unknown fields and unknown `kind` values are ignored or shown read-only and never crash the application.

## 7. Minimal example: a custom violet theme

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

Save it as `violet-theme.json` and import it inside the app.
