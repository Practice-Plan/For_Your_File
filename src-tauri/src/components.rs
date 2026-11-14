//! Component package system (v0.0.5).
//!
//! Features are exposed as declarative component packages. A package is a
//! plain JSON document described by [`ComponentPackage`]; the host interprets
//! its `components[].config` values — no executable code is loaded, which keeps
//! third-party packages safe to import.
//!
//! Storage layout (inside the application data directory):
//!
//! ```text
//! components/
//!   state.json            # { "enabled": { "<package id>": true|false } }
//!   packages/<id>.json    # imported packages
//! ```
//!
//! Three packages ship inside the binary as *internal defaults* (see
//! `src/builtin/*.json`); they can be enabled or disabled but never deleted or
//! overwritten.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Manager};

/// Only schema 1 is understood by this build.
const SCHEMA_VERSION: u32 = 1;
/// Hard cap for an imported package file (1 MiB).
const MAX_PACKAGE_BYTES: u64 = 1024 * 1024;
/// Cap for a single component's serialized config (256 KiB).
const MAX_COMPONENT_BYTES: usize = 256 * 1024;
/// Allowed package kinds.
const ALLOWED_KINDS: [&str; 4] = ["theme", "search", "workflow", "bundle"];

const BUILTIN_APPEARANCE: &str = include_str!("builtin/appearance-default.json");
const BUILTIN_SEARCH: &str = include_str!("builtin/search-balanced.json");
const BUILTIN_WORKFLOW: &str = include_str!("builtin/workflow-quicklaunch.json");

/// 结构化错误：只返回稳定错误码与插值参数，由展示层翻译成当前语言。
#[derive(Debug, Clone, Serialize)]
pub struct AppError {
    pub code: String,
    #[serde(skip_serializing_if = "serde_json::Map::is_empty")]
    pub params: serde_json::Map<String, serde_json::Value>,
}

impl AppError {
    pub fn new(code: &str) -> Self {
        Self {
            code: code.to_string(),
            params: serde_json::Map::new(),
        }
    }

    pub fn param(mut self, key: &str, value: impl Into<serde_json::Value>) -> Self {
        self.params.insert(key.to_string(), value.into());
        self
    }
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.code)
    }
}

/// A single declarative feature inside a package.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentSpec {
    pub id: String,
    pub kind: String,
    pub title: String,
    /// 本地化标题的文案键；存在时界面优先使用它。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title_key: Option<String>,
    #[serde(default)]
    pub config: Value,
}

/// A component package (manifest + components).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentPackage {
    pub schema: u32,
    pub id: String,
    pub name: String,
    /// 本地化名称的文案键；存在时界面优先使用它。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name_key: Option<String>,
    pub version: String,
    #[serde(default)]
    pub description: String,
    /// 本地化描述的文案键；存在时界面优先使用它。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description_key: Option<String>,
    #[serde(default)]
    pub author: String,
    #[serde(default)]
    pub kind: String,
    #[serde(default)]
    pub components: Vec<ComponentSpec>,
    /// True for packages embedded in the binary. Not read from JSON.
    #[serde(default, skip_deserializing)]
    pub builtin: bool,
    /// Resolved from `state.json` when listing. Not read from JSON.
    #[serde(default, skip_deserializing)]
    pub enabled: bool,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct PackageState {
    #[serde(default)]
    enabled: std::collections::HashMap<String, bool>,
}

// ---------------------------------------------------------------------------
// Paths & state
// ---------------------------------------------------------------------------

fn components_dir(app_handle: &AppHandle) -> Result<PathBuf, AppError> {
    let base = crate::ppc_linker::resolve_data_dir(app_handle).map_err(|e| {
        log::warn!("Cannot resolve data directory: {}", e);
        AppError::new("components.writeFailed")
    })?;
    let dir = base.join("components");
    std::fs::create_dir_all(dir.join("packages")).map_err(|e| {
        log::warn!("Cannot create component directory: {}", e);
        AppError::new("components.writeFailed")
    })?;
    Ok(dir)
}

fn state_path(app_handle: &AppHandle) -> Result<PathBuf, AppError> {
    Ok(components_dir(app_handle)?.join("state.json"))
}

fn read_state(app_handle: &AppHandle) -> PackageState {
    let Ok(path) = state_path(app_handle) else {
        return PackageState::default();
    };
    std::fs::read_to_string(path)
        .ok()
        .and_then(|text| serde_json::from_str(&text).ok())
        .unwrap_or_default()
}

fn write_state(app_handle: &AppHandle, state: &PackageState) -> Result<(), AppError> {
    let path = state_path(app_handle)?;
    let text = serde_json::to_string_pretty(state).map_err(|e| {
        log::warn!("Cannot serialize component state: {}", e);
        AppError::new("components.writeFailed")
    })?;
    std::fs::write(path, text).map_err(|e| {
        log::warn!("Cannot save component state: {}", e);
        AppError::new("components.writeFailed")
    })
}

