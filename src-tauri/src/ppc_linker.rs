//! PPC (Programmable Processing Core) Linker
//!
//! Connects the For_Your_File application to the PPC Central Processing System
//! running on 127.0.0.1:9527. Handles:
//! - TCP connection management
//! - App registration (REGISTER_APP) and optional authentication (AUTH)
//! - PPC version verification (v0.0.9 ~ v0.1.1)
//! - PPC error code mapping to human-readable messages
//! - Command sending and response parsing
//!
//! Since v0.0.5 the PPC process is started by the standalone launcher
//! (`launcher/`, written in Go) instead of this module. The app only connects,
//! and every user-facing error is surfaced through PPC's `WINDOW ERROR`
//! command — there is deliberately no in-app fallback dialog.
//!
//! This module does NOT modify any PPC source files. It acts purely as a
//! client that communicates with PPC over TCP.

use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::Mutex;
use std::time::Duration;
use tauri::Manager;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// PPC server address (localhost only, as PPC binds to 127.0.0.1).
const PPC_HOST: &str = "127.0.0.1";
/// PPC server port.
const PPC_PORT: u16 = 9527;

/// The app ID under which For_Your_File registers with PPC.
const APP_ID: &str = "fyf";

/// For_Your_File's own version, sent during registration.
const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Minimum supported PPC version.
const PPC_MIN_VERSION: &str = "0.0.9";
/// Maximum supported PPC version.
///
/// v0.1.1 is the current stable PPC release: tray/terminal dual mode,
/// `ppc`/`ppc32` mutual exclusion, per-session logging and globally relaxed
/// signatures. `AUTH` remains supported but is optional there.
const PPC_MAX_VERSION: &str = "0.1.1";

/// TCP read/write timeout in seconds.
const TIMEOUT_SECS: u64 = 5;

// ---------------------------------------------------------------------------
// Connection state (held in a Tauri State)
// ---------------------------------------------------------------------------

/// Holds the authenticated session state after a successful registration +
/// AUTH cycle. The hash is needed to re-authenticate on new TCP connections
/// (PPC requires AUTH on every new TCP connection).
#[derive(Clone, serde::Serialize)]
pub struct PpcSession {
    /// Whether PPC is currently connected and authenticated.
    pub connected: bool,
    /// The app hash returned by PPC during registration (used for AUTH).
    pub app_hash: Option<String>,
    /// The PPC version reported by the server.
    pub ppc_version: Option<String>,
    /// Human-readable status message.
    pub status_message: String,
    /// Last error code from PPC (hex string like "0x10000").
    pub last_error_code: Option<String>,
}

impl Default for PpcSession {
    fn default() -> Self {
        Self {
            connected: false,
            app_hash: None,
            ppc_version: None,
            status_message: "Not connected".to_string(),
            last_error_code: None,
        }
    }
}

/// Tauri-managed state wrapping the session.
pub struct PpcState {
    pub session: Mutex<PpcSession>,
}

impl PpcState {
    pub fn new() -> Self {
        Self {
            session: Mutex::new(PpcSession::default()),
        }
    }
}

// ---------------------------------------------------------------------------
// Error code mapping
// ---------------------------------------------------------------------------