// ---------------------------------------------------------------------------
// Loading & validation
// ---------------------------------------------------------------------------

fn builtin_packages() -> Result<Vec<ComponentPackage>, AppError> {
    [BUILTIN_APPEARANCE, BUILTIN_SEARCH, BUILTIN_WORKFLOW]
        .iter()
        .map(|raw| {
            let mut package: ComponentPackage = serde_json::from_str(raw).map_err(|e| {
                log::error!("Built-in component package is invalid: {}", e);
                AppError::new("components.builtinInvalid")
            })?;
            package.builtin = true;
            Ok(package)
        })
        .collect()
}

fn load_installed(app_handle: &AppHandle) -> Result<Vec<ComponentPackage>, AppError> {
    let dir = components_dir(app_handle)?.join("packages");
    let mut packages = Vec::new();

    let entries = match std::fs::read_dir(&dir) {
        Ok(entries) => entries,
        Err(e) => {
            log::warn!("Cannot read component package directory: {}", e);
            return Ok(packages);
        }
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
            continue;
        }
        match std::fs::read_to_string(&path)
            .map_err(|e| e.to_string())
            .and_then(|text| {
                serde_json::from_str::<ComponentPackage>(&text).map_err(|e| e.to_string())
            }) {
            Ok(mut package) => {
                package.builtin = false;
                packages.push(package);
            }
            Err(e) => log::warn!(
                "Skipping invalid component package {}: {}",
                path.display(),
                e
            ),
        }
    }

    packages.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(packages)
}

fn validate_package(package: &ComponentPackage) -> Result<(), AppError> {
    if package.schema != SCHEMA_VERSION {
        return Err(AppError::new("components.schemaUnsupported")
            .param("actual", package.schema)
            .param("expected", SCHEMA_VERSION));
    }
    validate_id(&package.id)?;

    if package.name.trim().is_empty() || package.name.chars().count() > 100 {
        return Err(AppError::new("components.nameInvalid"));
    }
    if !is_valid_version(&package.version) {
        return Err(
            AppError::new("components.versionInvalid").param("version", package.version.clone())
        );
    }
    if !package.kind.is_empty() && !ALLOWED_KINDS.contains(&package.kind.as_str()) {
        return Err(AppError::new("components.kindUnsupported")
            .param("kind", package.kind.clone())
            .param("kinds", ALLOWED_KINDS.join(", ")));
    }
    if package.components.len() > 32 {
        return Err(AppError::new("components.tooManyComponents").param("max", 32));
    }

    for component in &package.components {
        if component.id.trim().is_empty() || component.id.chars().count() > 64 {
            return Err(AppError::new("components.componentIdInvalid"));
        }
        if component.kind.trim().is_empty() {
            return Err(
                AppError::new("components.componentKindMissing").param("id", component.id.clone())
            );
        }
        let size = serde_json::to_string(&component.config)
            .map(|text| text.len())
            .unwrap_or(0);
        if size > MAX_COMPONENT_BYTES {
            return Err(AppError::new("components.componentTooLarge")
                .param("id", component.id.clone())
                .param("size", size)
                .param("max", MAX_COMPONENT_BYTES));
        }
    }

    Ok(())
}

fn validate_id(id: &str) -> Result<(), AppError> {
    let mut chars = id.chars();
    let first_ok = chars
        .next()
        .map(|c| c.is_ascii_alphanumeric())
        .unwrap_or(false);
    let rest_ok = chars.all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_'));
    if !first_ok || !rest_ok || id.len() < 3 || id.len() > 64 {
        return Err(AppError::new("components.idInvalid").param("id", id));
    }
    Ok(())
}

fn is_valid_version(version: &str) -> bool {
    let parts: Vec<&str> = version.split('.').collect();
    (1..=4).contains(&parts.len())
        && parts
            .iter()
            .all(|part| !part.is_empty() && part.chars().all(|c| c.is_ascii_digit()))
}

/// Resolve a package by id from either the built-in set or disk.
fn find_package(app_handle: &AppHandle, id: &str) -> Result<ComponentPackage, AppError> {
    let mut packages = builtin_packages()?;
    packages.extend(load_installed(app_handle)?);
    packages
        .into_iter()
        .find(|package| package.id == id)
        .ok_or_else(|| AppError::new("components.notFound").param("id", id))
}

// ---------------------------------------------------------------------------
// Tauri commands
// ---------------------------------------------------------------------------

/// List built-in and imported component packages with their enabled state.
#[tauri::command]
pub fn list_component_packages(app_handle: AppHandle) -> Result<Vec<ComponentPackage>, AppError> {
    let state = read_state(&app_handle);

    let mut packages = builtin_packages()?;
    packages.extend(load_installed(&app_handle)?);

    for package in &mut packages {
        // Built-ins are enabled by default; imported packages start enabled
        // once, then follow the persisted state.
        package.enabled = state.enabled.get(&package.id).copied().unwrap_or(true);
    }

    Ok(packages)
}