/// Map a PPC hex status code to a human-readable message in English.
///
/// The mapping follows the `app_code.json` definitions in the PPC project.
/// Unknown codes return a generic message.
pub fn map_ppc_code(code: &str) -> String {
    match code {
        // Success codes (0x0xxxx)
        "0x00000" => "Success".to_string(),
        "0x00001" => "App registered successfully".to_string(),
        "0x00002" => "App updated successfully".to_string(),
        "0x00003" => "DLL loaded successfully".to_string(),
        "0x00004" => "Command executed successfully".to_string(),

        // Error codes (0x1xxxx)
        "0x10000" => "Unknown error".to_string(),
        "0x10001" => "Unsupported command".to_string(),
        "0x10002" => "Resource not found".to_string(),
        "0x10003" => "Signature verification failed".to_string(),
        "0x10004" => "Hash mismatch".to_string(),
        "0x10005" => "Registration failed".to_string(),
        "0x10006" => "Update failed".to_string(),
        "0x10007" => "App not registered or not authenticated".to_string(),
        "0x10008" => "Invalid parameters".to_string(),
        "0x10009" => "File not found".to_string(),
        "0x10010" => "Permission denied".to_string(),
        "0x10011" => "DLL not found".to_string(),
        "0x10012" => "DLL load failed".to_string(),
        "0x10013" => "Config parse failed".to_string(),
        "0x10014" => "Database error".to_string(),
        "0x10015" => "Connection failed".to_string(),
        "0x10016" => "Timeout".to_string(),
        "0x10017" => "PPC server not running".to_string(),
        "0x10018" => "PPC authentication failed".to_string(),
        "0x10019" => "PPC version not supported".to_string(),
        "0x10020" => "PPC response invalid".to_string(),
        "0x10021" => "Window pin failed".to_string(),
        "0x10022" => "Window not found".to_string(),
        "0x10023" => "Hotkey registration failed".to_string(),
        "0x10024" => "Config save failed".to_string(),
        "0x10025" => "Config load failed".to_string(),
        "0x10026" => "State persistence failed".to_string(),
        "0x10027" => "Administrator privileges required".to_string(),
        "0x10028" => "Localization load failed".to_string(),
        "0x10029" => "Single instance violation".to_string(),
        "0x10030" => "Hook installation failed".to_string(),
        "0x10031" => "Log write failed".to_string(),
        "0x10032" => "DLL call failed".to_string(),

        // Status codes (0x2xxxx)
        "0x20000" => "Waiting for signature".to_string(),
        "0x20001" => "Waiting for app info".to_string(),
        "0x20002" => "Processing".to_string(),
        "0x20003" => "Initializing".to_string(),
        "0x20004" => "Loading config".to_string(),
        "0x20005" => "Executing command".to_string(),
        "0x20006" => "Connecting".to_string(),
        "0x20007" => "Registering".to_string(),
        "0x20008" => "Authenticating".to_string(),
        "0x20009" => "Reconnecting".to_string(),

        // Warning codes (0x3xxxx)
        "0x30000" => "Deprecated command warning".to_string(),
        "0x30001" => "Old version warning".to_string(),
        "0x30002" => "High memory usage warning".to_string(),
        "0x30003" => "PPC reconnect warning".to_string(),
        "0x30004" => "Fallback behavior warning".to_string(),
        "0x30005" => "Language fallback warning".to_string(),

        _ => format!("Unknown status code: {}", code),
    }
}

/// Check if a PPC code represents success (starts with "0x0").
pub fn is_success_code(code: &str) -> bool {
    code.to_lowercase().starts_with("0x0")
}

/// Check if a PPC code represents an error (starts with "0x1").
#[allow(dead_code)]
pub fn is_error_code(code: &str) -> bool {
    code.to_lowercase().starts_with("0x1")
}

// ---------------------------------------------------------------------------
// Version comparison
// ---------------------------------------------------------------------------

/// Parse a version string "x.y.z" into a Vec<u32>.
fn parse_version(v: &str) -> Vec<u32> {
    v.split('.').filter_map(|s| s.parse::<u32>().ok()).collect()
}

/// Check if a PPC version is within the supported range.
/// Only versions between 0.0.9 and 0.1.1 are supported.
pub fn is_version_supported(version: &str) -> bool {
    let v = parse_version(version);
    let min = parse_version(PPC_MIN_VERSION);
    let max = parse_version(PPC_MAX_VERSION);

    if v.len() != 3 || min.len() != 3 || max.len() != 3 {
        // Fallback to string comparison if parsing fails
        return version == PPC_MIN_VERSION;
    }

    // Compare: min <= v <= max
    let v_parts = (v[0], v[1], v[2]);
    let min_parts = (min[0], min[1], min[2]);
    let max_parts = (max[0], max[1], max[2]);

    v_parts >= min_parts && v_parts <= max_parts
}

// ---------------------------------------------------------------------------
// TCP communication
// ---------------------------------------------------------------------------

/// Send a command to PPC and return the raw response string.
fn send_command(command: &str) -> Result<String, String> {
    let addr = format!("{}:{}", PPC_HOST, PPC_PORT);
    let mut stream = TcpStream::connect_timeout(
        &addr
            .parse()
            .map_err(|e| format!("Invalid address: {}", e))?,
        Duration::from_secs(TIMEOUT_SECS),
    )
    .map_err(|e| format!("Failed to connect to PPC ({}): {}", addr, e))?;

    stream
        .set_read_timeout(Some(Duration::from_secs(TIMEOUT_SECS)))
        .map_err(|e| format!("Failed to set read timeout: {}", e))?;
    stream
        .set_write_timeout(Some(Duration::from_secs(TIMEOUT_SECS)))
        .map_err(|e| format!("Failed to set write timeout: {}", e))?;

    // Send command (PPC reads raw bytes, no line-ending required but \n is safe)
    let cmd_bytes = format!("{}\n", command);
    stream
        .write_all(cmd_bytes.as_bytes())
        .map_err(|e| format!("Failed to send command: {}", e))?;

    // Read response
    let mut buf = [0u8; 4096];
    let n = stream
        .read(&mut buf)
        .map_err(|e| format!("Failed to read response: {}", e))?;

    String::from_utf8(buf[..n].to_vec()).map_err(|e| format!("UTF-8 decode error: {}", e))
}

/// Parse the first line of a PPC response as the status code.
fn parse_status_code(response: &str) -> Option<String> {
    response.lines().next().map(|line| line.trim().to_string())
}

/// Parse the second line of a PPC response (often the payload).
fn parse_payload(response: &str) -> Option<String> {
    response.lines().nth(1).map(|line| line.trim().to_string())
}

// ---------------------------------------------------------------------------
// Registration & Authentication
// ---------------------------------------------------------------------------

/// Get the current executable path for registration.
fn get_exe_path() -> String {
    std::env::current_exe()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| ".".to_string())
}

/// Register the For_Your_File app with PPC.
/// Returns the app hash on success.
pub fn register_with_ppc() -> Result<String, String> {
    let exe_path = get_exe_path();
    let command = format!("REGISTER_APP {}|{}|{}", APP_ID, APP_VERSION, exe_path);
    log::info!("PPC register: {}", command);

    let response = send_command(&command)?;
    let code = parse_status_code(&response).unwrap_or_default();

    if is_success_code(&code) {
        // Payload is the hash on the second line
        let hash = parse_payload(&response)
            .ok_or_else(|| format!("Registration succeeded but no hash returned: {}", response))?;
        log::info!("PPC registration succeeded, hash={}", hash);
        Ok(hash)
    } else {
        let msg = map_ppc_code(&code);
        Err(format!("PPC registration failed: {} ({})", msg, code))
    }
}

/// Authenticate with PPC using the stored hash.
/// Returns Ok(()) on success.
pub fn authenticate_with_ppc(hash: &str) -> Result<(), String> {
    let command = format!("AUTH {}", hash);
    let response = send_command(&command)?;
    let code = parse_status_code(&response).unwrap_or_default();

    if is_success_code(&code) {
        log::info!("PPC authentication succeeded");
        Ok(())
    } else {
        let msg = map_ppc_code(&code);
        Err(format!("PPC authentication failed: {} ({})", msg, code))
    }
}

/// PPC 版本检查失败的原因。
pub enum PpcVersionError {
    /// PPC 返回了不受支持的版本号。
    Unsupported(String),
    /// 无法查询或解析版本（诊断信息仅用于日志）。
    Query(String),
}

/// Query the PPC version and verify it's within the supported range.
pub fn check_ppc_version() -> Result<String, PpcVersionError> {
    let response = send_command("PPCVERSION").map_err(PpcVersionError::Query)?;
    let code = parse_status_code(&response).unwrap_or_default();

    if !is_success_code(&code) {
        return Err(PpcVersionError::Query(format!(
            "Failed to query PPC version: {} ({})",
            map_ppc_code(&code),
            code
        )));
    }

    // Response format: "0x00000\nPPC Version: 0.0.7\n"
    // Extract version from the payload line
    let payload = parse_payload(&response).unwrap_or_default();
    let version = payload
        .replace("PPC Version:", "")
        .replace("PPC version:", "")
        .trim()
        .to_string();

    if version.is_empty() {
        return Err(PpcVersionError::Query(format!(
            "Cannot parse PPC version: {}",
            response
        )));
    }

    if !is_version_supported(&version) {
        return Err(PpcVersionError::Unsupported(version));
    }

    log::info!("PPC version check passed: {}", version);
    Ok(version)
}

/// Ping PPC to check if it's running.
pub fn ping_ppc() -> Result<bool, String> {
    match send_command("PING") {
        Ok(response) => {
            let code = parse_status_code(&response).unwrap_or_default();
            Ok(is_success_code(&code))
        }
        Err(_) => Ok(false),
    }
}