/// Import a package from a JSON file chosen by the user.
#[tauri::command]
pub fn import_component_package(
    app_handle: AppHandle,
    path: String,
) -> Result<ComponentPackage, AppError> {
    let source = Path::new(&path);
    let metadata = std::fs::metadata(source).map_err(|e| {
        log::warn!("Cannot read component package '{}': {}", path, e);
        AppError::new("components.readFailed").param("path", path.clone())
    })?;
    if !metadata.is_file() {
        return Err(AppError::new("components.notAFile").param("path", path));
    }
    if metadata.len() > MAX_PACKAGE_BYTES {
        return Err(AppError::new("components.tooLarge")
            .param("size", metadata.len())
            .param("max", MAX_PACKAGE_BYTES));
    }

    let text = std::fs::read_to_string(source).map_err(|e| {
        log::warn!("Cannot read component package '{}': {}", path, e);
        AppError::new("components.readFailed").param("path", path.clone())
    })?;
    let mut package: ComponentPackage = serde_json::from_str(&text).map_err(|e| {
        log::warn!("Invalid component package JSON: {}", e);
        AppError::new("components.invalidJson")
    })?;
    validate_package(&package)?;

    // Built-in ids are reserved so the internal defaults cannot be shadowed.
    let builtin_ids: Vec<String> = builtin_packages()?.into_iter().map(|p| p.id).collect();
    if builtin_ids.contains(&package.id) {
        return Err(AppError::new("components.reservedId").param("id", package.id.clone()));
    }

    let target = components_dir(&app_handle)?
        .join("packages")
        .join(format!("{}.json", package.id));
    let pretty = serde_json::to_string_pretty(&package).map_err(|e| {
        log::warn!("Cannot serialize component package: {}", e);
        AppError::new("components.writeFailed")
    })?;
    std::fs::write(&target, pretty).map_err(|e| {
        log::warn!("Cannot save component package: {}", e);
        AppError::new("components.writeFailed")
    })?;

    package.builtin = false;
    package.enabled = read_state(&app_handle)
        .enabled
        .get(&package.id)
        .copied()
        .unwrap_or(true);

    log::info!("Imported component package '{}' from {}", package.id, path);
    Ok(package)
}

/// Export a package (built-in or imported) to a JSON file.
#[tauri::command]
pub fn export_component_package(
    app_handle: AppHandle,
    id: String,
    path: String,
) -> Result<(), AppError> {
    let package = find_package(&app_handle, &id)?;
    let text = serde_json::to_string_pretty(&package).map_err(|e| {
        log::warn!("Cannot serialize component package: {}", e);
        AppError::new("components.writeFailed")
    })?;
    std::fs::write(&path, text).map_err(|e| {
        log::warn!("Cannot write component package '{}': {}", path, e);
        AppError::new("components.writeFailed")
    })?;
    log::info!("Exported component package '{}' to {}", id, path);
    Ok(())
}

/// Enable or disable a package. Built-in packages cannot be deleted but can be
/// toggled like any other package.
#[tauri::command]
pub fn set_component_package_enabled(
    app_handle: AppHandle,
    id: String,
    enabled: bool,
) -> Result<(), AppError> {
    // Ensure the package exists before persisting state for it.
    find_package(&app_handle, &id)?;

    let mut state = read_state(&app_handle);
    state.enabled.insert(id, enabled);
    write_state(&app_handle, &state)
}

/// Delete an imported package. Built-in packages are rejected.
#[tauri::command]
pub fn delete_component_package(app_handle: AppHandle, id: String) -> Result<(), AppError> {
    validate_id(&id)?;

    if builtin_packages()?.iter().any(|p| p.id == id) {
        return Err(AppError::new("components.builtinReadonly").param("id", id));
    }

    let target = components_dir(&app_handle)?
        .join("packages")
        .join(format!("{}.json", id));
    if target.exists() {
        std::fs::remove_file(&target).map_err(|e| {
            log::warn!("Cannot delete component package '{}': {}", id, e);
            AppError::new("components.deleteFailed").param("id", id.clone())
        })?;
    }

    let mut state = read_state(&app_handle);
    state.enabled.remove(&id);
    write_state(&app_handle, &state)?;

    log::info!("Deleted component package '{}'", id);
    Ok(())
}

/// Return the authoring guide in the requested language (Markdown).
#[tauri::command]
pub fn get_component_creation_guide(locale: Option<String>, app: tauri::AppHandle) -> String {
    let lang = locale.unwrap_or_else(|| {
        app.try_state::<crate::i18n::AppLanguage>()
            .map(|state| state.current())
            .unwrap_or_else(|| "en".to_string())
    });
    crate::guide::for_language(&lang)
}