// ---------------------------------------------------------------------------
// Data directory resolution (PPC-managed config path)
// ---------------------------------------------------------------------------

/// Cached PPC base path once successfully resolved, so we don't repeat the
/// register→auth→PPCPATH round-trip on every data-dir lookup.
static PPC_BASE_PATH_CACHE: Mutex<Option<std::path::PathBuf>> = Mutex::new(None);

/// Cooldown deadline after a failed PPC connection attempt.
/// When the current time is before this instant, skip the connection attempt
/// to avoid repeatedly blocking on an unavailable PPC server during startup.
static PPC_FAIL_COOLDOWN: Mutex<Option<std::time::Instant>> = Mutex::new(None);

/// Duration to wait before retrying a failed PPC connection (30 seconds).
const PPC_FAIL_COOLDOWN_SECS: u64 = 30;

/// Query the PPC installation directory via the PPCPATH command.
///
/// PPCPATH is not a public command, so this performs REGISTER_APP → AUTH →
/// PPCPATH on a single TCP connection. Returns the PPC base directory
/// (the directory containing ppc.exe).
#[allow(dead_code)]
fn query_ppc_base_path() -> Result<std::path::PathBuf, String> {
    // Fast path: cached value from a previous successful resolution
    if let Some(cached) = PPC_BASE_PATH_CACHE.lock().unwrap().as_ref() {
        return Ok(cached.clone());
    }

    // Check cooldown: skip connection attempt if we recently failed
    {
        let cooldown = PPC_FAIL_COOLDOWN.lock().unwrap();
        if let Some(deadline) = cooldown.as_ref() {
            if std::time::Instant::now() < *deadline {
                log::debug!("PPC connection in cooldown, skipping query");
                return Err("PPC not reachable (cooldown active)".to_string());
            }
        }
    }

    let addr = format!("{}:{}", PPC_HOST, PPC_PORT);
    // Use a short connection timeout (500ms) to avoid blocking app startup
    // when PPC is not yet running. Once connected, read/write timeouts
    // remain at TIMEOUT_SECS for reliable command exchange.
    let connect_result = TcpStream::connect_timeout(
        &addr
            .parse()
            .map_err(|e| format!("Invalid address: {}", e))?,
        Duration::from_millis(500),
    );

    let mut stream = match connect_result {
        Ok(stream) => stream,
        Err(e) => {
            // Record cooldown on failure so subsequent calls don't block
            *PPC_FAIL_COOLDOWN.lock().unwrap() =
                Some(std::time::Instant::now() + Duration::from_secs(PPC_FAIL_COOLDOWN_SECS));
            return Err(format!("PPC not reachable: {}", e));
        }
    };

    stream
        .set_read_timeout(Some(Duration::from_secs(TIMEOUT_SECS)))
        .map_err(|e| format!("Failed to set read timeout: {}", e))?;
    stream
        .set_write_timeout(Some(Duration::from_secs(TIMEOUT_SECS)))
        .map_err(|e| format!("Failed to set write timeout: {}", e))?;

    let mut buf = [0u8; 4096];

    // Step 1: REGISTER_APP (public command, no auth required)
    let exe_path = get_exe_path();
    let register_cmd = format!("REGISTER_APP {}|{}|{}\n", APP_ID, APP_VERSION, exe_path);
    stream
        .write_all(register_cmd.as_bytes())
        .map_err(|e| format!("Failed to send REGISTER_APP: {}", e))?;
    let n = stream
        .read(&mut buf)
        .map_err(|e| format!("Failed to read REGISTER_APP response: {}", e))?;
    let response = String::from_utf8_lossy(&buf[..n]).to_string();
    let code = parse_status_code(&response).unwrap_or_default();
    if !is_success_code(&code) {
        let msg = map_ppc_code(&code);
        return Err(format!("PPC register failed: {} ({})", msg, code));
    }
    let hash = parse_payload(&response).unwrap_or_default();
    if hash.is_empty() {
        return Err("PPC register returned empty hash".to_string());
    }

    // Step 2: AUTH on the same connection
    let auth_cmd = format!("AUTH {}\n", hash);
    stream
        .write_all(auth_cmd.as_bytes())
        .map_err(|e| format!("Failed to send AUTH: {}", e))?;
    let n = stream
        .read(&mut buf)
        .map_err(|e| format!("Failed to read AUTH response: {}", e))?;
    let response = String::from_utf8_lossy(&buf[..n]).to_string();
    let code = parse_status_code(&response).unwrap_or_default();
    if !is_success_code(&code) {
        let msg = map_ppc_code(&code);
        return Err(format!("PPC auth failed: {} ({})", msg, code));
    }

    // Step 3: PPCPATH on the same connection
    stream
        .write_all(b"PPCPATH\n")
        .map_err(|e| format!("Failed to send PPCPATH: {}", e))?;
    let n = stream
        .read(&mut buf)
        .map_err(|e| format!("Failed to read PPCPATH response: {}", e))?;
    let response = String::from_utf8_lossy(&buf[..n]).to_string();
    let code = parse_status_code(&response).unwrap_or_default();
    if !is_success_code(&code) {
        let msg = map_ppc_code(&code);
        return Err(format!("Failed to query PPC path: {} ({})", msg, code));
    }
    let path = parse_payload(&response)
        .unwrap_or_default()
        .trim()
        .to_string();
    if path.is_empty() {
        return Err("Cannot parse PPC path from response".to_string());
    }

    let path_buf = std::path::PathBuf::from(path);
    log::info!("PPC base path resolved: {}", path_buf.display());

    // Cache for subsequent lookups and clear any failure cooldown
    *PPC_BASE_PATH_CACHE.lock().unwrap() = Some(path_buf.clone());
    *PPC_FAIL_COOLDOWN.lock().unwrap() = None;

    Ok(path_buf)
}

/// Resolve the application data directory for For_Your_File.
///
/// All mutable application data is stored under:
///   `%APPDATA%/wang.station/app/For_Your_File`
///
/// PPC remains available as a communication service, but it does not control
/// this application's local paths.
pub fn resolve_data_dir(app_handle: &tauri::AppHandle) -> Result<std::path::PathBuf, String> {
    let app_data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))?;
    let app_data_dir = app_data_dir
        .parent()
        .ok_or_else(|| "Failed to resolve the roaming app data directory".to_string())?
        .join("wang.station")
        .join("app")
        .join("For_Your_File");

    std::fs::create_dir_all(&app_data_dir)
        .map_err(|e| format!("Failed to create data directory: {}", e))?;

    log::info!(
        "Data directory resolved to wang.station path: {}",
        app_data_dir.display()
    );
    Ok(app_data_dir)
}

// ---------------------------------------------------------------------------
// Error reporting (PPC-only, no fallback)
// ---------------------------------------------------------------------------

/// Report a user-facing error through PPC.
///
/// Every error path in this application funnels through here: PPC shows a
/// native `WINDOW ERROR` dialog on the host. There is intentionally **no**
/// in-app fallback dialog (v0.0.5 requirement) — if PPC cannot be reached the
/// error is only written to the application log.
pub fn report_error(message: &str) -> bool {
    if show_ppc_error_window(message) {
        return true;
    }
    log::error!("PPC unreachable, error not displayed: {}", message);
    false
}

/// Send a WINDOW ERROR command to PPC to show a native error dialog.
///
/// This uses the TCP protocol documented in main/src/window.rs:
///   `WINDOW ERROR <message>`
///
/// Returns true if the command was sent successfully.
pub fn show_ppc_error_window(message: &str) -> bool {
    // PPC must be running for this to work. We use a short timeout since
    // this is best-effort — if PPC isn't up, we silently fall back.
    let addr = format!("{}:{}", PPC_HOST, PPC_PORT);
    let mut stream = match TcpStream::connect_timeout(
        &addr
            .parse()
            .unwrap_or_else(|_| "127.0.0.1:9527".parse().unwrap()),
        Duration::from_millis(2000),
    ) {
        Ok(s) => s,
        Err(e) => {
            log::debug!("Cannot reach PPC for error window: {}", e);
            return false;
        }
    };

    // PPC window commands are globally signature-relaxed (no auth needed).
    let cmd = format!("WINDOW ERROR {}\n", message);
    match stream.write_all(cmd.as_bytes()) {
        Ok(_) => {
            log::info!("Sent WINDOW ERROR to PPC: {}", message);
            true
        }
        Err(e) => {
            log::debug!("Failed to send WINDOW ERROR: {}", e);
            false
        }
    }
}

/// Send a WINDOW WARN command to PPC to show a native warning dialog.
#[allow(dead_code)]
pub fn show_ppc_warn_window(message: &str) -> bool {
    let addr = format!("{}:{}", PPC_HOST, PPC_PORT);
    let mut stream = match TcpStream::connect_timeout(
        &addr
            .parse()
            .unwrap_or_else(|_| "127.0.0.1:9527".parse().unwrap()),
        Duration::from_millis(2000),
    ) {
        Ok(s) => s,
        Err(e) => {
            log::debug!("Cannot reach PPC for warn window: {}", e);
            return false;
        }
    };

    let cmd = format!("WINDOW WARN {}\n", message);
    match stream.write_all(cmd.as_bytes()) {
        Ok(_) => {
            log::info!("Sent WINDOW WARN to PPC: {}", message);
            true
        }
        Err(e) => {
            log::debug!("Failed to send WINDOW WARN: {}", e);
            false
        }
    }
}

// ---------------------------------------------------------------------------
// Tauri commands
// ---------------------------------------------------------------------------

/// Connect to an already-running PPC instance.
///
/// v0.0.5: PPC is started by the standalone launcher (`launcher/`, Go) before
/// this app runs, so this command never spawns a process and has no fallback.
/// It verifies connectivity, checks the version (0.0.9 ~ 0.1.1), registers the
/// app and attempts `AUTH`.
///
/// `AUTH` is optional on PPC v0.1.1 (global signature relaxation). A failed
/// AUTH therefore downgrades the session to "connected, unauthenticated"
/// instead of aborting — all commands remain available.
#[tauri::command]
pub async fn ppc_connect_auto(
    app_handle: tauri::AppHandle,
    state: tauri::State<'_, PpcState>,
) -> Result<PpcSession, String> {
    // 错误文案统一从语言表取，代码里不写死任何一种语言。
    let lang = app_handle
        .try_state::<crate::i18n::AppLanguage>()
        .map(|state| state.current())
        .unwrap_or_else(|| "en".to_string());

    log::info!("PPC connect: probing 127.0.0.1:9527");
    if !ping_ppc().unwrap_or(false) {
        let msg = crate::i18n::t(&lang, "errors.ppc.notRunning", &[]);
        log::error!("{}", msg);
        let mut session = state.session.lock().map_err(|e| e.to_string())?;
        session.connected = false;
        session.status_message = msg.clone();
        session.last_error_code = Some("0x10017".to_string());
        // PPC 不可达，没有任何展示回退：只记录日志。
        return Err(msg);
    }

    // ── Version check ────────────────────────────────────────────────
    let ppc_version = match check_ppc_version() {
        Ok(version) => version,
        Err(PpcVersionError::Unsupported(version)) => {
            let msg = crate::i18n::t(
                &lang,
                "errors.ppc.versionUnsupported",
                &[
                    ("version", &version),
                    ("min", PPC_MIN_VERSION),
                    ("max", PPC_MAX_VERSION),
                ],
            );
            log::error!("{}", msg);
            let mut session = state.session.lock().map_err(|e| e.to_string())?;
            session.connected = false;
            session.status_message = msg.clone();
            session.last_error_code = Some("0x10019".to_string());
            report_error(&msg);
            return Err(msg);
        }
        Err(PpcVersionError::Query(detail)) => {
            log::error!("PPC version query failed: {}", detail);
            let msg = crate::i18n::t(&lang, "errors.ppc.versionCheckFailed", &[]);
            let mut session = state.session.lock().map_err(|e| e.to_string())?;
            session.connected = false;
            session.status_message = msg.clone();
            session.last_error_code = Some("0x10019".to_string());
            report_error(&msg);
            return Err(msg);
        }
    };

    // ── Register ─────────────────────────────────────────────────────
    let app_hash = match register_with_ppc() {
        Ok(hash) => hash,
        Err(detail) => {
            log::error!("PPC registration failed: {}", detail);
            let msg = crate::i18n::t(&lang, "errors.ppc.registerFailed", &[("code", "0x10005")]);
            let mut session = state.session.lock().map_err(|e| e.to_string())?;
            session.connected = false;
            session.status_message = msg.clone();
            session.last_error_code = Some("0x10005".to_string());
            report_error(&msg);
            return Err(msg);
        }
    };

    // ── Authenticate (optional on PPC 0.1.1) ─────────────────────────
    let authenticated = match authenticate_with_ppc(&app_hash) {
        Ok(()) => true,
        Err(e) => {
            log::warn!("PPC AUTH failed (optional since v0.1.1, continuing): {}", e);
            false
        }
    };

    // ── Success ──────────────────────────────────────────────────────
    let mut session = state.session.lock().map_err(|e| e.to_string())?;
    session.connected = true;
    session.app_hash = Some(app_hash);
    session.ppc_version = Some(ppc_version.clone());
    session.status_message = if authenticated {
        format!("Connected to PPC v{}", ppc_version)
    } else {
        format!("Connected to PPC v{} (unauthenticated)", ppc_version)
    };
    session.last_error_code = None;

    // Clear the failure cooldown now that PPC is confirmed reachable
    *PPC_FAIL_COOLDOWN.lock().unwrap() = None;

    Ok(session.clone())
}

/// Report an error through PPC (used by the frontend for all error surfaces).
#[tauri::command]
pub fn ppc_report_error(message: String) -> bool {
    report_error(&message)
}

/// Get current PPC session status (without connecting).
#[tauri::command]
pub fn ppc_status(state: tauri::State<'_, PpcState>) -> PpcSession {
    let session = state.session.lock().unwrap_or_else(|e| {
        log::error!("PPC state lock poisoned: {}", e);
        std::process::abort();
    });
    session.clone()
}

/// Send a raw command to PPC. Requires an active session.
///
/// PPC v0.1.1 accepts every command without `AUTH` (global signature
/// relaxation), while older builds expect an identity on each new TCP
/// connection. `AUTH` is therefore sent only when a hash exists and a rejected
/// AUTH is logged instead of aborting the command.
#[tauri::command]
pub fn ppc_send_command(
    state: tauri::State<'_, PpcState>,
    command: String,
) -> Result<String, String> {
    let (hash, connected) = {
        let session = state.session.lock().map_err(|e| e.to_string())?;
        (session.app_hash.clone(), session.connected)
    };

    if !connected {
        return Err("PPC not connected, call ppc_connect first".to_string());
    }

    let addr = format!("{}:{}", PPC_HOST, PPC_PORT);
    let mut stream = TcpStream::connect_timeout(
        &addr
            .parse()
            .map_err(|e| format!("Invalid address: {}", e))?,
        Duration::from_secs(TIMEOUT_SECS),
    )
    .map_err(|e| format!("Failed to connect to PPC: {}", e))?;

    stream
        .set_read_timeout(Some(Duration::from_secs(TIMEOUT_SECS)))
        .map_err(|e| format!("Failed to set read timeout: {}", e))?;
    stream
        .set_write_timeout(Some(Duration::from_secs(TIMEOUT_SECS)))
        .map_err(|e| format!("Failed to set write timeout: {}", e))?;

    let mut buf = [0u8; 4096];

    // Optional AUTH on the same connection (best effort).
    if let Some(hash) = hash {
        stream
            .write_all(format!("AUTH {}\n", hash).as_bytes())
            .map_err(|e| format!("Failed to send AUTH: {}", e))?;

        let auth_n = stream
            .read(&mut buf)
            .map_err(|e| format!("Failed to read AUTH response: {}", e))?;
        let auth_response = String::from_utf8_lossy(&buf[..auth_n]).to_string();
        let auth_code = parse_status_code(&auth_response).unwrap_or_default();
        if !is_success_code(&auth_code) {
            log::warn!(
                "PPC AUTH rejected ({}); continuing because v0.1.1 does not require it",
                auth_code
            );
        }
    }

    // Send the actual command
    let cmd_bytes = format!("{}\n", command);
    stream
        .write_all(cmd_bytes.as_bytes())
        .map_err(|e| format!("Failed to send command: {}", e))?;

    let n = stream
        .read(&mut buf)
        .map_err(|e| format!("Failed to read response: {}", e))?;

    let response =
        String::from_utf8(buf[..n].to_vec()).map_err(|e| format!("UTF-8 decode error: {}", e))?;

    // Parse status code and map to readable message
    let code = parse_status_code(&response).unwrap_or_default();
    if !is_success_code(&code) {
        let msg = map_ppc_code(&code);
        log::warn!(
            "PPC command '{}' returned error: {} ({})",
            command,
            msg,
            code
        );
    }

    Ok(response)
}

/// Disconnect from PPC (clears the session state).
#[tauri::command]
pub fn ppc_disconnect(state: tauri::State<'_, PpcState>) -> Result<(), String> {
    let mut session = state.session.lock().map_err(|e| e.to_string())?;
    session.connected = false;
    session.status_message = "Disconnected".to_string();
    Ok(())
}

/// Get the list of PPC error codes and their descriptions.
/// Useful for the frontend to display error reference.
#[tauri::command]
pub fn ppc_error_codes() -> Vec<(String, String)> {
    vec![
        ("0x00000".to_string(), "Success".to_string()),
        (
            "0x00001".to_string(),
            "App registered successfully".to_string(),
        ),
        (
            "0x00002".to_string(),
            "App updated successfully".to_string(),
        ),
        ("0x00003".to_string(), "DLL loaded successfully".to_string()),
        (
            "0x00004".to_string(),
            "Command executed successfully".to_string(),
        ),
        ("0x10000".to_string(), "Unknown error".to_string()),
        ("0x10001".to_string(), "Unsupported command".to_string()),
        ("0x10002".to_string(), "Resource not found".to_string()),
        (
            "0x10003".to_string(),
            "Signature verification failed".to_string(),
        ),
        ("0x10004".to_string(), "Hash mismatch".to_string()),
        ("0x10005".to_string(), "Registration failed".to_string()),
        ("0x10006".to_string(), "Update failed".to_string()),
        (
            "0x10007".to_string(),
            "App not registered or not authenticated".to_string(),
        ),
        ("0x10008".to_string(), "Invalid parameters".to_string()),
        ("0x10009".to_string(), "File not found".to_string()),
        ("0x10010".to_string(), "Permission denied".to_string()),
        ("0x10011".to_string(), "DLL not found".to_string()),
        ("0x10012".to_string(), "DLL load failed".to_string()),
        ("0x10013".to_string(), "Config parse failed".to_string()),
        ("0x10014".to_string(), "Database error".to_string()),
        ("0x10015".to_string(), "Connection failed".to_string()),
        ("0x10016".to_string(), "Timeout".to_string()),
        ("0x10017".to_string(), "PPC server not running".to_string()),
        (
            "0x10018".to_string(),
            "PPC authentication failed".to_string(),
        ),
        (
            "0x10019".to_string(),
            "PPC version not supported".to_string(),
        ),
        ("0x10020".to_string(), "PPC response invalid".to_string()),
        ("0x10021".to_string(), "Window pin failed".to_string()),
        ("0x10022".to_string(), "Window not found".to_string()),
        (
            "0x10023".to_string(),
            "Hotkey registration failed".to_string(),
        ),
        ("0x10024".to_string(), "Config save failed".to_string()),
        ("0x10025".to_string(), "Config load failed".to_string()),
        (
            "0x10026".to_string(),
            "State persistence failed".to_string(),
        ),
        (
            "0x10027".to_string(),
            "Administrator privileges required".to_string(),
        ),
        (
            "0x10028".to_string(),
            "Localization load failed".to_string(),
        ),
        (
            "0x10029".to_string(),
            "Single instance violation".to_string(),
        ),
        (
            "0x10030".to_string(),
            "Hook installation failed".to_string(),
        ),
        ("0x10031".to_string(), "Log write failed".to_string()),
        ("0x10032".to_string(), "DLL call failed".to_string()),
        ("0x20000".to_string(), "Waiting for signature".to_string()),
        ("0x20001".to_string(), "Waiting for app info".to_string()),
        ("0x20002".to_string(), "Processing".to_string()),
        ("0x20003".to_string(), "Initializing".to_string()),
        ("0x20004".to_string(), "Loading config".to_string()),
        ("0x20005".to_string(), "Executing command".to_string()),
        ("0x20006".to_string(), "Connecting".to_string()),
        ("0x20007".to_string(), "Registering".to_string()),
        ("0x20008".to_string(), "Authenticating".to_string()),
        ("0x20009".to_string(), "Reconnecting".to_string()),
        (
            "0x30000".to_string(),
            "Deprecated command warning".to_string(),
        ),
        ("0x30001".to_string(), "Old version warning".to_string()),
        (
            "0x30002".to_string(),
            "High memory usage warning".to_string(),
        ),
        ("0x30003".to_string(), "PPC reconnect warning".to_string()),
        (
            "0x30004".to_string(),
            "Fallback behavior warning".to_string(),
        ),
        (
            "0x30005".to_string(),
            "Language fallback warning".to_string(),
        ),
    ]
}
